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

pub fn benchmark_hashset<const SIZE: usize, const LOOKUPS: usize>(c: &mut Criterion) {
    let mut rng = thread_rng();

    let data: Vec<_> = make_uniq_seq::<Id, _, _>(SIZE, |k| k.clone()).collect();
    let lookups: Vec<_> = (0..LOOKUPS)
        .map(|_| data.choose(&mut rng).map(|k| k.clone()).unwrap())
        .collect();

    let mut group = c.benchmark_group(format!("{LOOKUPS} Lookups, {SIZE} items"));

    group.bench_function("HashSet<Id>", |b| {
        use std::collections::HashSet;
        let set: HashSet<_> = data.iter().cloned().collect();

        b.iter(|| {
            for k in &lookups {
                black_box(set.get(black_box(k)));
            }
        })
    });

    group.bench_function("HashSet<Id> fast hasher", |b| {
        use ahash::RandomState;
        use std::collections::HashSet;
        let set: HashSet<_, RandomState> = data.iter().cloned().collect();

        b.iter(|| {
            for k in &lookups {
                black_box(set.get(black_box(k)));
            }
        })
    });

    group.bench_function("BTreeSet<Id>", |b| {
        use std::collections::BTreeSet;
        let set: BTreeSet<_> = data.iter().cloned().collect();

        b.iter(|| {
            for k in &lookups {
                black_box(set.get(black_box(k)));
            }
        })
    });

    group.bench_function(format!("[Id; {SIZE}]"), |b| {
        let mut stack_array: [MaybeUninit<Id>; SIZE] = [0; SIZE].map(|_| MaybeUninit::uninit());

        for (i, k) in data.iter().enumerate() {
            stack_array[i].write(k.clone());
        }

        b.iter(|| {
            for k in &lookups {
                let key = black_box(k);
                black_box(
                    stack_array
                        .iter()
                        .find(|k| unsafe { &*k.as_ptr() as &Id } == key),
                );
            }
        })
    });

    group.bench_function(format!("[Id; {SIZE}] binary search"), |b| {
        let mut stack_array: [MaybeUninit<Id>; SIZE] = [0; SIZE].map(|_| MaybeUninit::uninit());

        for (i, k) in data.iter().enumerate() {
            stack_array[i].write(k.clone());
        }
        stack_array.sort_by_key(|k| unsafe { &*k.as_ptr() as &Id });

        b.iter(|| {
            for k in &lookups {
                black_box(
                    stack_array.binary_search_by_key(k, |k| unsafe { &*k.as_ptr() as &Id }.clone()),
                );
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_hashset::<10, 1000_00>,
    benchmark_hashset::<20, 1000_00>,
    benchmark_hashset::<40, 1000_00>,
    benchmark_hashset::<80, 1000_00>,
    benchmark_hashset::<100, 1000_00>,
    benchmark_hashset::<1_000, 1000_00>,
);
criterion_main!(benches);
