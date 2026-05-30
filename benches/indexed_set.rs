// SPDX-FileCopyrightText: Copyright (c) 2026 Byte Facets
// SPDX-License-Identifier: MIT

use bytefacets_collections:: IndexedSet;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rustc_hash::FxHasher;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_insert");
    group.bench_function("indexed_set_insert_1000", |b| {
        b.iter_batched(
            || {
                IndexedSet::with_capacity(2000, 0.75)
            },
            |mut set| {
                for i in 0..1000 {
                    set.insert(i);
                }
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("hash_set_insert_1000", |b| {
        b.iter_batched(
            || {
                // note that using the default hasher (RandomState) is significantly worse performing
                HashSet::with_capacity_and_hasher(
                    2000,
                    BuildHasherDefault::<FxHasher>::default(),
                )
            },
            |mut set| {
                for i in 0..1000 {
                    set.insert(i);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_remove");
    group.bench_function("indexed_set_remove_1000", |b| {
        b.iter_batched(
            || {
                let mut set = IndexedSet::with_capacity(2000, 0.75);
                for i in 0..1000 {
                    set.insert(i);
                }
                set
            },
            |mut set| {
                for i in 0..1000 {
                    set.remove(&i);
                }
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("hash_set_remove_1000", |b| {
        b.iter_batched(
            || {
                let mut set = HashSet::with_capacity_and_hasher(
                    2000,
                    BuildHasherDefault::<FxHasher>::default(),
                );
                for i in 0..1000 {
                    set.insert(i);
                }
                set
            },
            |mut set| {
                for i in 0..1000 {
                    set.remove(&i);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_grow(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_grow");
    group.bench_function("indexed_set_grow_1000", |b| {
        b.iter_batched(
            || {
                IndexedSet::with_capacity(16, 0.75)
            },
            |mut set| {
                for i in 0..1000 {
                    set.insert(i);
                }
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("hash_map_grow_1000", |b| {
        b.iter_batched(
            || {
                HashSet::with_capacity_and_hasher(
                    16,
                    BuildHasherDefault::<FxHasher>::default(),
                )
            },
            |mut set| {
                for i in 0..1000 {
                    set.insert(i);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_insert, bench_remove, bench_grow);
criterion_main!(benches);
