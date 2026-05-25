// SPDX-FileCopyrightText: Copyright (c) 2026 Byte Facets
// SPDX-License-Identifier: MIT

use crate::num_utils::next_power_of_2;
use rustc_hash::FxHasher;
use std::default::Default;
use std::fmt;
use std::hash::BuildHasherDefault;
use std::hash::{BuildHasher, Hash};

/// The maximum capacity that can be used.
pub(crate) const DEFAULT_INITIAL_CAPACITY: usize = 16;

/// Default load factor for hash-based collections.
const DEFAULT_LOAD_FACTOR: f64 = 0.75;

/// An abstract base class for indexed collections.
pub struct IndexedSet<K> {
    /// The hash seed.
    hasher: BuildHasherDefault<FxHasher>,
    /// The total number of entries in the hash table.
    capacity: usize,
    /// The number of occupied slots in the hash table.
    size: usize,
    /// The tail of the free list
    free_list: i32,
    /// The maximum number of entries the map can contain before resizing.
    resize_threshold: usize,
    next_unused_entry: usize,
    modification_count: usize,
    max_head: usize,
    load_factor: f64,
    // hash table implementation
    heads: Vec<i32>,
    nexts: Vec<i32>,
    /// The values of the set.
    keys: Vec<K>,
}

struct FindResult {
    head: Option<usize>,
    entry: Option<usize>,
    prev_entry: Option<usize>,
}

impl<K: Default + Clone + PartialEq + Hash> IndexedSet<K> {
    /// Creates a new `IndexedSet` with the given initial capacity and load factor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `IndexedSet` with the given initial capacity and load factor.
    pub fn with_capacity(initial_capacity: usize, load_factor: f64) -> Self {
        let capacity =
            next_power_of_2((initial_capacity.max(2) as u32).try_into().unwrap());
        let resize_threshold = (capacity as f64 * load_factor) as usize;

        Self {
            hasher: BuildHasherDefault::<FxHasher>::default(),
            capacity,
            size: 0,
            max_head: 0,
            free_list: -1,
            modification_count: 0,
            next_unused_entry: 0,
            load_factor,
            resize_threshold,
            heads: vec![-1; capacity],
            nexts: vec![-1; capacity],
            keys: vec![K::default(); capacity],
        }
    }

    /// The underlying capacity of the set
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// The number of items in the set
    pub fn len(&self) -> usize {
        self.size
    }

    /// Whether the set has no items
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Adds a key to the set and returns a tuple with whether the key was
    /// added (true), or already in the set (false), and the entry where they key was stored.
    /// The returned value is a contract that the key will be addressable in `get_key_at`
    /// or `remove_at`
    /// Returns a tuple indicating:
    /// - Whether the key was newly inserted (true) or already existed (false)
    /// - The entry index in the map
    pub fn insert(&mut self, key: K) -> (bool, usize) {
        if let Some(entry) = self.lookup_entry(&key) {
            return (false, entry);
        }

        let entry = self.allocate_new_entry();
        if entry >= self.resize_threshold {
            self.rehash();
        }

        let head = self.compute_head(&key);
        self.register_new_entry_for_head(head as i32, entry);
        self.keys[entry] = key;

        // some light record-keeping
        self.max_head = std::cmp::max(self.max_head, head);
        self.size += 1;
        self.modification_count += 1;

        (true, entry)
    }

    /// Removes the key from the collection, returning the entry that was freed,
    /// or None if the key was not found.
    pub fn remove(&mut self, key: &K) -> Option<usize> {
        if let Some(find_result) = self.find(key) {
            self.remove_internal(find_result, true)
        } else {
            None
        }
    }

    /// Removes the key at the given entry.
    pub fn remove_at(&mut self, entry: usize) {
        let head = self.get_head(entry as i32);
        let mut prev = -1;
        let mut e = self.heads[head as usize];
        while e >= 0 {
            if e != entry as i32 {
                prev = e;
                e = self.nexts[e as usize];
                continue;
            }

            let next = self.nexts[entry];
            if prev < 0 {
                self.heads[head as usize] = next; // new head
            } else {
                self.nexts[prev as usize] = next;
            }

            self.free_reserved_entry(entry);
            self.size -= 1;
            self.modification_count += 1;
            return;
        }
    }

