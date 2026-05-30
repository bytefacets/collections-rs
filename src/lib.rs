// SPDX-FileCopyrightText: Copyright (c) 2026 Byte Facets
// SPDX-License-Identifier: MIT

mod bi;
mod indexed_set;
mod indexed_map;
mod num_utils;
pub mod pack;
mod store;

pub use indexed_set::IndexedSet;
pub use indexed_map::IndexedMap;

pub(crate) fn ensure_entry<T:Clone + Default>(vec: &mut Vec<T>, entry: usize) {
    ensure_size(vec, entry + 1);
}

pub(crate) fn ensure_size<T:Clone + Default>(vec: &mut Vec<T>, size: usize) {
    if size >= vec.len() {
        vec.resize(grow(size + 1, vec.len()), T::default());
    }
}

fn grow(necessary_size: usize, current_size: usize) -> usize {
    let mut new_size = current_size;
    if new_size == 0 {
        new_size = 1;
    }
    while new_size < necessary_size {
        new_size <<= 1;
    }
    new_size
}
