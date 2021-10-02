#![allow(non_snake_case)]
use crate::{
    combi::join_step,
    types::{
        FrequentItemsets, Inventory, ItemCounts, ItemId, Itemset, ItemsetCounts, RawTransaction,
        ReverseLookup, Transaction,
    },
};
use itertools::Itertools;
use pyo3::prelude::pyfunction;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

/// Generate frequent itemsets from a list of transactions.
pub fn generate_frequent_itemsets(
    raw_transactions: Vec<RawTransaction>,
    min_support: f32,
    k: usize,
) -> (FrequentItemsets, Inventory) {
    let mut all_frequent_itemsets: FrequentItemsets = HashMap::with_capacity(k);
    let mut nonfrequent: Vec<Itemset> = Vec::with_capacity(1024); // arbitrary
    let N = raw_transactions.len() as f32;
    let min_support_count = (min_support * N).ceil() as usize;

    // 1-itemset
    let (item_counts, inventory, mut transactions_new) =
        generate_frequent_item_counts(raw_transactions, min_support);

    // 2-itemset
    if k == 1 {
        let counts: ItemsetCounts = convert_to_itemset_counts(item_counts);
        all_frequent_itemsets.insert(1, counts);
    } else {
        transactions_new.retain(|transaction| transaction.len() >= 2);
        let candidates = item_counts.keys().combinations(2);
        let two_itemset_counts: HashMap<Itemset, u32> = candidates
            .par_bridge()
            .into_par_iter()
            .filter_map(|candidate| {
                let candidate_count = transactions_new
                    .par_iter()
                    .filter(|transaction| candidate.iter().all(|item| transaction.contains(item)))
                    .count();
                if candidate_count >= min_support_count {
                    let mut freq: Itemset = candidate.iter().map(|x| **x).collect();
                    freq.sort_unstable();
                    Some((freq, candidate_count as u32))
                } else {
                    None
                }
            })
            .collect();

        // insert
        let counts: ItemsetCounts = convert_to_itemset_counts(item_counts);
        all_frequent_itemsets.insert(1, counts);
        all_frequent_itemsets.insert(2, two_itemset_counts);
    }

    // k-itemset, k>1
    for size in 3..=k {

        transactions_new.retain(|transaction| transaction.len() >= size);

        let prev_frequent_itemsets = &all_frequent_itemsets[&(size - 1_usize)];
        let candidates: ItemsetCounts =
            generate_candidates_from_prev(prev_frequent_itemsets, size, &nonfrequent);

        let frequent_itemsets = update_counts_with_transactions(
            &candidates,
            &transactions_new,
            min_support,
            min_support_count,
            size,
            &mut nonfrequent,
        );
        all_frequent_itemsets.insert(size, frequent_itemsets);
    }

    (all_frequent_itemsets, inventory)
}

/// includes pruning
fn update_counts_with_transactions(
    candidate_counts: &ItemsetCounts,
    transactions: &[Transaction],
    min_support: f32,
    min_support_count: usize,
    size: usize,
    nonfrequent: &mut Vec<Itemset>,
) -> HashMap<Itemset, u32> {
    // let N = transactions.len() as f32;
    // let min_support_count = (min_support * N).ceil() as usize;

    let candidates = candidate_counts.keys().par_bridge();
    let next_item_counts: HashMap<Itemset, u32> = candidates
        .into_par_iter()
        .filter_map(|candidate| {
            let candidate_count = transactions
                .par_iter()
                .filter(|transaction| candidate.iter().all(|item| transaction.contains(item)))
                .count();
            if candidate_count >= min_support_count {
                Some((candidate.iter().copied().collect(), candidate_count as u32))
            } else {
                None
            }
        })
        .collect();

    next_item_counts
}

/// target k
fn generate_candidates_from_prev(
    prev_frequent_itemsets: &ItemsetCounts,
    target_size: usize,
    nonfrequent: &[Itemset],
) -> ItemsetCounts {
    let mut next_itemset_counts: ItemsetCounts =
        HashMap::with_capacity(prev_frequent_itemsets.len());

    let mut curr: Vec<Itemset> = prev_frequent_itemsets.keys().cloned().collect();
    let combinations = join_step(&mut curr);

    for combi in combinations.into_iter() {
        next_itemset_counts.insert(combi.iter().copied().collect(), 0);
    }

    next_itemset_counts
}

