// SPDX-FileCopyrightText: Copyright (c) 2025 Byte Facets
// SPDX-License-Identifier: MIT

#![allow(dead_code)]

use crate::num_utils;
use std::fmt::Debug;

/// A store for generic values which are stored in array "chunks". Using chunks is
/// especially beneficial for resizing because only new chunks need to be allocated.
pub struct ChunkStore<T> {
    chunk_size: usize,
    chunk_mask: usize,
    shift: u32,
    chunks: Vec<Vec<T>>,
    capacity: usize,
    limit: usize,
}

impl<T: Clone + Default + Debug> ChunkStore<T> {
    pub fn new(initial_size: usize, chunk_size: usize) -> Self {
        assert!(initial_size >= 1, "initial_size must be at least 1");
        assert!(chunk_size >= 2, "chunk_size must be at least 2");
        let chunk_size = num_utils::next_power_of_2(chunk_size);
        let chunk_mask = chunk_size - 1;
        let shift = chunk_size.trailing_zeros();
        let required_chunks = (initial_size as f64 / chunk_size as f64).ceil() as usize;
        let chunks = vec![vec![T::default(); chunk_size]; required_chunks];
        let capacity = chunks.len() * chunk_size;
        Self {
            chunk_size,
            chunk_mask,
            shift,
            chunks,
            capacity,
            limit: 0,
        }
    }

    /// Returns the value at the given index or the type's default if beyond the capacity.
    pub fn get(&self, index: usize) -> T {
        if index >= self.capacity {
            return T::default();
        }
        let offset = index & self.chunk_mask;
        let chunk = index >> self.shift;
        self.chunks[chunk][offset].clone()
    }

    /// Sets the value at the given index, and grows the store to accommodate the index
    /// if necessary.
    pub fn set(&mut self, index: usize, value: T) {
        if index >= self.capacity {
            self.grow(index + 1);
        }
        self.limit = self.limit.max(index);
        let offset = index & self.chunk_mask;
        let chunk = index >> self.shift;
        self.chunks[chunk][offset] = value;
    }

    fn grow(&mut self, index: usize) {
        let required_chunks = (index as f64 / self.chunk_size as f64).ceil() as usize;
        let old_len = self.chunks.len();
        self.chunks.resize(required_chunks, vec![]);
        for i in old_len..self.chunks.len() {
            self.chunks[i] = vec![T::default(); self.chunk_size];
        }
        self.capacity = self.chunks.len() * self.chunk_size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let store = ChunkStore::<i32>::new(100, 32);
        assert_eq!(store.chunk_size, 32);
        assert_eq!(store.capacity, 128);
        assert_eq!(store.chunks.len(), 4);
    }

    #[test]
    fn test_get_and_set() {
        let mut store = ChunkStore::<i32>::new(10, 16);
        assert_eq!(store.get(5), 0);
        store.set(5, 123);
        assert_eq!(store.get(5), 123);
        assert_eq!(store.get(100), 0); // Test out of bounds
    }

    #[test]
    fn test_grow() {
        let mut store = ChunkStore::<i32>::new(10, 16);
        assert_eq!(store.capacity, 16);
        store.set(20, 456);
        assert_eq!(store.capacity, 32);
        assert_eq!(store.get(20), 456);
    }
}

/// A matrix store for values which are stored in array "chunks". Using chunks is
/// especially beneficial for resizing because only new chunks need to be allocated.
pub struct ChunkMatrixStore<T> {
    chunk_size: usize,
    chunk_mask: usize,
    shift: u32,
    num_fields: usize,
    chunks: Vec<Vec<T>>,
    capacity: usize,
}

impl<T: Clone + Default> ChunkMatrixStore<T> {
    /// * `initialSize` - initial size of the store in terms of the number of groups of fields;
    ///                   though depending on the chunkSize, you may see your initialSize larger
    ///                   than requested; minimum valid value is 1
    /// * `chunkSize` - the size of the arrays used internally which impact data locality and growth.
    ///                 minimum value is 2, and given values will be rounded up to the next power of 2
    /// * `numFields` - the number of logical fields in the store; minimum valid value is 1
    pub fn new(initial_size: usize, chunk_size: usize, num_fields: usize) -> Self {
        assert!(initial_size >= 1, "initial_size must be at least 1");
        assert!(chunk_size >= 2, "chunk_size must be at least 2");
        assert!(num_fields >= 1, "num_fields must be at least 1");

        let chunk_size = num_utils::next_power_of_2(chunk_size);
        let chunk_mask = chunk_size - 1;
        let shift = chunk_size.trailing_zeros();

        let raw_size = initial_size * num_fields;
        let required_chunks = (raw_size as f64 / chunk_size as f64).ceil() as usize;
        let chunks = vec![vec![T::default(); chunk_size]; required_chunks];
        let capacity = (chunks.len() * chunk_size) / num_fields;

        Self {
            chunk_size,
            chunk_mask,
            shift,
            num_fields,
            chunks,
            capacity,
        }
    }

