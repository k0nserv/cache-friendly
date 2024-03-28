use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;
use rand::seq::SliceRandom;

use cache_friendly::make_uniq_seq;

pub fn benchmark_hashmap<const SIZE: usize, const LOOKUPS: usize>(c: &mut Criterion) {
    let mut rng = thread_rng();

    let data: Vec<_> = make_uniq_seq::<(u64, u64), _, _>(SIZE, |(k, _)| *k).collect();
    let lookups: Vec<u64> = (0..LOOKUPS)
        .map(|_| data.choose(&mut rng).map(|(k, _)| *k).unwrap())
        .collect();

    let mut group = c.benchmark_group(format!("{LOOKUPS} Lookups, {SIZE} items"));

    group.bench_function("HashMap<u64, u64>", |b| {
        use std::collections::HashMap;
        let hash_map: HashMap<_, _> = data.iter().cloned().collect();

        b.iter(|| {
            for k in &lookups {
                black_box(hash_map.get(black_box(k)));
            }
        })
    });

    group.bench_function("HashMap<u64, u64> fast hasher", |b| {
        use ahash::RandomState;
        use std::collections::HashMap;
        let hash_map: HashMap<_, _, RandomState> = data.iter().cloned().collect();

        b.iter(|| {
            for k in &lookups {
                black_box(hash_map.get(black_box(k)));
            }
        })
    });

    group.bench_function("BTreeMap<u64, u64>", |b| {
        use std::collections::BTreeMap;
        let hash_map: BTreeMap<_, _> = data.iter().cloned().collect();

        b.iter(|| {
            for k in &lookups {
                black_box(hash_map.get(black_box(k)));
            }
        })
    });

    group.bench_function(format!("[(u64, u64); {SIZE}]"), |b| {
        let mut stack_array = [(0, 0); SIZE];
        for (i, (k, v)) in data.iter().enumerate() {
            stack_array[i] = (*k, *v);
        }

        b.iter(|| {
            for k in &lookups {
                let key = black_box(k);
                black_box(stack_array.iter().find(|(k, _)| k == key));
            }
        })
    });

    group.bench_function(format!("[(u64, u64); {SIZE}] binary search"), |b| {
        let mut stack_array = [(0, 0); SIZE];
        for (i, (k, v)) in data.iter().enumerate() {
            stack_array[i] = (*k, *v);
        }
        stack_array.sort_by_key(|(k, _)| *k);

        b.iter(|| {
            for k in &lookups {
                black_box(stack_array.binary_search_by_key(k, |(k, _)| *k));
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_hashmap::<10, 1000_00>,
    benchmark_hashmap::<20, 1000_00>,
    benchmark_hashmap::<40, 1000_00>,
    benchmark_hashmap::<80, 1000_00>,
    benchmark_hashmap::<100, 1000_00>,
    benchmark_hashmap::<1_000, 1000_00>,
);
criterion_main!(benches);
