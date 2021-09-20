#![allow(non_snake_case)]
use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict, PyFrozenSet};
use pyo3::wrap_pyfunction;

macro_rules! itemset {
    ($($x:expr),*) => {
        {
            let mut set: Itemset = Vec::new();
            $(set.push($x);)*
            set
        }
    };
}

type ItemId = usize;
type Itemset = Vec<ItemId>;
type ItemsetCounts<'l> = HashMap<Itemset, u32>;
type OneItemsetCounts<'l> = HashMap<ItemId, u32>;
type FrequentItemsets<'l> = HashMap<usize, ItemsetCounts<'l>>;
type TransactionNew = Vec<ItemId>;

type TransactionRaw<'l> = HashSet<&'l str>;

fn main() {
    #[pymodule]
    fn apriori(_: Python, module: &PyModule) -> PyResult<()> {
        module.add_function(wrap_pyfunction!(
            generate_frequent_itemsets_wrapper,
            module
        )?)?;
        Ok(())
    }
}

/// Apriori algorithm for association rules.
///
/// Args:
///     transactions: A list of list of items.
///     min_support: The minimum support.
///     min_confidence: The minimum confidence.
///     max_len: Maximum no. of items in an association rule.
///
/// Returns:
///     A list of association rules.
#[pyfunction]
#[pyo3(text_signature = "(/, *, transactions, min_support, min_confidence, max_len)")]
fn apriori(
    transactions: Vec<HashSet<&str>>,
    min_support: f32,
    min_confidence: f32,
    max_len: usize,
) {
    let N = transactions.len();

    let (itemset_counts, _) = generate_frequent_itemsets(&transactions, min_support, max_len);

    println!("Creating rules");

    let candidates: HashMap<Itemset, u32> = itemset_counts
        .into_iter()
        .flat_map(|(_, itemset_count)| itemset_count)
        .collect();

    for (candidate, count) in &candidates {
        let mut antecedents: HashSet<Vec<usize>> = HashSet::new();
        let mut skipped_ys: HashSet<Vec<usize>> = HashSet::new();

        candidate
            .iter()
            .permutations(candidate.len())
            .for_each(|pattern| {
                for i in (1..pattern.len()).rev() {
                    let rule = pattern.split_at(i);

                    let (x, y) = rule;
                    let mut antecedent: Vec<usize> = x.iter().map(|&&x| x).collect();

                    if x.len() > 1 {
                        antecedent.sort_unstable();
                    }

                    if antecedents.contains(&antecedent) {
                        continue;
                    }

                    let num = *count as f32;
                    let den = *candidates.get(&antecedent).unwrap() as f32;
                    if num / den >= min_confidence {
                        println!("• {:?} -> {:?}", antecedent, y,);
                    } else {
                        skipped_ys.insert(y.iter().map(|&&x| x).collect());
                    }

                    antecedents.insert(antecedent);
                }
            });
    }
}

#[pyfunction]
fn generate_frequent_itemsets_wrapper(
    transactions: Vec<TransactionRaw>,
    min_support: f32,
    max_length: usize,
) -> Py<PyDict> {
    let (itemset_counts, _) = generate_frequent_itemsets(&transactions, min_support, max_length);

    Python::with_gil(|py| {
        itemset_counts
            .into_iter()
            .map(|(count, itemset_counts)| {
                let itemsetg: Py<PyDict> = itemset_counts
                    .into_iter()
                    .map(|(itemset, count)| {
                        let yo: Vec<usize> = itemset.into_iter().collect();
                        let set: Py<PyFrozenSet> = PyFrozenSet::new(py, &yo).unwrap().into();
                        (set, count)
                    })
                    .collect::<Vec<(Py<PyFrozenSet>, u32)>>()
                    .into_py_dict(py)
                    .into();
                (count, itemsetg)
            })
            .into_py_dict(py)
            .into()
    })
}

