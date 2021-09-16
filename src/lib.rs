#![warn(non_snake_case)]
use itertools::Itertools;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::collections::{HashMap, HashSet}; // 0.8.2

macro_rules! str_vec {
    ($($x:expr),*) => {
        {
            let mut vec: Vec<String> = vec![];
            $(vec.push($x.into());)*
            vec
        }
    };
}
macro_rules! str_vec2 {
    ($($x:expr),*) => {
        {
            let mut vec: Vec<&String> = vec![];
            $(vec.push($x);)*
            vec
        }
    };
}

fn main() {
    #[pymodule]
    fn apriori(_: Python, m: &PyModule) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(apriori, m)?)?;
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
fn apriori(transactions: Vec<Vec<String>>, min_support: f32, min_confidence: f32, max_len: usize) {
    let N = transactions.len();

    let itemset_counts = create_itemset_counts_multipass(
        &transactions,
        max_len,
        (min_support * N as f32).round() as i32 + 1,
    );
    // let itemset_counts = create_itemset_counts(&transactions, max_len);

    let candidates = get_candidates(
        &itemset_counts,
        (min_support * N as f32).round() as i32 + 1,
        N as i32,
    );

    println!("Creating rules");

    candidates.iter().for_each(|&(candidate, count)| {
        let mut antecedents: HashSet<Vec<&String>> = HashSet::new();
        candidate
            .iter()
            .permutations(candidate.len())
            .for_each(|permutation| {
                for i in 1..permutation.len() {
                    let (x, y) = permutation.split_at(i);
                    let mut antecedent: Vec<&String> = x.iter().map(|&&x| x).collect();
                    if x.len() > 1 {
                        antecedent.sort();
                    }

                    if antecedents.contains(&antecedent) {
                        continue;
                    }

                    let den = (*itemset_counts.get(&antecedent).unwrap()) as f32;
                    let num = *count as f32;
                    if num > min_confidence * den {
                        println!("• {:?} -> {:?}", antecedent, y,);
                    }

                    antecedents.insert(antecedent);
                }
            });
    });
}

fn get_candidates<'a>(
    itemset_counts: &'a HashMap<Vec<&String>, i32>,
    min_support_count: i32,
    N: i32,
) -> Vec<(&'a Vec<&'a String>, &'a i32)> {
    println!("Getting candidates\n");
    itemset_counts
        .iter()
        .filter_map(|(itemset, count)| {
            if *count >= min_support_count {
                Some((itemset, count))
            } else {
                None
            }
        })
        .collect()
}

/// One-pass algorithm for generating itemsets
fn create_itemset_counts(
    transactions: &[Vec<String>],
    max_size: usize,
) -> HashMap<Vec<&String>, i32> {
    println!("Creating itemset counts\n");
    let mut itemset_counts: HashMap<Vec<&String>, i32> = HashMap::new();

    transactions.iter().for_each(|transaction| {
        for item in transaction {
            let count = itemset_counts.entry(vec![item]).or_insert(0);
            *count += 1;
        }

        (2..=max_size).for_each(|size| {
            let combis = transaction.iter().combinations(size);
            for mut itemset in combis {
                itemset.sort();
                let count = itemset_counts.entry(itemset).or_insert(0);
                *count += 1;
            }
        });
    });

    println!("{:?}", itemset_counts);

    itemset_counts
}