    /// Removes the entry from the hash table, but leaves the key in place. This can be
    /// useful in compound operations where you might process the key again, or just encounter a new
    /// key, but don't want to re-allocate the entry. Once you're ready to allow the reallocation
    /// of the entry, use the freeReservedEntry method. If you don't free the reserved entry later,
    /// the set will never use the entry again, and result in a memory leek.
    pub fn remove_at_and_reserve(&mut self, entry: usize) {
        let key = &self.keys[entry];
        if let Some(find_result) = self.find(key) {
            self.remove_internal(find_result, false);
        }
    }

    /// Returns the entry for the given key or None if the key is not found.
    pub fn lookup_entry(&self, key: &K) -> Option<usize> {
        let head = self.compute_head(key);
        let mut entry = self.heads[head];
        while entry >= 0 {
            if *key == self.keys[entry as usize] {
                return Some(entry as usize);
            }
            entry = self.nexts[entry as usize];
        }
        None
    }

    /// Returns the key at the given entry
    pub fn get_key_at(&self, entry: usize) -> &K {
        &self.keys[entry]
    }

    /// Whether the key exists in the collection
    pub fn contains(&self, key: &K) -> bool {
        self.lookup_entry(key).is_some()
    }

    /// Clears the contents of the set
    pub fn clear(&mut self) {
        if self.size > 0 {
            let lim = std::cmp::min(self.heads.len(), self.max_head + 1);
            self.heads[0..lim].fill(-1);

            let lim = std::cmp::min(self.nexts.len(), self.next_unused_entry);
            self.nexts[0..lim].fill(-1);
            self.keys[0..lim].fill(K::default());

            self.size = 0;
            self.max_head = 0;
            self.free_list = -1;
        }
    }

    fn calculate_new_capacity(&mut self) {
        // double
        let new_capacity = (self.capacity as u32)
            .checked_shl(1)
            .expect("capacity overflow");
        self.capacity = new_capacity as usize;
        self.resize_threshold = (new_capacity as f64 * self.load_factor) as usize;
    }

    fn find(&self, key: &K) -> Option<FindResult> {
        let head = self.compute_head(key);
        let head_entry = self.heads[head];
        if head_entry < 0 {
            return None;
        }
        let mut prev = -1;
        let mut e = head_entry;
        while e >= 0 {
            if key != &self.keys[e as usize] {
                prev = e;
                e = self.nexts[e as usize];
                continue;
            }
            return Some(FindResult {
                head: Some(head),
                entry: Some(e as usize),
                prev_entry: if prev >= 0 { Some(prev as usize) } else { None },
            });
        }
        None
    }

    fn get_head(&self, entry: i32) -> i32 {
        let mut e = entry;
        while e >= 0 {
            e = self.nexts[e as usize];
        }
        -e - 1
    }

    fn compute_head(&self, _key: &K) -> usize {
        let hash = self.hasher.hash_one(_key);
        (hash & (self.heads.len() - 1) as u64) as usize
    }

    #[allow(clippy::needless_range_loop)]
    fn rehash(&mut self) {
        let old_heads = self.heads.clone();
        let old_nexts = self.nexts.clone();

        self.calculate_new_capacity();

        self.heads.fill(-1);
        self.nexts.fill(-1);
        self.heads.resize(self.capacity, -1);
        self.nexts.resize(self.capacity, -1);
        self.keys.resize(self.capacity, K::default());

        // walk through the old heads and process each list
        let len = std::cmp::min(self.max_head + 1, old_heads.len()); // limit loop
        self.max_head = 0;
        for i in 0..len {
            let mut entry = old_heads[i];
            while entry >= 0 {
                let new_head = self.compute_head(&self.keys[entry as usize]);
                self.register_new_entry_for_head(new_head as i32, entry as usize);
                self.max_head = std::cmp::max(self.max_head, new_head);
                entry = old_nexts[entry as usize];
            }
        }
        self.modification_count += 1;
    }

