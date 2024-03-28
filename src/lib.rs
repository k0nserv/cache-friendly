use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;

use rand::distributions::Standard;
use rand::prelude::*;

pub fn make_seq<T>(count: usize) -> impl Iterator<Item = T>
where
    Standard: Distribution<T>,
{
    (0..count).into_iter().map(|_| random())
}

pub fn make_uniq_seq<T, K, F>(count: usize, extract_key: F) -> impl Iterator<Item = T>
where
    Standard: Distribution<T>,
    F: Fn(&T) -> K,
    K: Hash + Eq,
{
    UniqueGenerator::new(count, extract_key)
}

pub struct UniqueGenerator<T, F, K> {
    count: usize,
    left: usize,
    extract_key: F,
    seen: HashSet<K>,

    phantom: PhantomData<T>,
}

impl<T, F, K> UniqueGenerator<T, F, K>
where
    Standard: Distribution<T>,
    F: Fn(&T) -> K,
    K: Hash + Eq,
{
    fn new(count: usize, extract_key: F) -> Self {
        Self {
            count,
            left: count,
            extract_key,
            seen: HashSet::default(),
            phantom: PhantomData,
        }
    }
}

impl<T, F, K> Iterator for UniqueGenerator<T, F, K>
where
    Standard: Distribution<T>,
    F: Fn(&T) -> K,
    K: Hash + Eq,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.left == 0 {
            return None;
        }
        self.left = self.left.saturating_sub(1);

        let mut x = random();
        let mut k = (self.extract_key)(&x);

        while self.seen.contains(&k) {
            x = random();
            k = (self.extract_key)(&x);
        }

        self.seen.insert(k);

        Some(x)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

// Deliberately not including easily confusable chars, 0, O, l, 1
const ID_SAMPLE: &[u8] = b"abcdefghjkmnpqrstuvxyzABCDEFGHJKLMNPQRSTUVXYZ23456789";

// SAFETY: This array _must_ produce ASCII.
pub fn random_id_array<const SIZE: usize>() -> [u8; SIZE] {
    let mut result = [0; SIZE];

    let mut rng = &mut rand::thread_rng();

    for r in result.iter_mut() {
        *r = *ID_SAMPLE.choose(&mut rng).unwrap();
    }

    result
}

pub fn random_id<R: Rng + ?Sized>(length: usize, rng: &mut R) -> String {
    let mut result = String::with_capacity(length);

    for _ in 0..length {
        // Unwrap is okay because None is only returned when the slice is empty, which it isn't.
        result.push(*ID_SAMPLE.choose(rng).unwrap() as char);
    }

    result
}
