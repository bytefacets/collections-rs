use crate::ensure_size;
use crate::pack::{pack_usize, unpack_usize};
use crate::{ensure_entry, IndexedSet};

/// A collection which can hold and access usize associations in 1-n relationships. It is
/// called "Compact" because the values should be 0-based as they are typically array offsets
/// internally for performance and memory reasons. If your domain values are not compact,
/// they can be made compact by first performing a range transformation using an IndexedSet.
pub struct CompactOneToMany {
    mappings: IndexedSet<u128>,
    left: Side,
}

impl CompactOneToMany {
    /// Default factory method
    pub fn new() -> Self {
        Self::with_capacity(16, 16)
    }

    /// Factory method with initial sizes
    pub fn with_capacity(initial_lefts: usize, initial_mapping_capacity: usize) -> Self {
        CompactOneToMany {
            mappings: IndexedSet::with_capacity(initial_mapping_capacity, 0.75),
            left: Side::with_capacity(initial_lefts, initial_mapping_capacity),
        }
    }

    /// The number of mappings in the collection
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Whether the collection is empty
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Whether the collection contains the mapping
    pub fn contains(&self, left_key: usize, right_key: usize) -> bool {
        self.mappings.contains(&pack_usize((left_key, right_key)))
    }

    /// Returns the entry of the given mapping if there is one
    pub fn lookup_entry(&self, left_key: usize, right_key: usize) -> Option<usize> {
        self.mappings
            .lookup_entry(&pack_usize((left_key, right_key)))
    }

    /// Puts the given mapping into the collection and returns a tuple indicating
    /// (.0) if the mapping was new and (.1) the entry of the mapping.
    pub fn put(&mut self, left_key: usize, right_key: usize) -> (bool, usize) {
        let key = pack_usize((left_key, right_key));
        let result = self.mappings.insert(key);
        if !result.0 {
            return result;
        }
        self.left.insert_entry(left_key, result.1);
        result
    }

    /// Removes the given mapping, if it's present, and returns the freed entry if
    /// the mapping was removed.
    pub fn remove(&mut self, left_key: usize, right_key: usize) -> Option<usize> {
        let key = pack_usize((left_key, right_key));
        let result = self.mappings.remove(&key);
        if let Some(entry) = result {
            self.left.remove_entry(left_key, entry)
        }
        result
    }

    /// Returns a handle to the mappings for the given left key
    pub fn with_left(&self, left_key: usize) -> Mapping<'_> {
        Mapping {
            o2m: self,
            side: &self.left,
            key: Some(left_key),
        }
    }

    /// Removes the mapping at the given entry
    pub fn remove_at(&mut self, entry: usize) {
        let packed_key = self.mappings.get_key_at(entry);
        let key = unpack_usize(*packed_key);
        self.mappings.remove_at(entry);
        self.left.remove_entry(key.0, entry);
    }

    /// Clears the collection
    pub fn clear(&mut self) {
        self.mappings.clear();
        self.left.clear();
    }

    /// Returns the left/right mapping for the given entry
    pub fn get_at(&self, entry: usize) -> (usize, usize) {
        let packed_key = self.mappings.get_key_at(entry);
        unpack_usize(*packed_key)
    }

    /// Removes the entry from the collection, but leaves the key in place. This can be
    /// useful in compound operations where you might process the key again, or just encounter a new
    /// key, but don't want to re-allocate the entry. Once you're ready to allow the reallocation
    /// of the entry, use the freeReservedEntry method. If you don't free the reserved entry later,
    /// the set will never use the entry again, and result in a memory leek.
    pub fn remove_at_and_reserve(&mut self, entry: usize) {
        let packed_key = *self.mappings.get_key_at(entry);
        self.mappings.remove_at_and_reserve(entry);
        self.left.remove_entry(unpack_usize(packed_key).0, entry);
    }

    /// Used in combination with the remove_at_and_reserve method, this clears the key at the
    /// reserved entry and puts the entry back on the free list. This does not check whether
    /// you first reserved the entry. Calling this with active entries can corrupt the collection.
    pub fn free_reserved_entry(&mut self, entry: usize) {
        self.mappings.free_reserved_entry(entry);
    }
}