    fn register_new_entry_for_head(&mut self, head: i32, entry: usize) {
        // and link the entry in the hash table
        if self.heads[head as usize] >= 0 {
            // if there's a head, point this entry to the head
            self.nexts[entry] = self.heads[head as usize];
        } else {
            // store reference to the head in negative space so that we can find
            // the head when attempting to remove_at(entry)
            self.nexts[entry] = -head - 1;
        }
        self.heads[head as usize] = entry as i32;
    }

    /// Used in combination with the removeAtAndReserve method, this clears the key at the
    /// reserved entry and puts the entry back on the free list. This does not check whether
    /// you first reserved the entry. Calling this with active entries can corrupt the collection.
    pub fn free_reserved_entry(&mut self, entry: usize) {
        self.nexts[entry] = self.free_list;
        self.free_list = entry as i32;
        self.keys[entry] = K::default();
    }

    fn allocate_new_entry(&mut self) -> usize {
        if self.free_list != -1 {
            let entry = self.free_list as usize;
            self.free_list = self.nexts[entry];
            entry
        } else {
            let entry = self.next_unused_entry;
            self.next_unused_entry += 1;
            entry
        }
    }

    fn remove_internal(&mut self, find_result: FindResult, free: bool) -> Option<usize> {
        if let Some(entry) = find_result.entry {
            let next = self.nexts[entry];
            if let Some(prev) = find_result.prev_entry {
                self.nexts[prev] = next;
            } else {
                self.heads[find_result.head.unwrap()] = next; // new head
            }

            if free {
                self.free_reserved_entry(entry);
            } else {
                self.nexts[entry] = -1;
                // leave key and value set
            }
            self.size -= 1;
            self.modification_count += 1;
            return find_result.entry;
        }
        None
    }
    pub fn iter(&self) -> impl Iterator<Item = &K> {
        self.entry_iter().map(|(_, key)| key)
    }

    pub fn entry_iter(&self) -> Iter<'_, K> {
        Iter {
            set: self,
            initialized: false,
            internal_mod_count: self.modification_count,
            head: -1,
            entry: -1,
            prev: -1,
        }
    }
}

impl<K: Default + Clone + PartialEq + Hash> Default for IndexedSet<K> {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_INITIAL_CAPACITY, DEFAULT_LOAD_FACTOR)
    }
}

impl<T: fmt::Display> fmt::Display for IndexedSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let lim = std::cmp::max(self.heads.len(), self.max_head + 1);
        let mut first = true;
        for h in 0..lim {
            let mut entry = self.heads[h];
            if !first && entry >= 0 {
                write!(f, ", ")?;
            }
            while entry >= 0 {
                write!(f, "{}", self.keys[entry as usize])?;
                entry = self.nexts[entry as usize];
                first = false;
                if entry >= 0 {
                    write!(f, ", ")?;
                }
            }
        }
        write!(f, "}}")
    }
}

pub struct Iter<'a, K> {
    set: &'a IndexedSet<K>,
    initialized: bool,
    internal_mod_count: usize,
    head: i32,
    entry: i32,
    prev: i32,
}

impl<'a, K> Iter<'a, K> {
    fn next_head(&mut self) {
        self.head += 1;
        let limit = std::cmp::min(self.set.max_head + 1, self.set.heads.len()) as i32;
        while self.head < limit {
            self.entry = self.set.heads[self.head as usize];
            if self.entry >= 0 {
                self.prev = -1;
                return;
            }
            self.head += 1;
        }
    }
}