fn convert_to_itemset_counts(item_counts: ItemCounts) -> ItemsetCounts {
    item_counts.into_iter().map(|(k, v)| (vec![k], v)).collect()
}

/// 1-itemset
/// space: O(2n)
#[pyfunction]
pub fn generate_frequent_item_counts(
    raw_transactions: Vec<HashSet<&str>>,
    min_support: f32,
) -> (ItemCounts, Inventory, Vec<Transaction>) {
    let N = raw_transactions.len() as f32;
    let approx_num_unique_items = 1024; // arbitrary
    let approx_num_items_in_transaction = 16;
    let mut reverse_lookup: ReverseLookup = HashMap::with_capacity(approx_num_unique_items);
    let mut inventory: Inventory = HashMap::with_capacity(approx_num_unique_items);
    let mut last_item_id = 0;
    let mut item_counts = HashMap::with_capacity(approx_num_unique_items);

    let mut items = Vec::with_capacity(approx_num_items_in_transaction);
    let min_support_count = (min_support * N).ceil() as u32;

    // Update counts
    let transactions_new: Vec<Transaction> = raw_transactions
        .iter()
        .map(|raw_transaction| {
            items.clear();

            for &item in raw_transaction {
                let item_id: usize;

                if reverse_lookup.contains_key(item) {
                    item_id = *reverse_lookup.get(&item).unwrap();
                    items.push(item_id);
                } else {
                    item_id = last_item_id;
                    reverse_lookup.insert(item, item_id);
                    inventory.insert(item_id, item);
                    items.push(item_id);
                    last_item_id += 1;
                }

                let count = item_counts.entry(item_id).or_insert(0);
                *count += 1;
            }

            items.sort_unstable();

            items.to_owned()
        })
        .collect();

    // Prune
    item_counts.retain(|_, &mut support_count| support_count >= min_support_count);

    (item_counts, inventory, transactions_new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    const A: &str = "Item A";
    const B: &str = "Item B";
    const C: &str = "Item C";
    const D: &str = "Item D";

    macro_rules! raw_transaction {
        ($($x:expr),*) => {
            {
                let mut set: HashSet<_> = HashSet::new();
                $(set.insert($x);)*
                set
            }
        };
    }

    macro_rules! hashset {
        ($($x:expr),*) => {
            {
                let mut set: HashSet<_> = HashSet::new();
                $(set.insert($x);)*
                set
            }
        };
    }

    macro_rules! sorted_vec {
        ($($x:expr),*) => {
            {
                let mut vec: Itemset = Vec::with_capacity(5);
                $(vec.push($x);)*
                vec.sort_unstable();
                vec
            }
        };
    }

    #[test]
    fn update_counts() {
        let transactions = vec![vec![0, 1]];
        let mut candidate_counts = hashmap! {
            vec![0] => 0,
            vec![1] => 0,
        };
        let mut nonfrequent = vec![];

        update_counts_with_transactions(
            &mut candidate_counts,
            &transactions,
            0.0,
            1,
            &mut nonfrequent,
        );

        assert_eq!(candidate_counts, hashmap! { vec![0] => 1, vec![1] => 1 });
    }

    #[test]
    fn update_counts_with_min_support_1() {
        let transactions = vec![vec![10, 11], vec![10, 12]];
        let mut candidate_counts = hashmap! {
            vec![10] => 0,
            vec![11] => 0,
        };
        let mut nonfrequent = vec![];

        update_counts_with_transactions(
            &mut candidate_counts,
            &transactions,
            1.0,
            1,
            &mut nonfrequent,
        );

        assert_eq!(candidate_counts, hashmap! {vec![10] => 2})
    }

    #[test]
    fn update_counts_with_min_support_0_5_1_itemset() {
        let transactions = vec![
            vec![10, 11],
            vec![10, 15],
            vec![10, 12],
            vec![10, 12],
            vec![10, 12],
            vec![11, 12],
        ];
        let mut candidate_counts = hashmap! {
            vec![10] => 0,
            vec![11] => 0,
            vec![12] => 0,
            vec![15] => 0,
        };
        let mut nonfrequent = vec![];

        update_counts_with_transactions(
            &mut candidate_counts,
            &transactions,
            0.5,
            1,
            &mut nonfrequent,
        );

        assert_eq!(
            candidate_counts,
            hashmap! {
            vec![10] => 5,
            vec![12] => 4,
            }
        );
    }

    #[test]
    fn update_counts_with_min_support_0_5_2_itemset() {
        let transactions = vec![
            vec![10, 11],
            vec![10, 15],
            vec![10, 13],
            vec![10, 13],
            vec![10, 13],
            vec![11, 13],
        ];
        let mut candidate_counts = hashmap! {
            vec![10, 11] => 0,
            vec![10, 13] => 0,
            vec![10, 15] => 0,
            vec![11, 13] => 0,
            vec![11, 15] => 0,
        };
        let mut nonfrequent = vec![];

        update_counts_with_transactions(
            &mut candidate_counts,
            &transactions,
            0.5,
            2,
            &mut nonfrequent,
        );

        assert_eq!(candidate_counts, hashmap! { vec![10, 13] => 3});
    }

    #[test]
    fn update_counts_with_min_support() {
        let transactions = vec![vec![10, 11], vec![10, 13]];
        let mut candidate_counts = hashmap! {
            vec![10] => 0,
            vec![11] => 0,
        };
        let mut nonfrequent = vec![];

        update_counts_with_transactions(
            &mut candidate_counts,
            &transactions,
            1.0,
            1,
            &mut nonfrequent,
        );

        assert_eq!(candidate_counts, hashmap! { vec![10] => 2 });
    }

    #[test]
    fn update_counts_2() {
        let transactions = vec![vec![10, 11, 13]];
        let mut candidate_counts = hashmap! {
            vec![10] => 0,
            vec![11] => 0,
        };
        let mut nonfrequent = vec![];

        update_counts_with_transactions(
            &mut candidate_counts,
            &transactions,
            0.0,
            1,
            &mut nonfrequent,
        );
        assert_eq!(
            candidate_counts,
            hashmap! { vec![10] => 1,
            vec![11] => 1}
        );
    }

    #[test]
    fn update_counts_3() {
        let transactions = vec![vec![10, 11, 13], vec![10]];
        let mut candidate_counts = hashmap! {
            vec![10] => 0,
            vec![11] => 0,
        };
        let mut nonfrequent = vec![];

        update_counts_with_transactions(
            &mut candidate_counts,
            &transactions,
            0.0,
            1,
            &mut nonfrequent,
        );
        assert_eq!(
            candidate_counts,
            hashmap! { vec![10] => 2,
            vec![11] => 1}
        );
    }

    #[test]
    fn create_counts_one_itemset_with_sorted_transaction_ids() {
        let raw_transactions = vec![raw_transaction![A, B, D], raw_transaction![A]];
        let (itemset_counts, inventory, transaction_ids) =
            generate_frequent_item_counts(raw_transactions, 0.0);
        let lookup = get_reverse_lookup(inventory);

        assert_eq!(itemset_counts.len(), 3);
        assert_eq!(itemset_counts[&lookup[A]], 2);
        assert_eq!(itemset_counts[&lookup[B]], 1);
        assert_eq!(itemset_counts[&lookup[D]], 1);

        assert_eq!(
            transaction_ids,
            vec![
                sorted_vec![lookup[A], lookup[B], lookup[D]],
                vec![lookup[A]]
            ]
        );
    }

    #[test]
    fn create_counts_one_itemset_with_min_support_1() {
        let raw_transactions = vec![raw_transaction![A, B, D], raw_transaction![A]];
        let (itemset_counts, inventory, _) = generate_frequent_item_counts(raw_transactions, 1.0);
        let lookup = get_reverse_lookup(inventory);

        assert_eq!(itemset_counts.len(), 1);
        assert_eq!(itemset_counts[&lookup[A]], 2);
    }

    #[test]
    fn create_counts_one_itemset_with_min_support_05() {
        let raw_transactions = vec![
            raw_transaction![A, B, C],
            raw_transaction![A],
            raw_transaction![B],
            raw_transaction![A, C],
        ];
        let (itemset_counts, inventory, _) = generate_frequent_item_counts(raw_transactions, 0.5);
        let lookup = get_reverse_lookup(inventory);

        assert_eq!(itemset_counts.len(), 3);
        assert_eq!(itemset_counts[&lookup[A]], 3);
        assert_eq!(itemset_counts[&lookup[B]], 2);
        assert_eq!(itemset_counts[&lookup[C]], 2);
    }

    #[test]
    fn test_convert_to_itemset_counts() {
        let item_counts: ItemCounts = hashmap! {
            13 => 3,
            10 => 0,
            11 => 5,
        };
        let itemset_counts = convert_to_itemset_counts(item_counts);

        let expected = hashmap! {
            vec![10] => 0,
            vec![11] => 5,
            vec![13] => 3,
        };

        assert_eq!(itemset_counts, expected);
    }

    #[test]
    fn create_counts_from_prev_1_itemset() {
        let itemset_counts = hashmap! {
            vec![10] => 0,
            vec![13] => 0,
            vec![14] => 0,
        };
        let candidate_counts = generate_candidates_from_prev(&itemset_counts, 2, &[]);

        let expected = hashmap! {
            vec![10, 13] => 0,
            vec![10, 14] => 0,
            vec![13, 14] => 0,
        };

        assert_eq!(candidate_counts, expected);
    }

    #[test]
    fn test_generate_frequent_itemsets_001_minsupport() {
        let transactions = vec![
            hashset![A, B],
            hashset![A, C],
            hashset![A, B, C],
            hashset![B, D],
        ];
        let (frequent_itemsets, inventory) = generate_frequent_itemsets(transactions, 0.01, 3);
        let lookup = get_reverse_lookup(inventory);

        let expected = hashmap! {
            1 => hashmap! {
                vec![lookup[A]] => 3,
                vec![lookup[B]] => 3,
                vec![lookup[C]] => 2,
                vec![lookup[D]] => 1,
            },
            2 => hashmap! {
                sorted_vec![lookup[A], lookup[B]] => 2,
                sorted_vec![lookup[A], lookup[C]] => 2,
                sorted_vec![lookup[B], lookup[C]] => 1,
                sorted_vec![lookup[B], lookup[D]] => 1,
            },
            3 => hashmap! {
                sorted_vec![0, 1, 2] => 1,
            },
        };

        assert_eq!(frequent_itemsets, expected);
    }

    #[test]
    fn test_generate_frequent_itemsets_05_minsupport() {
        let transactions = vec![
            hashset![A, B],
            hashset![A, C],
            hashset![A, B, C],
            hashset![B, D],
        ];
        let (frequent_itemsets, inventory) = generate_frequent_itemsets(transactions, 0.5, 3);
        let lookup = get_reverse_lookup(inventory);

        let expected = hashmap! {
            1 => hashmap! {
                vec![lookup[A]] => 3,
                vec![lookup[B]] => 3,
                vec![lookup[C]] => 2,
            },
            2 => hashmap! {
                sorted_vec![lookup[A], lookup[B]] => 2,
                sorted_vec![lookup[A], lookup[C]] => 2,
            },
            3 => hashmap! {},
        };

        assert_eq!(frequent_itemsets, expected);
    }

    #[test]
    fn test_generate_frequent_itemsets_05_minsupport_large_k() {
        let transactions: Vec<RawTransaction> = vec![
            hashset![A, B],
            hashset![A, C],
            hashset![A, B, C],
            hashset![B, C],
        ];
        let (frequent_itemsets, inventory) = generate_frequent_itemsets(transactions, 0.5, 5);
        let lookup = get_reverse_lookup(inventory);

        let expected = hashmap! {
            1 => hashmap! {
                vec![lookup[A]] => 3,
                vec![lookup[B]] => 3,
                vec![lookup[C]] => 3,
            },
            2 => hashmap! {
                sorted_vec![lookup[A], lookup[B]] => 2,
                sorted_vec![lookup[A], lookup[C]] => 2,
                sorted_vec![lookup[B], lookup[C]] => 2,
            },
            3 => hashmap! {},
            4 => hashmap! {},
            5 => hashmap! {},
        };

        assert_eq!(frequent_itemsets, expected);
    }

    fn get_reverse_lookup(inventory: Inventory) -> ReverseLookup {
        inventory.into_iter().map(|(k, v)| (v, k)).collect()
    }
}