fn generate_frequent_itemsets<'items>(
    transactions: &'items [TransactionRaw],
    min_support: f32,
    k: usize,
) -> (FrequentItemsets<'items>, Inventory<'items>) {
    if k < 1 {
        panic!("k must be at least 1");
    }

    let mut counter: HashMap<usize, ItemsetCounts> = HashMap::new();

    println!("\nCounting itemsets of length 1.");
    // Generate 1-itemset (separated because of possible optimisation opportunity
    // using a simpler hashmapkey type)
    let (counts, inventory, transactions_new) = create_counts(transactions, min_support);
    let counts = conversion_course(counts);
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
    transactions: &[TransactionNew],
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
                if candidate.iter().all(|x| transaction.contains(x)) {
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
fn create_counts_from_prev<'items>(
    itemset_counts: &ItemsetCounts<'items>,
    size: usize,
    nonfrequent: &Vec<Itemset>,
) -> ItemsetCounts<'items> {
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
        println!("⭐️ experimental enumerating combinations...");

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
        let mut num_combis = 0;

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
        let mut num_combis = 0;

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

fn conversion_course(one_itemset_counts: OneItemsetCounts) -> ItemsetCounts {
    let mut new_itemset_counts = HashMap::new();
    one_itemset_counts.into_iter().for_each(|(k, v)| {
        new_itemset_counts.insert(itemset![k], v);
    });
    new_itemset_counts
}

type Inventory<'l> = HashMap<&'l str, ItemId>;

// 1-itemset
fn create_counts<'items>(
    transactions: &'items [TransactionRaw],
    min_support: f32,
) -> (
    OneItemsetCounts<'items>,
    Inventory<'items>,
    Vec<TransactionNew>,
) {
    let N = transactions.len() as f32;

    let mut inventory: Inventory = HashMap::new();
    let mut last_item_id = 0;

    // update counts
    let mut one_itemset_counts = HashMap::new();
    let transactions_new: Vec<TransactionNew> = transactions
        .iter()
        .map(|transaction| {
            let mut newset = Vec::new();

            for &item in transaction {
                let item_id: usize;

                if inventory.contains_key(item) {
                    item_id = *inventory.get(&item).unwrap();
                    newset.push(item_id);
                } else {
                    item_id = last_item_id;
                    inventory.insert(item, item_id);
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

    macro_rules! transaction {
        ($($x:expr),*) => {
            {
                let mut vec = vec![];
                $(vec.push($x);)*
                vec.sort_unstable();
                vec
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

    // #[test]
    // fn update_counts() {
    //     let transactions = vec![transaction![0, 1]];
    //     let mut candidate_counts = hashmap! {
    //         itemset![0] => 0,
    //         itemset![1] => 0,
    //     };

    //     update_counts_with_transactions(&mut candidate_counts, &transactions, 0.0, 1);
    //     assert_eq!(candidate_counts.len(), 2);
    //     assert_eq!(candidate_counts[&itemset![0]], 1);
    //     assert_eq!(candidate_counts[&itemset![1]], 1);
    // }

    // #[test]
    // fn update_counts_with_min_support_1() {
    //     let transactions = vec![transaction![10, 11], transaction![10, 12]];
    //     let mut candidate_counts = hashmap! {
    //         itemset![10] => 0,
    //         itemset![11] => 0,
    //     };

    //     update_counts_with_transactions(&mut candidate_counts, &transactions, 1.0, 1);
    //     assert_eq!(candidate_counts.len(), 1);
    //     assert_eq!(candidate_counts[&itemset![10]], 2);
    // }

    // #[test]
    // fn update_counts_with_min_support_0_5_1_itemset() {
    //     let transactions = vec![
    //         transaction![10, 11],
    //         transaction![10, 15],
    //         transaction![10, 12],
    //         transaction![10, 12],
    //         transaction![10, 12],
    //         transaction![11, 12],
    //     ];
    //     let mut candidate_counts = hashmap! {
    //         itemset![10] => 0,
    //         itemset![11] => 0,
    //         itemset![12] => 0,
    //         itemset![15] => 0,
    //     };

    //     update_counts_with_transactions(&mut candidate_counts, &transactions, 0.5, 1);
    //     assert_eq!(candidate_counts.len(), 2);
    //     assert_eq!(candidate_counts[&itemset![10]], 5);
    //     assert_eq!(candidate_counts[&itemset![15]], 4);
    // }

    // #[test]
    // fn update_counts_with_min_support_0_5_2_itemset() {
    //     let transactions = vec![
    //         transaction![10, 11],
    //         transaction![10, 15],
    //         transaction![10, 13],
    //         transaction![10, 13],
    //         transaction![10, 13],
    //         transaction![11, 13],
    //     ];
    //     let mut candidate_counts = hashmap! {
    //         itemset![10, 11] => 0,
    //         itemset![10, 13] => 0,
    //         itemset![10, 15] => 0,
    //         itemset![11, 13] => 0,
    //         itemset![11, 15] => 0,
    //     };

    //     update_counts_with_transactions(&mut candidate_counts, &transactions, 0.5, 2);
    //     assert_eq!(candidate_counts.len(), 1);
    //     assert_eq!(candidate_counts[&itemset![10, 13]], 3);
    // }

    // #[test]
    // fn update_counts_with_min_support() {
    //     let transactions = vec![transaction![10, 11], transaction![10, 13]];
    //     let mut candidate_counts = hashmap! {
    //         itemset![10] => 0,
    //         itemset![11] => 0,
    //     };

    //     update_counts_with_transactions(&mut candidate_counts, &transactions, 1.0, 1);
    //     assert_eq!(candidate_counts.len(), 1);
    //     assert_eq!(candidate_counts[&itemset![10]], 2);
    // }

    // #[test]
    // fn update_counts_2() {
    //     let transactions = vec![transaction![10, 11, 13]];
    //     let mut candidate_counts = hashmap! {
    //         itemset![10] => 0,
    //         itemset![11] => 0,
    //     };

    //     update_counts_with_transactions(&mut candidate_counts, &transactions, 0.0, 1);
    //     assert_eq!(candidate_counts.len(), 2);
    //     assert_eq!(candidate_counts[&itemset![10]], 1);
    //     assert_eq!(candidate_counts[&itemset![11]], 1);
    // }

    // #[test]
    // fn update_counts_3() {
    //     let transactions = vec![transaction![10, 11, 13], transaction![10]];
    //     let mut candidate_counts = hashmap! {
    //         itemset![10] => 0,
    //         itemset![11] => 0,
    //     };

    //     update_counts_with_transactions(&mut candidate_counts, &transactions, 0.0, 1);
    //     assert_eq!(candidate_counts.len(), 2);
    //     assert_eq!(candidate_counts[&itemset![10]], 2);
    //     assert_eq!(candidate_counts[&itemset![11]], 1);
    // }

    // #[test]
    // fn create_counts_one_itemset() {
    //     let transactions = vec![transaction!["10", "11", "13"], transaction!["10"]];
    //     let (itemset_counts, b, c) = create_counts(&transactions, 0.0);

    //     println!("{:?}", c);

    //     assert_eq!(itemset_counts.len(), 3);
    //     assert_eq!(itemset_counts[&0], 2);
    //     assert_eq!(itemset_counts[&1], 1);
    //     assert_eq!(itemset_counts[&2], 1);
    // }

    // #[test]
    // fn create_counts_one_itemset_with_min_support_1() {
    //     let transactions = vec![transaction!["10", "11", "13"], transaction!["10"]];
    //     let (itemset_counts, _, _) = create_counts(&transactions, 1.0);

    //     assert_eq!(itemset_counts.len(), 1);
    //     assert_eq!(itemset_counts[&0], 2);
    // }

    // #[test]
    // fn create_counts_one_itemset_with_min_support_05() {
    //     let transactions = vec![
    //         transaction!["10", "11", "12"],
    //         transaction!["10"],
    //         transaction!["11"],
    //         transaction!["10", "12"],
    //     ];
    //     let (itemset_counts, _, _) = create_counts(&transactions, 0.5);

    //     assert_eq!(itemset_counts.len(), 3);
    //     assert_eq!(itemset_counts[&0], 3);
    //     assert_eq!(itemset_counts[&1], 2);
    //     assert_eq!(itemset_counts[&2], 2);
    // }

    // #[test]
    // fn test_conversion_course() {
    //     let one_itemset_counts: OneItemsetCounts = hashmap! {
    //         13 => 3,
    //         10 => 0,
    //         11 => 5,
    //     };
    //     let itemset_counts = conversion_course(one_itemset_counts);

    //     let expected = hashmap! {
    //         itemset![10] => 0,
    //         itemset![11] => 5,
    //         itemset![13] => 3,
    //     };

    //     assert_eq!(itemset_counts, expected);
    // }

    // #[test]
    // fn create_counts_from_prev_1_itemset() {
    //     let itemset_counts = hashmap! {
    //         itemset![10] => 0,
    //         itemset![13] => 0,
    //         itemset![14] => 0,
    //     };
    //     let candidate_counts = create_counts_from_prev(&itemset_counts, 2);

    //     let expected = hashmap! {
    //         itemset![10, 13] => 0,
    //         itemset![10, 14] => 0,
    //         itemset![13, 14] => 0,
    //     };

    //     assert_eq!(candidate_counts, expected);
    // }

    // #[test]
    // fn create_counts_from_prev_2_itemset() {
    //     let itemset_counts = hashmap! {
    //         itemset![10, 11] => 0,
    //         itemset![13, 14] => 0,
    //     };
    //     let candidate_counts = create_counts_from_prev(&itemset_counts, 3);

    //     let expected = hashmap! {
    //         itemset![10, 11, 13] => 0,
    //         itemset![10, 11, 14] => 0,
    //         itemset![10, 13, 14] => 0,
    //         itemset![11, 14, 13] => 0,
    //     };

    //     assert_eq!(candidate_counts, expected);
    // }

    // #[test]
    // fn create_counts_from_prev_2_itemset_second_example() {
    //     let itemset_counts = hashmap! {
    //         itemset![10, 13] => 2,
    //         itemset![11, 13] => 1,
    //         itemset![10, 11] => 2,
    //         itemset![11, 14] => 2,
    //     };
    //     let candidate_counts = create_counts_from_prev(&itemset_counts, 3);

    //     let expected = hashmap! {
    //         itemset![10, 11, 13] => 0,
    //         itemset![10, 11, 14] => 0,
    //         itemset![10, 13, 14] => 0,
    //         itemset![11, 14, 13] => 0,
    //     };

    //     assert_eq!(candidate_counts, expected);
    // }

    // #[test]
    // fn test_generate_frequent_itemsets_001_minsupport() {
    //     let transactions = vec![
    //         transaction!["10", "11"],
    //         transaction!["10", "12"],
    //         transaction!["10", "11", "12"],
    //         transaction!["11", "13"],
    //     ];
    //     let frequent_itemsets = generate_frequent_itemsets(&transactions, 0.01, 3);

    //     let expected = hashmap! {
    //         1 => hashmap! {
    //             itemset![0] => 3,
    //             itemset![1] => 3,
    //             itemset![2] => 2,
    //             itemset![3] => 1,
    //         },
    //         2 => hashmap! {
    //             itemset![0, 1] => 2,
    //             itemset![0, 2] => 2,
    //             itemset![1, 2] => 1,
    //             itemset![1, 3] => 1,
    //         },
    //         3 => hashmap! {
    //             itemset![0, 1, 2] => 1,
    //         },
    //     };

    //     assert_eq!(frequent_itemsets, expected);
    // }

    // #[test]
    // fn test_generate_frequent_itemsets_05_minsupport() {
    //     let transactions = vec![
    //         transaction!["10", "11"],
    //         transaction!["10", "12"],
    //         transaction!["10", "11", "12"],
    //         transaction!["11", "13"],
    //     ];
    //     let frequent_itemsets = generate_frequent_itemsets(&transactions, 0.5, 3);

    //     let expected = hashmap! {
    //         1 => hashmap! {
    //             itemset![0] => 3,
    //             itemset![1] => 3,
    //             itemset![2] => 2,
    //         },
    //         2 => hashmap! {
    //             itemset![0, 1] => 2,
    //             itemset![0, 2] => 2,
    //         },
    //         3 => hashmap! {},
    //     };

    //     assert_eq!(frequent_itemsets, expected);
    // }

    // #[test]
    // fn test_generate_frequent_itemsets_05_minsupport_large_k() {
    //     let transactions = vec![
    //         transaction!["10", "11"],
    //         transaction!["10", "12"],
    //         transaction!["10", "11", "12"],
    //         transaction!["11", "13"],
    //     ];
    //     let frequent_itemsets = generate_frequent_itemsets(&transactions, 0.5, 5);

    //     let expected = hashmap! {
    //         1 => hashmap! {
    //             itemset![0] => 3,
    //             itemset![1] => 3,
    //             itemset![2] => 2,
    //         },
    //         2 => hashmap! {
    //             itemset![0, 1] => 2,
    //             itemset![0, 2] => 2,
    //         },
    //         3 => hashmap! {},
    //         4 => hashmap! {},
    //         5 => hashmap! {},
    //     };

    //     assert_eq!(frequent_itemsets, expected);
    // }

    #[test]
    fn test_get_blacklist() {
        let keys = vec![itemset![11], itemset![12], itemset![20]];
        let keys: &Vec<&Itemset> = &keys.iter().collect();
        let blacklist = vec![itemset![11, 20], itemset![20, 23]];
        let blacklist: &Vec<&Itemset> = &blacklist.iter().collect();
        let h = get_blacklist(keys, blacklist);

        assert_eq!(h.len(), 2);
        assert_eq!(
            h,
            hashmap! {
                itemset![11] => itemset![20],
                itemset![20] => itemset![11, 23],
            }
        );
    }

    #[test]
    fn test_get_combinations() {
        let prevs = vec![itemset![0], itemset![1], itemset![2]];
        let prevs: &Vec<&Itemset> = &prevs.iter().collect();
        let nonfrequent = vec![itemset![0, 2], itemset![2, 3]];
        let nonfrequent: &Vec<&Itemset> = &nonfrequent.iter().collect();
        let currs = vec![
            itemset![0, 1],
            itemset![0, 3],
            itemset![1, 2],
            itemset![1, 3],
        ];
        let currs: &Vec<&Itemset> = &currs.iter().collect();
        let combis = get_combinations(&prevs, &currs, &nonfrequent);
        assert_eq!(combis, hashset![itemset![0, 1, 3]]);
    }

    #[test]
    fn test_join_step() {
        println!("Hello, world!");
        let mut itemsets: Vec<Itemset> = vec![
            vec![1, 2, 3],
            vec![1, 2, 4],
            vec![1, 3, 4],
            vec![1, 3, 5],
            vec![2, 3, 4],
        ];
        let y = join_step(&mut itemsets);
        println!("{:?}", y);
    }
}

/// https://github.com/tommyod/Efficient-Apriori/blob/master/efficient_apriori/itemsets.py
fn join_step(itemsets: &mut [Itemset]) -> Vec<Itemset> {
    let mut final_itemsets: Vec<Itemset> = vec![];

    itemsets.sort_unstable();

    let mut i = 0;
    while i < itemsets.len() {
        let mut skip = 1;

        let (itemset_first, itemset_last) = itemsets[i].split_at(itemsets[i].len() - 1);
        let itemset_last = itemset_last.to_owned().pop().unwrap();

        let mut tail_items: Itemset = vec![itemset_last];

        for j in (i + 1)..itemsets.len() {
            let (itemset_n_first, itemset_n_last) = itemsets[j].split_at(itemsets[j].len() - 1);
            let itemset_n_last = itemset_n_last.to_owned().pop().unwrap();

            if itemset_first == itemset_n_first {
                tail_items.push(itemset_n_last);
                skip += 1;
            } else {
                break;
            }
        }

        for combi in tail_items.iter().combinations(2).sorted() {
            let mut itemset_first_tuple = itemset_first.to_owned();
            let (a, b) = combi.split_at(1);
            let a = *a.to_owned().pop().unwrap();
            let b = *b.to_owned().pop().unwrap();

            itemset_first_tuple.push(a);
            itemset_first_tuple.push(b);
            // itemset_first_tuple.sort_unstable();
            final_itemsets.push(itemset_first_tuple.to_owned());
        }

        i += skip;
    }

    final_itemsets
}

fn get_combinations(
    prev_candidates: &[&Itemset],
    curr_candidates: &[&Itemset],
    nono: &[&Itemset],
) -> HashSet<Itemset> {
    let mut combis = HashSet::new();

    println!("getting combis...");
    let blacklist = get_blacklist(prev_candidates, nono);

    for prev_candidate in prev_candidates {
        for curr_candidate in curr_candidates.iter() {
            if prev_candidate
                .iter()
                .any(|item| curr_candidate.contains(item))
            {
                continue;
            }

            if blacklist.contains_key(*prev_candidate)
                && curr_candidate
                    .iter()
                    .any(|item| curr_candidate.contains(item))
            {
                continue;
            }

            let mut combi: Itemset = curr_candidate.to_vec();
            combi.extend(&(*prev_candidate).clone());
            combi.sort_unstable();
            combis.insert(combi);
        }
    }
    combis
}

type FrequentItemset = Itemset;
type NonfrequentItemset = Itemset;

/// Assume sorted
fn get_blacklist(
    frequent_candidates: &[&Itemset],
    nonfrequent_candidates: &[&Itemset],
) -> HashMap<FrequentItemset, NonfrequentItemset> {
    let mut dict: HashMap<Itemset, Itemset> = HashMap::new();

    for frequent_candidate in frequent_candidates {
        for nonfrequent_candidate in nonfrequent_candidates {
            // if k is len 1

            let frequent_candidate1: HashSet<usize> = frequent_candidate.iter().cloned().collect();
            let nonfrequent_candidate1: HashSet<usize> =
                nonfrequent_candidate.iter().cloned().collect();

            if frequent_candidate1.is_subset(&nonfrequent_candidate1) {
                let mut nonfrequent_items: Vec<usize> = frequent_candidate1
                    .symmetric_difference(&nonfrequent_candidate1)
                    .copied()
                    .collect();

                nonfrequent_items.sort_unstable();

                if !nonfrequent_items.is_empty() {
                    if dict.contains_key(*frequent_candidate) {
                        let existing_nonfrequent_items = dict.get_mut(*frequent_candidate).unwrap();
                        existing_nonfrequent_items.extend(nonfrequent_items);
                        existing_nonfrequent_items.sort_unstable();
                    } else {
                        dict.insert(frequent_candidate.to_vec(), nonfrequent_items);
                    }
                }
            }
        }
    }

    dict
}
