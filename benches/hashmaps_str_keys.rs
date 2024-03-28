use std::mem::MaybeUninit;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::distributions::Standard;
use rand::prelude::*;
use rand::seq::SliceRandom;

use cache_friendly::{make_uniq_seq, random_id};

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
#[repr(transparent)]
struct Id(String);

impl Distribution<Id> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Id {
        Id(random_id(20, rng))
    }
}

pub fn benchmark_hashmap<const SIZE: usize, const LOOKUPS: usize>(c: &mut Criterion) {
    let mut rng = thread_rng();

    let data: Vec<_> = make_uniq_seq::<(Id, u64), _, _>(SIZE, |(k, _)| k.clone()).collect();
    let lookups: Vec<_> = (0..LOOKUPS)
        .map(|_| data.choose(&mut rng).map(|(k, _)| k.clone()).unwrap())
        .collect();

    let mut group = c.benchmark_group(format!("{LOOKUPS} Lookups, {SIZE} items"));

    group.bench_function("HashMap<Id, u64>", |b| {
        use std::collections::HashMap;
        let hash_map: HashMap<_, _> = data.iter().cloned().collect();

        b.iter(|| {
            for k in &lookups {
                black_box(hash_map.get(black_box(k)));
            }
        })
    });

    group.bench_function("HashMap<Id, u64> fast hasher", |b| {
        use ahash::RandomState;
        use std::collections::HashMap;
        let hash_map: HashMap<_, _, RandomState> = data.iter().cloned().collect();

        b.iter(|| {
            for k in &lookups {
                black_box(hash_map.get(black_box(k)));
            }
        })
    });

    group.bench_function("BTreeMap<Id, u64>", |b| {
        use std::collections::BTreeMap;
        let hash_map: BTreeMap<_, _> = data.iter().cloned().collect();

        b.iter(|| {
            for k in &lookups {
                black_box(hash_map.get(black_box(k)));
            }
        })
    });

    group.bench_function(format!("[(Id, u64); {SIZE}]"), |b| {
        let mut stack_array: [(MaybeUninit<Id>, u64); SIZE] =
            [0; SIZE].map(|_| (MaybeUninit::uninit(), 0));

        for (i, (k, v)) in data.iter().enumerate() {
            stack_array[i].0.write(k.clone());
            stack_array[i].1 = *v;
        }

        b.iter(|| {
            for k in &lookups {
                let key = black_box(k);
                black_box(
                    stack_array
                        .iter()
                        .find(|(k, _)| unsafe { &*k.as_ptr() as &Id } == key),
                );
            }
        })
    });

    group.bench_function(format!("[(u64, u64); {SIZE}] binary search"), |b| {
        let mut stack_array: [(MaybeUninit<Id>, u64); SIZE] =
            [0; SIZE].map(|_| (MaybeUninit::uninit(), 0));

        for (i, (k, v)) in data.iter().enumerate() {
            stack_array[i].0.write(k.clone());
            stack_array[i].1 = *v;
        }
        stack_array.sort_by_key(|(k, _)| unsafe { &*k.as_ptr() as &Id });

        b.iter(|| {
            for k in &lookups {
                black_box(
                    stack_array
                        .binary_search_by_key(k, |(k, _)| unsafe { &*k.as_ptr() as &Id }.clone()),
                );
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
