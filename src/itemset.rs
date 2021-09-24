use crate::{
    combi::join_step,
    types::{
        FrequentItemsets, Inventory, ItemCounts, Itemset, ItemsetCounts, RawTransaction,
        ReverseLookup, Transaction,
    },
};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

pub fn generate_frequent_itemsets<'items>(
    transactions: &'items [RawTransaction],
    min_support: f32,
    k: usize,
) -> (FrequentItemsets, Inventory<'items>) {
    if k < 1 {
        panic!("k must be at least 1");
    }

    let mut counter: HashMap<usize, ItemsetCounts> = HashMap::new();

    println!("\nCounting itemsets of length 1.");
    // Generate 1-itemset (separated because of possible optimisation opportunity
    // using a simpler hashmapkey type)
    let (counts, inventory, transactions_new) = create_counts(transactions, min_support);
    let counts = convert_to_itemset_counts(counts);
    counter.insert(1, counts);

    let mut nonfrequent: Vec<Itemset> = vec![];

    // Then generate the rest
    for size in 2..=k {
        println!("\nCounting itemsets of length {}.", size);
        let prev_itemset_counts = &counter[&(size - 1_usize)];
        let mut counts = create_counts_from_prev(prev_itemset_counts, size, &nonfrequent);
        update_counts_with_transactions(
            &mut counts,
            &transactions_new,
            min_support,
            size,
            &mut nonfrequent,
        );
        counter.insert(size, counts);
    }

    (counter, inventory)
}

/// includes pruning
fn update_counts_with_transactions(
    candidate_counts: &mut ItemsetCounts,
    transactions: &[Transaction],
    min_support: f32,
    size: usize,
    nonfrequent: &mut Vec<Itemset>,
) {
    let N = transactions.len() as f32;

    println!("Updating counts...");

    transactions
        .iter()
        .filter(|transaction| transaction.len() >= size)
        .for_each(|transaction| {
            for (candidate, count) in candidate_counts.iter_mut() {
                if candidate.iter().all(|item| transaction.contains(item)) {
                    *count += 1;
                }
            }
        });

    println!("Pruning...");

    if size == 2 {
        candidate_counts
            .iter()
            .for_each(|(candidate, &support_count)| {
                if (support_count as f32 / N) < min_support {
                    nonfrequent.push(candidate.to_owned());
                }
            });
        println!("then nonfrequent length: {}", nonfrequent.len());
        candidate_counts.retain(|_, &mut support_count| (support_count as f32 / N) >= min_support);
    } else {
        candidate_counts.retain(|_, &mut support_count| (support_count as f32 / N) >= min_support);
    }
}

/// target k
fn create_counts_from_prev(
    itemset_counts: &ItemsetCounts,
    size: usize,
    nonfrequent: &Vec<Itemset>,
) -> ItemsetCounts {
    // if !itemset_counts.keys().all(|key| key.len() == size - 1) {
    //     panic!("keys of itemset_counts must be size-1");
    // }
    let mut next_itemset_counts: ItemsetCounts = HashMap::new();

    let mut unique_items: HashSet<usize> = HashSet::new();
    for (itemset, _) in itemset_counts.iter() {
        for &item in itemset.iter() {
            unique_items.insert(item);
        }
    }

    println!("enumerating combinations...");
    if size >= 3 {
        println!("start");
        println!("len itemset-count: {}", itemset_counts.len());
        let mut curr: Vec<Itemset> = itemset_counts.keys().cloned().collect();
        let combinations = join_step(&mut curr);
        println!("stop");
        // println!("{:?}", combinations);

        println!("curr: {}", curr.len());
        println!("combis: {}", combinations.len());

        // bottlenec
        println!("checking combinations...");
        let num_combis = 0;

        'combi1: for combi in combinations.into_iter() {
            for nonfreq in nonfrequent.iter() {
                if nonfreq.iter().zip(combi.iter()).all(|(x, y)| x == y) {
                    continue 'combi1;
                }
            }
            // for prev_itemset in itemset_counts.keys() {
            //     if prev_itemset.iter().zip(combi.iter()).all(|(x, y)| x == y) {
            next_itemset_counts.insert(combi.iter().copied().collect(), 0);
            //         num_combis += 1;
            //         continue 'combi1;
            //     }
            // }
        }
        println!("combinations: {}", num_combis);
    } else {
        let combinations = unique_items.iter().combinations(size);
        // bottlenec
        println!("checking combinations...");
        let num_combis = 0;

        'combi: for mut combi in combinations.into_iter() {
            combi.sort_unstable();

            for nonfreq in nonfrequent.iter() {
                if nonfreq.iter().zip(combi.iter()).all(|(x, &y)| x == y) {
                    continue 'combi;
                }
            }
            // for prev_itemset in itemset_counts.keys() {
            //     if prev_itemset.iter().zip(combi.iter()).all(|(x, &y)| x == y) {
            next_itemset_counts.insert(combi.iter().map(|x| **x).collect(), 0);
            //         num_combis += 1;
            //         continue 'combi;
            //     }
            // }
        }
        println!("combinations: {}", num_combis);
    }

    // if !next_itemset_counts.keys().all(|key| key.len() == size) {
    //     panic!("keys of itemset_counts must be size-1");
    // }

    next_itemset_counts
}

