use std::collections::HashMap;

use bitvec::prelude::*;
use itertools::Itertools;
use pyo3::prelude::pyfunction;

use crate::types::{ItemId, Itemset};

type BucketId = usize;

const NUM_BUCKETS: usize = 1024;

#[pyfunction]
pub fn pcy(transactions: Vec<Vec<ItemId>>, threshold: usize) -> Vec<Itemset> {
    let mut bucket_counts = get_bucket_counts(&transactions);
    bucket_counts.retain(|_, &mut count| count >= threshold);
    let bit_arr = hashmap_to_bitarr(&bucket_counts, &threshold);
    generate_frequent(bucket_counts, &bit_arr)
}

fn get_bucket_counts(transactions: &[Vec<ItemId>]) -> HashMap<BucketId, usize> {
    let mut counts = HashMap::new();

    for transaction in transactions {
        transaction.iter().combinations(2).for_each(|combi| {
            *counts.entry(f(&combi)).or_insert(0) += 1;
        });
    }

    counts
}

fn f(itemset: &[&ItemId]) -> usize {
    (itemset.iter().fold(0_usize, |a, &&b| a + b)) % NUM_BUCKETS
}

fn hashmap_to_bitarr(
    counts: &HashMap<BucketId, usize>,
    threshold: &usize,
) -> BitArray<Lsb0, [u16; 64]> {
    let mut bit_arr = bitarr![Lsb0, u16; 64; NUM_BUCKETS];

    for (id, mut bit) in bit_arr.iter_mut().enumerate() {
        if counts.contains_key(&id) {
            *bit = counts[&id] > *threshold;
        }
    }

    bit_arr
}

fn generate_frequent(
    counts: HashMap<usize, usize>,
    bit_arr: &BitArray<Lsb0, [u16; 64]>,
) -> Vec<Itemset> {
    counts
        .keys()
        .combinations(2)
        .filter(|combi| bit_arr[f(combi)])
        .map(|combi| combi.iter().map(|&&x| x).collect())
        .collect()
}
