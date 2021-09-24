#![allow(non_snake_case)]
mod combi;
mod itemset;
mod rules;
mod types;

use pyo3::types::{IntoPyDict, PyDict, PyFrozenSet};
use pyo3::wrap_pyfunction;
use pyo3::{prelude::*, PyObjectProtocol};
use std::collections::{HashMap, HashSet};
use types::FrequentItemsets;

type RawTransaction<'l> = HashSet<&'l str>;

fn main() {
    #[pymodule]
    fn apriori(_: Python, m: &PyModule) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(generate_frequent_itemsets, m)?)?;
        m.add_function(wrap_pyfunction!(apriori, m)?)?;
        m.add_class::<Person>()?;
        Ok(())
    }
}

#[pyclass]
struct Person {
    #[pyo3(get, set)]
    name: String,
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
) -> Vec<Rulez> {
    let (itemset_counts, _) =
        itemset::generate_frequent_itemsets(&transactions, min_support, max_len);

    let rules = rules::generate_rules(&min_confidence, &itemset_counts);

    rules
        .into_iter()
        .map(|x| Rulez {
            split: x.split,
            combi: x.combi,
        })
        .collect()
}

#[pyclass]
struct Rulez {
    split: usize,
    combi: Vec<usize>,
}

#[pyproto]
impl PyObjectProtocol for Rulez {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Rule {:?} => {:?}",
            &self.combi[..self.split],
            &self.combi[self.split..]
        ))
    }
}

#[pyfunction]
fn generate_frequent_itemsets(
    transactions: Vec<RawTransaction>,
    min_support: f32,
    max_length: usize,
) -> Py<PyDict> {
    let (itemset_counts, _) =
        itemset::generate_frequent_itemsets(&transactions, min_support, max_length);

    convert_itemset_counts(itemset_counts)
}

fn convert_itemset_counts(itemset_counts: FrequentItemsets) -> Py<PyDict> {
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
