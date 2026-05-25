//! Byte Facets Collections Library
//! 
//! A high-performance, indexed collections library for Rust

mod indexed_set;
mod indexed_map;
mod num_utils;
pub mod pack;
mod store;

pub use indexed_set::IndexedSet;
pub use indexed_map::IndexedMap;

// Re-export other modules or types as needed
