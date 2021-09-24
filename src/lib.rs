#![allow(non_snake_case)]
mod itemset;
mod rules;

use std::collections::HashSet;

use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict, PyFrozenSet};
use pyo3::wrap_pyfunction;

type TransactionRaw<'l> = HashSet<&'l str>;

fn main() {
    #[pymodule]
    fn apriori(_: Python, m: &PyModule) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(generate_frequent_itemsets_wrapper, m)?)?;
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
fn apriori(
    transactions: Vec<HashSet<&str>>,
    min_support: f32,
    min_confidence: f32,
    max_len: usize,
) {
    let (itemset_counts, _) =
        itemset::generate_frequent_itemsets(&transactions, min_support, max_len);

    println!("Creating rules");

    let rules = rules::generate_rules(&min_confidence, &itemset_counts);

    for rule in &rules {
        println!("{:?}", rule);
    }
}

#[pyfunction]
fn generate_frequent_itemsets_wrapper(
    transactions: Vec<TransactionRaw>,
    min_support: f32,
    max_length: usize,
) -> Py<PyDict> {
    let (itemset_counts, _) =
        itemset::generate_frequent_itemsets(&transactions, min_support, max_length);

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