fn convert_to_itemset_counts(item_counts: ItemCounts) -> ItemsetCounts {
    let mut new_itemset_counts = HashMap::new();
    item_counts.into_iter().for_each(|(k, v)| {
        new_itemset_counts.insert(vec![k], v);
    });
    new_itemset_counts
}

/// 1-itemset
/// space: O(2n)
fn create_counts<'items>(
    raw_transactions: &'items [RawTransaction],
    min_support: f32,
) -> (ItemCounts, Inventory<'items>, Vec<Transaction>) {
    let N = raw_transactions.len() as f32;

    let mut reverse_lookup: ReverseLookup = HashMap::new();
    let mut inventory: Inventory = HashMap::new();
    let mut last_item_id = 0;

    // update counts
    let mut one_itemset_counts = HashMap::new();
    let transactions_new: Vec<Transaction> = raw_transactions
        .iter()
        .map(|raw_transaction| {
            let mut newset = Vec::new();

            for &item in raw_transaction {
                let item_id: usize;

                if reverse_lookup.contains_key(item) {
                    item_id = *reverse_lookup.get(&item).unwrap();
                    newset.push(item_id);
                } else {
                    item_id = last_item_id;
                    reverse_lookup.insert(item, item_id);
                    inventory.insert(item_id, item);
                    newset.push(item_id);
                    last_item_id += 1;
                }

                let count = one_itemset_counts.entry(item_id).or_insert(0);
                *count += 1;
            }

            newset.sort_unstable();

            newset
        })
        .collect();

    // Prune
    one_itemset_counts.retain(|_, &mut support_count| (support_count as f32 / N) >= min_support);

    (one_itemset_counts, inventory, transactions_new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

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

    // #[test]
    // fn create_counts_one_itemset_with_sorted_transaction_ids() {
    //     let transactions = vec![raw_transaction!["10", "11", "13"], raw_transaction!["10"]];
    //     let (itemset_counts, inventory, transaction_ids) = create_counts(&transactions, 0.0);

    //     assert_eq!(itemset_counts.len(), 3);
    //     assert_eq!(itemset_counts[&inventory["10"]], 2);
    //     assert_eq!(itemset_counts[&inventory["11"]], 1);
    //     assert_eq!(itemset_counts[&inventory["13"]], 1);

    //     assert_eq!(
    //         transaction_ids,
    //         vec![
    //             vec![inventory["10"], inventory["11"], inventory["13"]]
    //                 .iter()
    //                 .copied()
    //                 .sorted()
    //                 .collect(),
    //             vec![inventory["10"]]
    //         ]
    //     );
    // }

    // #[test]
    // fn create_counts_one_itemset_with_min_support_1() {
    //     let transactions = vec![raw_transaction!["10", "11", "13"], raw_transaction!["10"]];
    //     let (itemset_counts, inventory, _) = create_counts(&transactions, 1.0);

    //     assert_eq!(itemset_counts.len(), 1);
    //     assert_eq!(itemset_counts[&inventory["10"]], 2);
    // }

    // #[test]
    // fn create_counts_one_itemset_with_min_support_05() {
    //     let transactions = vec![
    //         raw_transaction!["10", "11", "12"],
    //         raw_transaction!["10"],
    //         raw_transaction!["11"],
    //         raw_transaction!["10", "12"],
    //     ];
    //     let (itemset_counts, inventory, _) = create_counts(&transactions, 0.5);

    //     assert_eq!(itemset_counts.len(), 3);
    //     assert_eq!(itemset_counts[&inventory["10"]], 3);
    //     assert_eq!(itemset_counts[&inventory["11"]], 2);
    //     assert_eq!(itemset_counts[&inventory["12"]], 2);
    // }

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
        let candidate_counts = create_counts_from_prev(&itemset_counts, 2, &vec![]);

        let expected = hashmap! {
            vec![10, 13] => 0,
            vec![10, 14] => 0,
            vec![13, 14] => 0,
        };

        assert_eq!(candidate_counts, expected);
    }

    #[test]
    fn create_counts_from_prev_2_itemset() {
        let itemset_counts = hashmap! {
            vec![10, 11] => 0,
            vec![13, 14] => 0,
        };
        let candidate_counts = create_counts_from_prev(&itemset_counts, 3, &vec![]);

        let expected = hashmap! {};

        assert_eq!(candidate_counts, expected);
    }

    // #[test]
    // fn create_counts_from_prev_2_itemset_second_example() {
    //     let itemset_counts = hashmap! {
    //         itemset![10, 11] => 1,
    //         itemset![11, 13] => 2,
    //         itemset![13, 14] => 1,
    //     };
    //     // [10 11 13]: 1
    //     // [11 13 14]: 1
    //     //
    //     // [10 11]: 1
    //     // [11 13]: 2
    //     // [13 14]: 1
    //     let candidate_counts = create_counts_from_prev(&itemset_counts, 3, &vec![]);

    //     let expected = hashmap! {
    //         itemset![10, 11, 13] => 0,
    //         itemset![11, 13, 14] => 0,
    //     };

    //     assert_eq!(candidate_counts, expected);
    // }

    #[test]
    fn test_generate_frequent_itemsets_001_minsupport() {
        let transactions = vec![
            hashset!["10", "11"],
            hashset!["10", "12"],
            hashset!["10", "11", "12"],
            hashset!["11", "13"],
        ];
        let (frequent_itemsets, _) = generate_frequent_itemsets(&transactions, 0.01, 3);

        let expected = hashmap! {
            1 => hashmap! {
                vec![0] => 3,
                vec![1] => 3,
                vec![2] => 2,
                vec![3] => 1,
            },
            2 => hashmap! {
                vec![0, 1] => 2,
                vec![0, 2] => 2,
                vec![1, 2] => 1,
                vec![1, 3] => 1,
            },
            3 => hashmap! {
                vec![0, 1, 2] => 1,
            },
        };

        assert_eq!(frequent_itemsets, expected);
    }

    #[test]
    fn test_generate_frequent_itemsets_05_minsupport() {
        let transactions = vec![
            hashset!["10", "11"],
            hashset!["10", "12"],
            hashset!["10", "11", "12"],
            hashset!["11", "13"],
        ];
        let (frequent_itemsets, _) = generate_frequent_itemsets(&transactions, 0.5, 3);

        let expected = hashmap! {
            1 => hashmap! {
                vec![0] => 3,
                vec![1] => 3,
                vec![2] => 2,
            },
            2 => hashmap! {
                vec![0, 1] => 2,
                vec![0, 2] => 2,
            },
            3 => hashmap! {},
        };

        assert_eq!(frequent_itemsets, expected);
    }

    #[test]
    fn test_generate_frequent_itemsets_05_minsupport_large_k() {
        let transactions: Vec<RawTransaction> = vec![
            hashset!["10", "11"],
            hashset!["10", "12"],
            hashset!["10", "11", "12"],
            hashset!["11", "13"],
        ];
        let (frequent_itemsets, _) = generate_frequent_itemsets(&transactions, 0.5, 5);

        let expected = hashmap! {
            1 => hashmap! {
                vec![0] => 3,
                vec![1] => 3,
                vec![2] => 2,
            },
            2 => hashmap! {
                vec![0, 1] => 2,
                vec![0, 2] => 2,
            },
            3 => hashmap! {},
            4 => hashmap! {},
            5 => hashmap! {},
        };

        assert_eq!(frequent_itemsets, expected);
    }
}
