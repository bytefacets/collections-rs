// SPDX-FileCopyrightText: Copyright (c) 2026 Byte Facets
// SPDX-License-Identifier: MIT

use bytefacets_collections::IndexedMap;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rustc_hash::FxHasher;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_insert");
    group.bench_function("indexed_map_insert_1000", |b| {
        b.iter_batched(
            || {
                IndexedMap::with_capacity(2000, 0.75)
            },
            |mut map| {
                for i in 0..1000 {
                    map.insert(i, i);
                }
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("hash_map_insert_1000", |b| {
        b.iter_batched(
            || {
                // note that using the default hasher (RandomState) is significantly worse performing
                HashMap::with_capacity_and_hasher(
                    2000,
                    BuildHasherDefault::<FxHasher>::default(),
                )
            },
            |mut map| {
                for i in 0..1000 {
                    map.insert(i, i);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_remove");
    group.bench_function("indexed_map_remove_1000", |b| {
        b.iter_batched(
            || {
                let mut map = IndexedMap::with_capacity(2000, 0.75);
                for i in 0..1000 {
                    map.insert(i, i);
                }
                map
            },
            |mut map| {
                for i in 0..1000 {
                    map.remove(&i);
                }
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("hash_map_remove_1000", |b| {
        b.iter_batched(
            || {
                let mut map = HashMap::with_capacity_and_hasher(
                    2000,
                    BuildHasherDefault::<FxHasher>::default(),
                );
                for i in 0..1000 {
                    map.insert(i, i);
                }
                map
            },
            |mut map| {
                for i in 0..1000 {
                    map.remove(&i);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_grow(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_grow");
    group.bench_function("indexed_map_grow_1000", |b| {
        b.iter_batched(
            || {
                IndexedMap::with_capacity(16, 0.75)
            },
            |mut map| {
                for i in 0..1000 {
                    map.insert(i, i);
                }
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("hash_map_grow_1000", |b| {
        b.iter_batched(
            || {
                HashMap::with_capacity_and_hasher(
                    16,
                    BuildHasherDefault::<FxHasher>::default(),
                )
            },
            |mut map| {
                for i in 0..1000 {
                    map.insert(i, i);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_insert, bench_remove, bench_grow);
criterion_main!(benches);
