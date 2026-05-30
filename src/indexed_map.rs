// SPDX-FileCopyrightText: Copyright (c) 2026 Byte Facets
// SPDX-License-Identifier: MIT

use crate::indexed_set::IndexedSet;
use crate::num_utils::next_power_of_2;
use std::default::Default;
use std::fmt;
use std::hash::Hash;

pub struct IndexedMap<K, V> {
    /// The underlying indexed set for key management
    keys: IndexedSet<K>,
    /// Parallel vector storing values corresponding to keys
    values: Vec<V>,
}

impl<K: Default + Clone + PartialEq + Hash, V: Default + Clone> IndexedMap<K, V> {
    /// Creates a new `IndexedMap` with default capacity
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `IndexedMap` with the given initial capacity and load factor
    pub fn with_capacity(initial_capacity: usize, load_factor: f64) -> Self {
        let capacity = next_power_of_2((initial_capacity.max(2) as u32).try_into().unwrap());
        Self {
            keys: IndexedSet::with_capacity(initial_capacity, load_factor),
            values: Vec::with_capacity(capacity),
        }
    }

    /// Inserts a key-value pair into the map and returns a tuple with whether the key was
    /// added (true), or already in the set (false), and the entry where they key was stored.
    /// The returned value is a contract that the key will be addressable in `get_key_at`
    /// or `remove_at` or `put_at`
    /// Returns a tuple indicating:
    /// - Whether the key was newly inserted (true) or already existed (false)
    /// - The entry index in the map
    pub fn insert(&mut self, key: K, value: V) -> (bool, usize) {
        let (was_inserted, entry) = self.keys.insert(key);

        // Ensure values vector is large enough
        if entry >= self.values.len() {
            self.values.resize(self.keys.capacity(), V::default());
        }

        // Update or insert the value
        self.values[entry] = value;

        (was_inserted, entry)
    }

    /// Returns the entry for the given key or None if the key is not found.
    pub fn lookup_entry(&self, key: &K) -> Option<usize> {
        self.keys.lookup_entry(key)
    }

    /// Replaces the value at the given entry and returns the old one
    pub fn put_at(&mut self, entry: usize, value: V) -> V {
        std::mem::replace(&mut self.values[entry], value)
    }

    /// Retrieves a reference to the value associated with the given key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.keys.lookup_entry(key).map(|entry| &self.values[entry])
    }

    /// Retrieves a mutable reference to the value associated with the given key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.keys
            .lookup_entry(key)
            .map(|entry| &mut self.values[entry])
    }

    // Retrieves the key at the given entry
    pub fn get_key_at(&self, entry: usize) -> &K {
        self.keys.get_key_at(entry)
    }

    /// Retrieves a reference to the value stored at the given entry
    pub fn get_value_at(&self, entry: usize) -> &V {
        &self.values[entry]
    }

    /// Retrieves a mutable reference to the value stored at the given entry
    pub fn get_value_at_mut(&mut self, entry: usize) -> &mut V {
        &mut self.values[entry]
    }

    /// Retrieves the key, value pair at the given entry
    pub fn get_at(&self, entry: usize) -> (&K, &V) {
        (self.keys.get_key_at(entry), &self.values[entry])
    }

    /// Retrieves the key, value pair at the given entry, but the value is mutable
    pub fn get_at_mut(&mut self, entry: usize) -> (&K, &mut V) {
        (self.keys.get_key_at(entry), &mut self.values[entry])
    }

    /// Removes a key-value pair from the map, returning the entry if it was found
    pub fn remove(&mut self, key: &K) -> Option<usize> {
        let result = self.keys.remove(key);
        if let Some(entry) = result {
            self.values[entry] = V::default(); // Clear the value
        }
        result
    }

    /// Returns the number of key-value pairs in the map
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Checks if the map is empty
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Clears the map, removing all key-value pairs
    pub fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
    }

    /// Returns an iterator over references to the keys in the map
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.keys.iter()
    }

    /// Returns an iterator over references to the values in the map
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.keys.entry_iter().map(|(entry, _)| &self.values[entry])
    }

    /// Returns an iterator over key-value pairs in the map
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys
            .entry_iter()
            .map(|(entry, key)| (key, &self.values[entry]))
    }
}

impl<K: Default + Clone + PartialEq + Hash, V: Default + Clone> Default for IndexedMap<K, V> {
    fn default() -> Self {
        Self {
            keys: IndexedSet::default(),
            values: Vec::with_capacity(crate::indexed_set::DEFAULT_INITIAL_CAPACITY),
        }
    }
}

// Implement Display for debugging purposes
impl<K: fmt::Display + Default + Clone + PartialEq + Hash, V: fmt::Display> fmt::Display
    for IndexedMap<K, V>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for (entry, key) in self.keys.entry_iter() {
            if !first {
                write!(f, ", ")?;
                first = false;
            }
            write!(f, "{}: {}", key, self.values[entry])?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_basic_operations() {
        let mut map = IndexedMap::new();

        // Insert
        assert_eq!(map.insert(1, "one"), (true, 0));
        assert_eq!(map.insert(2, "two"), (true, 1));
        assert_eq!(map.insert(1, "updated"), (false, 0));

        // Get
        assert_eq!(map.get(&1), Some(&"updated"));
        assert_eq!(map.get(&2), Some(&"two"));
        assert_eq!(map.get(&3), None);

        // Get-At
        assert_eq!(map.get_key_at(1), &2);
        assert_eq!(map.get_value_at(1), &"two");
        assert_eq!(map.get_at(1), (&2, &"two"));

        // Mut
        let value = map.get_mut(&1).unwrap();
        *value = "updated-mut";
        assert_eq!(map.get(&1), Some(&"updated-mut"));

        // Put-At
        assert_eq!(map.put_at(0, "updated-put-at"), "updated-mut");
        assert_eq!(map.get(&1), Some(&"updated-put-at"));

        // Len and is_empty
        assert_eq!(map.len(), 2);
        assert!(!map.is_empty());

        // Remove
        assert_eq!(map.remove(&1), Some(0));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&1), None);

        // Clear
        map.clear();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn should_grow() {
        let mut map = IndexedMap::new();
        for i in 0..64 {
            map.insert(i, i * 10);
        }
        assert_eq!(map.get(&25), Some(&250));
        assert_eq!(map.get_value_at(33), &330);
    }

    #[test]
    fn test_iteration() {
        let mut map = IndexedMap::new();

        for i in 30..70 {
            map.insert(i, i * 3);
        }

        // Keys iteration
        let observed: HashSet<i32> = HashSet::from_iter(map.keys().cloned());
        let expected: HashSet<i32> = HashSet::from_iter((30..70).collect::<Vec<i32>>());
        assert_eq!(observed, expected);

        // Values iteration
        let observed: HashSet<i32> = HashSet::from_iter(map.values().cloned());
        let expected: HashSet<i32> =
            HashSet::from_iter((30..70).map(|i| i * 3).collect::<Vec<i32>>());
        assert_eq!(observed, expected);
    }
}