/// Multi-pass algorithm for generating itemsets
fn create_itemset_counts_multipass(
    transactions: &[Vec<String>],
    max_size: usize,
    min_support_count: i32,
) -> HashMap<Vec<&String>, i32> {
    println!("Creating itemset counts");
    let mut one_itemset_counts: HashMap<&String, i32> = HashMap::new();
    let mut itemset_counts: HashMap<Vec<&String>, i32> = HashMap::new();

    transactions.iter().for_each(|transaction| {
        for item in transaction {
            let count = one_itemset_counts.entry(item).or_insert(0);
            *count += 1;
        }
    });

    let mut to_remove = HashSet::new();
    one_itemset_counts.iter_mut().for_each(|(&k, &mut v)| {
        if v < min_support_count {
            println!("Removing {}. Support count {}", k, v);
            to_remove.insert(k);
        }
    });
    to_remove.iter().for_each(|item| {
        one_itemset_counts.remove(item);
    });

    transactions.iter().for_each(|transaction| {
        if !transaction.iter().any(|item| to_remove.contains(item)) {
            (2..=max_size).for_each(|size| {
                let combis = transaction.iter().combinations(size);
                for mut itemset in combis {
                    itemset.sort();
                    let count = itemset_counts.entry(itemset).or_insert(0);
                    *count += 1;
                }
            });
        }
    });

    to_remove.clear();

    one_itemset_counts.iter().for_each(|(&k, &v)| {
        itemset_counts.entry(vec![k]).or_insert(v);
    });
    one_itemset_counts.clear();

    println!("{:?}", itemset_counts);

    itemset_counts
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_itemset_counts_1_transaction() {
        let transactions = vec![str_vec!["bread", "milk"]];
        let itemset_counts = create_itemset_counts(&transactions, 2);

        assert_eq!(itemset_counts.len(), 3);
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string(), &"milk".to_string()]));
    }
    #[test]
    fn test_create_itemset_counts_1_transaction_multipass() {
        let transactions = vec![str_vec!["bread", "milk"]];
        let itemset_counts = create_itemset_counts_multipass(&transactions, 2, 0);

        assert_eq!(itemset_counts.len(), 3);
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string(), &"milk".to_string()]));
    }
    #[test]
    fn test_create_itemset_counts_1_transaction_max_1() {
        let transactions = vec![str_vec!["bread", "milk"]];
        let itemset_counts = create_itemset_counts(&transactions, 1);

        assert_eq!(itemset_counts.len(), 2);
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"milk".to_string()]));
    }
    #[test]
    fn test_create_itemset_counts_1_transaction_max_1_multipass() {
        let transactions = vec![str_vec!["bread", "milk"]];
        let itemset_counts = create_itemset_counts_multipass(&transactions, 1, 0);

        assert_eq!(itemset_counts.len(), 2);
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"milk".to_string()]));
    }
    #[test]
    fn test_create_itemset_counts_2_transactions() {
        let transactions = vec![str_vec!["bread", "milk"], str_vec!["bread", "yoghurt"]];
        let itemset_counts = create_itemset_counts(&transactions, 2);

        assert_eq!(itemset_counts.len(), 5);
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"yoghurt".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string(), &"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string(), &"yoghurt".to_string()]));
    }
    #[test]
    fn test_create_itemset_counts_2_transactions_multipass() {
        let transactions = vec![str_vec!["bread", "milk"], str_vec!["bread", "yoghurt"]];
        let itemset_counts = create_itemset_counts_multipass(&transactions, 2, 2);

        assert_eq!(itemset_counts.len(), 1);
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string()]));
    }
    #[test]
    fn test_create_itemset_counts_3_transactions() {
        let transactions = vec![
            str_vec!["bread", "milk"],
            str_vec!["bread", "yoghurt"],
            str_vec!["milk", "yoghurt", "cheese"],
        ];
        let itemset_counts = create_itemset_counts(&transactions, 3);

        assert_eq!(itemset_counts.len(), 10);
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"yoghurt".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"cheese".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string(), &"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string(), &"yoghurt".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"milk".to_string(), &"yoghurt".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"cheese".to_string(), &"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"cheese".to_string(), &"yoghurt".to_string()]));
        assert!(itemset_counts.contains_key(&vec![
            &"cheese".to_string(),
            &"milk".to_string(),
            &"yoghurt".to_string()
        ]));
    }
    #[test]
    fn test_create_itemset_counts_3_transactions_multipass_min_support_3() {
        let transactions = vec![
            str_vec!["bread", "milk"],
            str_vec!["bread", "yoghurt"],
            str_vec!["milk", "candy", "cheese"],
            str_vec!["bread", "yoghurt", "milk"],
        ];
        let itemset_counts = create_itemset_counts_multipass(&transactions, 3, 3);

        assert_eq!(itemset_counts.len(), 3);
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string(), &"milk".to_string()]));
    }
    #[test]
    fn test_create_itemset_counts_3_transactions_max_2() {
        let transactions = vec![
            str_vec!["bread", "milk"],
            str_vec!["bread", "yoghurt"],
            str_vec!["milk", "yoghurt", "cheese"],
        ];
        let itemset_counts = create_itemset_counts(&transactions, 2);

        assert_eq!(itemset_counts.len(), 9);
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"yoghurt".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"cheese".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string(), &"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"bread".to_string(), &"yoghurt".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"milk".to_string(), &"yoghurt".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"cheese".to_string(), &"milk".to_string()]));
        assert!(itemset_counts.contains_key(&vec![&"cheese".to_string(), &"yoghurt".to_string()]));
    }
}