/// A collection which can hold and access usize associations in n-n relationships. It is
/// called "Compact" because the values should be 0-based as they are typically array offsets
/// internally for performance and memory reasons. If your domain values are not compact,
/// they can be made compact by first performing a range transformation using an IndexedSet.
struct CompactManyToMany {
    base: CompactOneToMany,
    right: Side,
}
impl CompactManyToMany {
    /// Default factory method
    pub fn new() -> Self {
        Self::with_capacity(16, 16, 16)
    }

    /// Factory method with initial sizes
    pub fn with_capacity(
        initial_lefts: usize,
        initial_rights: usize,
        initial_mapping_capacity: usize,
    ) -> Self {
        CompactManyToMany {
            base: CompactOneToMany::with_capacity(initial_lefts, initial_mapping_capacity),
            right: Side::with_capacity(initial_rights, initial_mapping_capacity),
        }
    }

    /// The number of mappings in the collection
    pub fn len(&self) -> usize {
        self.base.len()
    }

    /// Whether the collection is empty
    pub fn is_empty(&self) -> bool {
        self.base.is_empty()
    }

    /// Whether the collection contains the mapping
    pub fn contains(&mut self, left_key: usize, right_key: usize) -> bool {
        self.base.contains(left_key, right_key)
    }

    /// Returns the entry of the given mapping if there is one
    pub fn lookup_entry(&self, left_key: usize, right_key: usize) -> Option<usize> {
        self.base.lookup_entry(left_key, right_key)
    }

    /// Puts the given mapping into the collection and returns a tuple indicating
    /// (.0) if the mapping was new and (.1) the entry of the mapping.
    pub fn put(&mut self, left_key: usize, right_key: usize) -> (bool, usize) {
        let result = self.base.put(left_key, right_key);
        if !result.0 {
            return result;
        }
        self.right.insert_entry(right_key, result.1);
        result
    }

    /// Removes the given mapping, if it's present, and returns the freed entry if
    /// the mapping was removed.
    pub fn remove(&mut self, left_key: usize, right_key: usize) -> Option<usize> {
        let result = self.base.remove(left_key, right_key);
        if let Some(entry) = result {
            self.right.remove_entry(right_key, entry)
        }
        result
    }

    /// Returns a handle to the mappings for the given left key
    pub fn with_left(&self, left_key: usize) -> Mapping<'_> {
        self.base.with_left(left_key)
    }

    /// Returns a handle to the mappings for the given right key
    pub fn with_right(&self, right_key: usize) -> Mapping<'_> {
        Mapping {
            o2m: &self.base,
            side: &self.right,
            key: Some(right_key),
        }
    }

    /// Removes the mapping at the given entry
    pub fn remove_at(&mut self, entry: usize) {
        let packed_key = self.base.mappings.get_key_at(entry);
        let key = unpack_usize(*packed_key);
        self.base.mappings.remove_at(entry);
        self.base.left.remove_entry(key.0, entry);
        self.right.remove_entry(key.1, entry);
    }

    /// Clears the collection
    pub fn clear(&mut self) {
        self.base.clear();
        self.right.clear();
    }

    /// Returns the left/right mapping for the given entry
    pub fn get_at(&self, entry: usize) -> (usize, usize) {
        let packed_key = self.base.mappings.get_key_at(entry);
        unpack_usize(*packed_key)
    }

    /// Removes the entry from the collection, but leaves the key in place. This can be
    /// useful in compound operations where you might process the key again, or just encounter a new
    /// key, but don't want to re-allocate the entry. Once you're ready to allow the reallocation
    /// of the entry, use the freeReservedEntry method. If you don't free the reserved entry later,
    /// the set will never use the entry again, and result in a memory leek.
    pub fn remove_at_and_reserve(&mut self, entry: usize) {
        let packed_key = *self.base.mappings.get_key_at(entry);
        self.base.remove_at_and_reserve(entry);
        self.right.remove_entry(unpack_usize(packed_key).1, entry);
    }

    /// Used in combination with the remove_at_and_reserve method, this clears the key at the
    /// reserved entry and puts the entry back on the free list. This does not check whether
    /// you first reserved the entry. Calling this with active entries can corrupt the collection.
    pub fn free_reserved_entry(&mut self, entry: usize) {
        self.base.free_reserved_entry(entry);
    }
}