impl<'a, K> Iterator for Iter<'a, K> {
    type Item = (usize, &'a K);

    fn next(&mut self) -> Option<Self::Item> {
        if self.set.modification_count != self.internal_mod_count {
            panic!("set was modified outside of iterator");
        }
        if !self.initialized {
            self.next_head();
            self.initialized = true;
        }
        if self.entry >= 0 {
            let entry = self.entry as usize;
            let result = (entry, &self.set.keys[entry]);
            self.prev = self.entry;
            self.entry = self.set.nexts[self.entry as usize];
            if self.entry < 0 {
                self.next_head();
            }
            return Some(result);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ops::Range;
    use std::collections::{BTreeSet, HashSet};

    struct Helper {
        indexed_set: IndexedSet<i32>,
        reference_set: BTreeSet<i32>,
        count: usize,
    }

    impl Helper {
        /// helper method to maintain set insert parity between our set and a reference set
        fn insert(&mut self, key: i32) -> (bool, usize) {
            self.reference_set.insert(key);
            let entry = self.indexed_set.insert(key);
            entry
        }

        /// helper method to maintain set remove parity between our set and a reference set
        fn remove(&mut self, key: i32) -> Option<usize> {
            self.reference_set.remove(&key);
            self.indexed_set.remove(&key)
        }

        fn assert_sets_equal(&mut self) {
            assert_eq!(self.indexed_set.len(), self.reference_set.len());
            self.count += 1;
            // println!("ASSERT {}     sizes: {} vs {}", self.count, self.indexed_set.len(), self.reference_set.len());
            // println!("ASSERT {}   indexed: {}", self.count, self.indexed_set.to_string());
            // println!("ASSERT {} reference: {:?}", self.count, self.reference_set);

            let mut ref_copy = self.reference_set.clone();
            for key in self.indexed_set.iter() {
                assert!(ref_copy.contains(key));
                ref_copy.remove(key);
            }
            assert_eq!(ref_copy.len(), 0);

            for key in self.reference_set.iter() {
                assert!(
                    self.indexed_set.contains(key),
                    "{:?} not found in indexed_set set",
                    key
                );
            }
        }
    }

    fn setup() -> Helper {
        Helper {
            indexed_set: IndexedSet::default(),
            reference_set: BTreeSet::new(),
            count: 0,
        }
    }

    /// Iterates a range, computing the heads of the keys, and stopping when it's computed the
    /// given number of keys.
    fn pick_keys_in_bucket<F>(hash_func: F, range: Range<i32>, num_keys: usize) -> (usize, Vec<i32>)
    where
        F: Fn(&i32) -> usize,
    {
        let mut headsets: [HashSet<i32>; 16] = std::array::from_fn(|_| HashSet::new());
        let mut heads_to_test = Vec::new();
        for i in range {
            let head = hash_func(&i);
            headsets[head].insert(i);
            if headsets[head].len() == num_keys {
                for k in headsets[head].iter() {
                    heads_to_test.push(k.clone());
                }
                return (head, heads_to_test);
            }
        }
        panic!("no keys found");
    }

    #[test]
    fn test_insert_and_get_in_order() {
        let mut helper = setup();

        let entry0 = helper.insert(30);
        let entry1 = helper.insert(31);
        let entry2 = helper.insert(32);

        assert_eq!(entry0, (true, 0));
        assert_eq!(entry1, (true, 1));
        assert_eq!(entry2, (true, 2));

        assert_eq!(*helper.indexed_set.get_key_at(entry0.1), 30);
        assert_eq!(*helper.indexed_set.get_key_at(entry1.1), 31);
        assert_eq!(*helper.indexed_set.get_key_at(entry2.1), 32);

        helper.assert_sets_equal();
    }

    #[test]
    fn should_report_existing_key() {
        let mut helper = setup();
        assert_eq!(helper.insert(30), (true, 0));
        assert_eq!(helper.insert(30), (false, 0));
        helper.assert_sets_equal();
    }

    /// Creates a hash bucket under one head that is 3 entries long, and removes the key
    /// at the END of the bucket with the expectation that the set is intact and navigable.
    #[test]
    fn should_remove_end_of_bucket() {
        let mut helper = setup();
        let (head, keys_to_test) =
            pick_keys_in_bucket(|k: &i32| helper.indexed_set.compute_head(&k), 0..48, 3);
        let mut entries = Vec::new();
        for key in keys_to_test.clone() {
            entries.push(helper.insert(key).1 as i32);
        }
        helper.assert_sets_equal();
        assert_eq!(helper.indexed_set.heads[head], entries[2]);
        assert_eq!(
            helper.remove(keys_to_test[0].clone()).unwrap(),
            entries[0] as usize
        );
        assert_eq!(helper.indexed_set.heads[head], entries[2]);

        helper.assert_sets_equal();
    }

    /// Creates a hash bucket under one head that is 3 entries long, and removes the key
    /// at the START of the bucket with the expectation that the set is intact and navigable.
    #[test]
    fn should_remove_start_of_bucket() {
        let mut helper = setup();
        let (head, keys_to_test) =
            pick_keys_in_bucket(|k: &i32| helper.indexed_set.compute_head(&k), 0..48, 3);
        let mut entries = Vec::new();
        for key in keys_to_test.clone() {
            entries.push(helper.insert(key).1 as i32);
        }

        helper.assert_sets_equal();
        assert_eq!(helper.indexed_set.heads[head], entries[2]);
        assert_eq!(helper.remove(keys_to_test[2]).unwrap(), entries[2] as usize);
        assert_eq!(helper.indexed_set.heads[head], entries[1]);

        helper.assert_sets_equal();
    }

    /// The set should use the most recently freed entries before allocating additional.
    #[test]
    fn should_use_free_list() {
        let mut helper = setup();
        for key in 30..40 {
            helper.insert(key);
            helper.assert_sets_equal();
        }

        let free35 = helper.remove(35).unwrap();
        helper.assert_sets_equal();
        let free37 = helper.remove(37).unwrap();
        helper.assert_sets_equal();

        assert_eq!(free37, helper.insert(90).1);
        helper.assert_sets_equal();
        assert_eq!(free35, helper.insert(91).1);
        helper.assert_sets_equal();
        assert_eq!(10, helper.insert(92).1);

        helper.assert_sets_equal();
    }

    /// The set should grow to accommodate more keys.
    #[test]
    fn should_grow() {
        let mut helper = setup();
        let mut key_entries = Vec::new();
        for key in 0..64 {
            key_entries.push((key, helper.insert(key).1));
        }
        helper.assert_sets_equal();

        // keys and entries should remain stable after growing
        for key_entry in key_entries.iter() {
            assert_eq!(helper.indexed_set.get_key_at(key_entry.1), &key_entry.0);
            assert_eq!(
                helper.indexed_set.lookup_entry(&key_entry.0).unwrap(),
                key_entry.1
            );
        }
    }

    /// The set should not allocate entries that are "reserved" when deleted.
    #[test]
    fn should_not_use_reserved_entries() {
        let mut helper = setup();
        for key in 30..40 {
            helper.insert(key);
        }
        let entry35 = helper.indexed_set.lookup_entry(&35).unwrap();
        let entry36 = helper.indexed_set.lookup_entry(&36).unwrap();
        helper.indexed_set.remove_at_and_reserve(entry35);
        helper.indexed_set.remove_at_and_reserve(entry36);
        helper.reference_set.remove(&35);
        helper.reference_set.remove(&36);
        helper.assert_sets_equal();

        assert_eq!(helper.insert(41), (true, 10));
        assert_eq!(helper.insert(42), (true, 11));

        helper.indexed_set.free_reserved_entry(entry35);
        helper.indexed_set.free_reserved_entry(entry36);

        assert_eq!(helper.insert(43), (true, entry36));
        assert_eq!(helper.insert(44), (true, entry35));
        helper.assert_sets_equal();
    }

    /// The set should be wiped when cleared.
    #[test]
    fn should_clear() {
        let mut helper = setup();
        for k in 40..140 {
            helper.insert(k);
        }
        helper.assert_sets_equal();
        helper.indexed_set.clear();
        helper.reference_set.clear();
        helper.assert_sets_equal();
        for k in 0..helper.indexed_set.keys.len() {
            assert_eq!(helper.indexed_set.keys[k], 0);
        }
        for k in 0..helper.indexed_set.heads.len() {
            assert_eq!(helper.indexed_set.heads[k], -1);
            assert_eq!(helper.indexed_set.nexts[k], -1);
        }
    }
}