    /// Returns the value at the given row and field. If the row is beyond the capacity
    /// of the store, it will return the type's default value. The field should be
    /// within the numFields with which the store was instantiated or it will panic.
    pub fn get(&self, row: usize, field: usize) -> T {
        if field >= self.num_fields {
            panic!(
                "Field {} is out of bounds for num_fields {}",
                field, self.num_fields
            );
        }
        if row >= self.capacity {
            return T::default();
        }
        let absolute_ix = (row * self.num_fields) + field;
        let offset = absolute_ix & self.chunk_mask;
        let chunk = absolute_ix >> self.shift;
        self.chunks[chunk][offset].clone()
    }

    /// Sets the value at the given row and field, growing the store if necessary.
    /// The field should be within the numFields with which the store was instantiated or it will panic.
    pub fn set(&mut self, row: usize, field: usize, value: T) {
        if field >= self.num_fields {
            panic!(
                "Field {} is out of bounds for num_fields {}",
                field, self.num_fields
            );
        }
        if row >= self.capacity {
            self.grow(row + 1);
        }
        let absolute_ix = (row * self.num_fields) + field;
        let offset = absolute_ix & self.chunk_mask;
        let chunk = absolute_ix >> self.shift;
        self.chunks[chunk][offset] = value;
    }

    fn grow(&mut self, row: usize) {
        let raw_size = row * self.num_fields;
        let required_chunks = (raw_size as f64 / self.chunk_size as f64).ceil() as usize;
        let old_len = self.chunks.len();
        self.chunks.resize(required_chunks, vec![]);
        for i in old_len..self.chunks.len() {
            self.chunks[i] = vec![T::default(); self.chunk_size];
        }
        self.capacity = (self.chunks.len() * self.chunk_size) / self.num_fields;
    }
}

#[cfg(test)]
mod matrix_tests {
    use super::*;

    #[test]
    fn test_matrix_new() {
        let matrix = ChunkMatrixStore::<i32>::new(10, 32, 3);
        assert_eq!(matrix.chunk_size, 32);
        // raw_size = 10 * 3 = 30. required_chunks = ceil(30/32) = 1.
        // capacity = (1 * 32) / 3 = 10
        assert_eq!(matrix.capacity, 10);
        assert_eq!(matrix.chunks.len(), 1);
    }

    #[test]
    fn test_matrix_get_and_set() {
        let mut matrix = ChunkMatrixStore::<i32>::new(10, 16, 2);
        assert_eq!(matrix.get(5, 1), 0);
        matrix.set(5, 1, 123);
        assert_eq!(matrix.get(5, 1), 123);
        assert_eq!(matrix.get(100, 0), 0); // Test out of bounds row
    }

    #[test]
    #[should_panic(expected = "Field 2 is out of bounds for num_fields 2")]
    fn test_matrix_get_invalid_field() {
        let matrix = ChunkMatrixStore::<i32>::new(10, 16, 2);
        matrix.get(5, 2); // field 2 is out of bounds
    }

    #[test]
    #[should_panic(expected = "Field 2 is out of bounds for num_fields 2")]
    fn test_matrix_set_invalid_field() {
        let mut matrix = ChunkMatrixStore::<i32>::new(10, 16, 2);
        matrix.set(5, 2, 123); // field 2 is out of bounds
    }

    #[test]
    fn test_matrix_grow() {
        let mut matrix = ChunkMatrixStore::<f64>::new(10, 16, 4);
        // raw_size = 10*4 = 40. chunks = ceil(40/16)=3. capacity = (3*16)/4 = 12
        assert_eq!(matrix.capacity, 12);
        matrix.set(15, 3, 99.9);
        // raw_size = 16*4 = 64. chunks = ceil(64/16)=4. capacity = (4*16)/4 = 16
        assert_eq!(matrix.capacity, 16);
        assert_eq!(matrix.get(15, 3), 99.9);
    }
}