impl Default for CompactOneToMany {
    fn default() -> CompactOneToMany {
        CompactOneToMany::new()
    }
}

impl Default for CompactManyToMany {
    fn default() -> CompactManyToMany {
        CompactManyToMany::new()
    }
}

pub struct Mapping<'a> {
    o2m: &'a CompactOneToMany,
    side: &'a Side,
    key: Option<usize>,
}

impl<'a> Mapping<'a> {
    /// The count of the mappings for the key
    pub fn count(&self) -> usize {
        self.key
            .map(|k| {
                if k < self.side.counts.len() {
                    self.side.counts[k]
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    /// Returns an Iterator of the mapping entries for the given key.
    pub fn iter(&self) -> impl Iterator<Item = LeftRightEntry> + use<'_> {
        let entry = self.key.and_then(|k| {
            if k < self.side.heads.len() {
                self.side.heads[k]
            } else {
                None
            }
        });
        OneToManyIterator {
            o2m: self.o2m,
            side: self.side,
            entry,
        }
    }
}

/// One side of a CompactOneToMany or CompactManyToMany.
struct Side {
    heads: Vec<Option<usize>>,
    nexts: Vec<Option<usize>>,
    prevs: Vec<Option<usize>>,
    counts: Vec<usize>,
}

impl Side {
    fn with_capacity(initial_capacity: usize, initial_mapping_capacity: usize) -> Self {
        Side {
            heads: vec![None; initial_capacity],
            counts: vec![0; initial_capacity],
            nexts: vec![None; initial_mapping_capacity],
            prevs: vec![None; initial_mapping_capacity],
        }
    }

    fn clear(&mut self) {
        self.heads.fill(None);
        self.nexts.fill(None);
        self.prevs.fill(None);
        self.counts.fill(0);
    }

    fn insert_entry(&mut self, key: usize, entry: usize) {
        ensure_size(&mut self.heads, key);
        ensure_size(&mut self.counts, key);
        ensure_entry(&mut self.nexts, entry);
        ensure_entry(&mut self.prevs, entry);

        if let Some(old_head) = self.heads[key] {
            self.nexts[entry] = Some(old_head);
            self.prevs[old_head] = Some(entry);
        }
        self.heads[key] = Some(entry); // always gets the new head
        self.counts[key] += 1;
    }

    fn remove_entry(&mut self, key: usize, entry: usize) {
        if let Some(old_head) = self.heads[key] {
            if old_head == entry {
                self.heads[key] = self.nexts[entry];
            } else {
                let prev = self.prevs[entry];
                let next = self.nexts[entry];
                if let Some(_prev) = prev {
                    self.nexts[_prev] = next;
                }
                if let Some(_next) = next {
                    self.prevs[_next] = prev;
                }
            }
        }
        self.prevs[entry] = None;
        self.nexts[entry] = None;
        self.counts[key] -= 1;
    }
}

pub struct OneToManyIterator<'a> {
    o2m: &'a CompactOneToMany,
    side: &'a Side,
    entry: Option<usize>,
}

pub struct LeftRightEntry {
    left: usize,
    right: usize,
    entry: usize,
}

impl<'a> Iterator for OneToManyIterator<'a> {
    type Item = LeftRightEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.entry {
            let key = self.o2m.mappings.get_key_at(entry);
            let left_right = unpack_usize(*key);
            self.entry = self.side.nexts[entry];
            Some(LeftRightEntry {
                left: left_right.0,
                right: left_right.1,
                entry,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod o2m_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn should_add() {
        let mut o2m = CompactOneToMany::new();
        assert!(o2m.is_empty());
        o2m.put(1, 10);
        assert!(!o2m.is_empty());
        o2m.put(3, 30);
        o2m.put(2, 20);
        assert_eq!(o2m.len(), 3);
        assert!(o2m.contains(1, 10));
        assert!(o2m.contains(2, 20));
        assert!(o2m.contains(3, 30));
        for left in 1..=3 {
            let mut set: HashSet<usize> = HashSet::new();
            for e in o2m.with_left(left).iter() {
                assert_eq!(e.left, left);
                set.insert(e.right);
            }
            assert_eq!(set, HashSet::from_iter(vec![left * 10]))
        }
    }

    #[test]
    fn should_make_list() {
        let mut o2m = CompactOneToMany::new();
        o2m.put(1, 10);
        o2m.put(1, 30);
        o2m.put(1, 20);
        assert_eq!(o2m.len(), 3);
        let mut set: HashSet<usize> = HashSet::new();
        for e in o2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(set, HashSet::from_iter(vec![10, 20, 30]))
    }

    #[test]
    fn should_remove_head() {
        let mut o2m = CompactOneToMany::new();
        for i in 1..=3 {
            o2m.put(1, i * 10);
        }
        let entry = o2m.lookup_entry(1, 30).unwrap();
        assert_eq!(o2m.remove(1, 30), Some(entry));

        let mut set: HashSet<usize> = HashSet::new();
        for e in o2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(set, HashSet::from_iter(vec![10, 20]))
    }

    #[test]
    fn should_remove_middle() {
        let mut o2m = CompactOneToMany::new();
        for i in 1..=3 {
            o2m.put(1, i * 10);
        }
        let entry = o2m.lookup_entry(1, 20).unwrap();
        assert_eq!(o2m.remove(1, 20), Some(entry));

        let mut set: HashSet<usize> = HashSet::new();
        for e in o2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(set, HashSet::from_iter(vec![10, 30]))
    }

    #[test]
    fn should_remove_tail() {
        let mut o2m = CompactOneToMany::new();
        for i in 1..=3 {
            o2m.put(1, i * 10);
        }
        let entry = o2m.lookup_entry(1, 10).unwrap();
        assert_eq!(o2m.remove(1, 10), Some(entry));

        let mut set: HashSet<usize> = HashSet::new();
        for e in o2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(set, HashSet::from_iter(vec![20, 30]))
    }

    #[test]
    fn should_manage_many() {
        let mut o2m = CompactOneToMany::new();
        let mut expected: HashSet<(usize, usize)> = HashSet::new();
        for left in 0..100 {
            for right in 0..100 {
                o2m.put(left, right * 10);
                expected.insert((left, right * 10));
            }
        }
        assert_eq!(expected.len(), 10000);
        assert_eq!(o2m.len(), 10000);
        for left in 0..100 {
            assert_eq!(o2m.with_left(left).count(), 100);
            for e in o2m.with_left(left).iter() {
                assert!(expected.remove(&(e.left, e.right)));
            }
        }
        assert_eq!(expected.len(), 0);
    }

    #[test]
    fn should_remove_at() {
        let mut o2m = CompactOneToMany::new();
        for i in 0..40 {
            o2m.put(1, i * 10);
        }
        o2m.remove_at(29);
        assert!(!o2m.contains(1, 290));
        let mut set: HashSet<usize> = HashSet::new();
        for e in o2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(
            set,
            HashSet::from_iter((0..40 as usize).filter(|i| *i != 29).map(|i| i * 10))
        );
    }

    #[test]
    fn should_get_at() {
        let mut o2m = CompactOneToMany::new();
        for i in 0..40 {
            o2m.put(1, i * 10);
        }
        assert_eq!(o2m.get_at(29), (1, 290));
    }

    #[test]
    fn should_clear() {
        let mut o2m = CompactOneToMany::new();
        for i in 0..40 {
            o2m.put(1 % 3, i * 10);
        }
        o2m.clear();
        assert_eq!(o2m.len(), 0);
        assert_eq!(o2m.mappings.len(), 0);
        assert_eq!(o2m.left.heads.iter().filter(|x| x.is_some()).count(), 0);
    }

    #[test]
    fn should_manage_reserved_entry() {
        let mut o2m = CompactOneToMany::new();
        for i in 0..40 {
            o2m.put(1, i * 10);
        }

        o2m.remove_at_and_reserve(29);
        assert!(!o2m.contains(1, 290));
        assert_eq!(o2m.len(), 39);
        assert_eq!(o2m.put(1, 400), (true, 40));

        let mut set: HashSet<usize> = HashSet::new();
        for e in o2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(
            set,
            HashSet::from_iter((0..=40 as usize).filter(|i| *i != 29).map(|i| i * 10))
        );

        o2m.free_reserved_entry(29);
        assert_eq!(o2m.put(1, 410), (true, 29)); // 29 from free list
    }

    #[test]
    fn should_return_empty_mapping_when_unknown_key() {
        let o2m = CompactOneToMany::new();
        assert_eq!(o2m.with_left(4).count(), 0);
        assert_eq!(o2m.with_left(100).iter().count(), 0)
    }
}

#[cfg(test)]
mod m2m_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn should_add() {
        let mut m2m = CompactManyToMany::new();
        assert!(m2m.is_empty());
        m2m.put(1, 10);
        assert!(!m2m.is_empty());
        m2m.put(3, 30);
        m2m.put(2, 20);
        assert_eq!(m2m.len(), 3);
        assert!(m2m.contains(1, 10));
        assert!(m2m.contains(2, 20));
        assert!(m2m.contains(3, 30));
        for left in 1..=3 {
            let mut set: HashSet<usize> = HashSet::new();
            for e in m2m.with_left(left).iter() {
                assert_eq!(e.left, left);
                set.insert(e.right);
            }
            assert_eq!(set, HashSet::from_iter(vec![left * 10]))
        }

        for left in 1..=3 {
            let right = left * 10;
            let mut set: HashSet<usize> = HashSet::new();
            for e in m2m.with_right(right).iter() {
                assert_eq!(e.right, right);
                set.insert(e.left);
            }
            assert_eq!(set, HashSet::from_iter(vec![left]))
        }
    }

    #[test]
    fn should_make_list() {
        let mut m2m = CompactManyToMany::new();
        m2m.put(1, 10);
        m2m.put(1, 30);
        m2m.put(1, 20);
        assert_eq!(m2m.len(), 3);
        let mut set: HashSet<usize> = HashSet::new();
        for e in m2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(set, HashSet::from_iter(vec![10, 20, 30]))
    }

    #[test]
    fn should_remove_head() {
        let mut m2m = CompactManyToMany::new();
        for i in 1..=3 {
            m2m.put(1, i * 10);
        }
        let entry = m2m.lookup_entry(1, 30).unwrap();
        assert_eq!(m2m.remove(1, 30), Some(entry));

        let mut set: HashSet<usize> = HashSet::new();
        for e in m2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(set, HashSet::from_iter(vec![10, 20]))
    }

    #[test]
    fn should_remove_middle() {
        let mut m2m = CompactManyToMany::new();
        for i in 1..=3 {
            m2m.put(1, i * 10);
        }
        let entry = m2m.lookup_entry(1, 20).unwrap();
        assert_eq!(m2m.remove(1, 20), Some(entry));

        let mut set: HashSet<usize> = HashSet::new();
        for e in m2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(set, HashSet::from_iter(vec![10, 30]))
    }

    #[test]
    fn should_remove_tail() {
        let mut m2m = CompactManyToMany::new();
        for i in 1..=3 {
            m2m.put(1, i * 10);
        }
        let entry = m2m.lookup_entry(1, 10).unwrap();
        assert_eq!(m2m.remove(1, 10), Some(entry));

        let mut set: HashSet<usize> = HashSet::new();
        for e in m2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(set, HashSet::from_iter(vec![20, 30]))
    }

    #[test]
    fn should_manage_many() {
        let mut m2m = CompactManyToMany::new();
        let mut expected: HashSet<(usize, usize)> = HashSet::new();
        for left in 0..100 {
            for right in 0..100 {
                m2m.put(left, right * 10);
                expected.insert((left, right * 10));
            }
        }
        assert_eq!(expected.len(), 10000);
        assert_eq!(m2m.len(), 10000);
        for left in 0..100 {
            assert_eq!(m2m.with_left(left).count(), 100);
            for e in m2m.with_left(left).iter() {
                assert!(expected.remove(&(e.left, e.right)));
            }
        }
        assert_eq!(expected.len(), 0);
    }

    #[test]
    fn should_remove_at() {
        let mut m2m = CompactManyToMany::new();
        for i in 0..40 {
            m2m.put(1, i * 10);
        }
        m2m.remove_at(29);
        assert!(!m2m.contains(1, 290));
        let mut set: HashSet<usize> = HashSet::new();
        for e in m2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(
            set,
            HashSet::from_iter((0..40 as usize).filter(|i| *i != 29).map(|i| i * 10))
        );
    }

    #[test]
    fn should_get_at() {
        let mut m2m = CompactManyToMany::new();
        for i in 0..40 {
            m2m.put(1, i * 10);
        }
        assert_eq!(m2m.get_at(29), (1, 290));
    }

    #[test]
    fn should_clear() {
        let mut m2m = CompactManyToMany::new();
        for i in 0..40 {
            m2m.put(1 % 3, i * 10);
        }
        m2m.clear();
        assert_eq!(m2m.len(), 0);
        assert_eq!(m2m.base.mappings.len(), 0);
        assert_eq!(
            m2m.base.left.heads.iter().filter(|x| x.is_some()).count(),
            0
        );
        assert_eq!(m2m.right.heads.iter().filter(|x| x.is_some()).count(), 0);
    }

    #[test]
    fn should_manage_reserved_entry() {
        let mut m2m = CompactManyToMany::new();
        for i in 0..40 {
            m2m.put(1, i * 10);
        }

        m2m.remove_at_and_reserve(29);
        assert!(!m2m.contains(1, 290));
        assert_eq!(m2m.len(), 39);
        assert_eq!(m2m.put(1, 400), (true, 40));

        let mut set: HashSet<usize> = HashSet::new();
        for e in m2m.with_left(1).iter() {
            assert_eq!(e.left, 1);
            set.insert(e.right);
        }
        assert_eq!(
            set,
            HashSet::from_iter((0..=40 as usize).filter(|i| *i != 29).map(|i| i * 10))
        );

        m2m.free_reserved_entry(29);
        assert_eq!(m2m.put(1, 410), (true, 29)); // 29 from free list
    }

    #[test]
    fn should_return_empty_mapping_when_unknown_key() {
        let m2m = CompactManyToMany::new();
        assert_eq!(m2m.with_left(4).count(), 0);
        assert_eq!(m2m.with_right(4).count(), 0);
        assert_eq!(m2m.with_left(100).iter().count(), 0);
        assert_eq!(m2m.with_right(100).iter().count(), 0);
    }

}
