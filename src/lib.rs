#![allow(non_snake_case)]
use core::panic;
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

    // Then generate the rest
    for size in 2..=k {
        println!("\nCounting itemsets of length {}.", size);
        let prev_itemset_counts = &counter[&(size - 1_usize)];
        let mut counts = create_counts_from_prev(prev_itemset_counts, size);
        update_counts_with_transactions(&mut counts, &transactions_new, min_support, size);
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
) {
    let N = transactions.len() as f32;

    println!("Updating counts...");

    transactions
        .iter()
        .filter(|transaction| transaction.len() >= size)
        .for_each(|transaction| {
            for (candidate, count) in candidate_counts.iter_mut() {
                let n = transaction.len();
                if n == size {
                    if candidate == transaction {
                        *count += 1;
                    }
                } else if candidate.iter().all(|x| transaction.contains(x)) {
                    *count += 1;
                }
            }
        });

    println!("Pruning...");
    candidate_counts.retain(|_, &mut support_count| (support_count as f32 / N) >= min_support);
}

/// target k
fn create_counts_from_prev<'items>(
    itemset_counts: &ItemsetCounts<'items>,
    size: usize,
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
    let combinations = unique_items.iter().combinations(size);

    // bottlenec
    println!("checking combinations...");
    let mut num_combis = 0;

    for mut combi in combinations.into_iter() {
        combi.sort_unstable();

        for prev_itemset in itemset_counts.keys() {
            if prev_itemset.iter().zip(combi.iter()).all(|(x, &y)| x == y) {
                next_itemset_counts.insert(combi.iter().map(|x| **x).collect(), 0);
                num_combis += 1;
                break;
            }
        }
    }
    println!("combinations: {}", num_combis);

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

    #[test]
    fn update_counts() {
        let transactions = vec![transaction![0, 1]];
        let mut candidate_counts = hashmap! {
            itemset![0] => 0,
            itemset![1] => 0,
        };

        update_counts_with_transactions(&mut candidate_counts, &transactions, 0.0, 1);
        assert_eq!(candidate_counts.len(), 2);
        assert_eq!(candidate_counts[&itemset![0]], 1);
        assert_eq!(candidate_counts[&itemset![1]], 1);
    }

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

    // #[test]
    // fn test_get_blacklist() {
    //     let keys0 = vec![itemset![11], itemset![13], itemset![10]];
    //     let keys1: Vec<&Itemset> = keys0.iter().collect();

    //     let blacklist = vec![itemset![11, 10], itemset![10, 12]];
    //     let h = get_blacklist(&keys1, &blacklist);
    //     assert_eq!(
    //         h,
    //         hashmap! {
    //             itemset![11] => itemset![10],
    //             itemset![10] => itemset![11, 12],
    //         }
    //     );
    // }

    // #[test]
    // fn test_get_combinations() {
    //     let prevs0 = vec![itemset![0], itemset![3], itemset![1]];
    //     let prevs1: Vec<&Itemset> = prevs0.iter().collect();
    //     let nono = vec![itemset![0, 2], itemset![2, 4]];
    //     let currs0 = vec![
    //         itemset![0, 3],
    //         itemset![0, 4],
    //         itemset![3, 2],
    //         itemset![3, 4],
    //     ];
    //     let currs = currs0.iter().collect();
    //     let combis = get_combinations(&prevs1, &currs, &nono);
    //     assert_eq!(combis, hashset![itemset![0, 3, 4]]);
    // }
}

// fn get_combinations<'a>(
//     prevs: &Vec<&Itemset>,
//     currs: &Vec<&Itemset>,
//     nono: &[Itemset],
// ) -> HashSet<Itemset> {
//     let mut combis = HashSet::new();

//     let blacklist = get_blacklist(prevs, nono);

//     for &prev in prevs {
//         for &curr in currs.iter() {
//             if curr.is_superset(prev) {
//                 continue;
//             }

//             if blacklist.contains_key(prev)
//                 && curr
//                     .intersection(&blacklist[prev])
//                     .peekable()
//                     .peek()
//                     .is_some()
//             {
//                 continue;
//             }
//             let mut h: Itemset = (*curr).iter().cloned().collect();
//             let k: Itemset = prev.iter().cloned().collect();
//             h.extend(k);
//             combis.insert(h);
//         }
//     }
//     combis
// }

// fn get_blacklist<'a>(keys: &[&Itemset], blacklist: &[Itemset]) -> HashMap<Itemset, Itemset> {
//     let mut dict: HashMap<Itemset, Itemset> = HashMap::new();

//     for &key in keys {
//         for x in blacklist.iter() {
//             if !x.is_superset(key) {
//                 continue;
//             }

//             let diff: BTreeSet<usize> = x.difference(key).copied().collect();

//             if dict.contains_key(key) {
//                 let entry = dict.get_mut(key).unwrap();
//                 entry.extend(diff);
//             } else {
//                 dict.insert(key.clone(), diff);
//             }
//         }
//     }

//     dict
// }
