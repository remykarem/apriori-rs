#![allow(dead_code)]
pub mod combi;
pub mod itemset;
mod pcy;
pub mod rules;
pub mod types;
mod wrapper;

use pcy::__pyo3_get_function_pcy;
use itemset::__pyo3_get_function_generate_frequent_item_counts;
use pyo3::wrap_pyfunction;
use pyo3::{prelude::*, PyObjectProtocol};
use std::collections::{HashMap, HashSet};
use types::{Inventory, PyFrequentItemsets, PyItemName, RawTransaction};

fn main() {
    #[pymodule]
    fn apriori(_: Python, m: &PyModule) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(apriori, m)?)?;
        m.add_function(wrap_pyfunction!(generate_frequent_itemsets, m)?)?;
        m.add_function(wrap_pyfunction!(generate_frequent_item_counts, m)?)?;
        m.add_function(wrap_pyfunction!(pcy, m)?)?;
        m.add_class::<Rule>()?;
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
///     A tuple of (i) a list of association rules and (ii) frequent itemsets by size.
#[pyfunction]
#[pyo3(text_signature = "(/, *, transactions, min_support, min_confidence, max_len)")]
fn apriori(
    raw_transactions: Vec<RawTransaction>,
    min_support: f32,
    min_confidence: f32,
    max_len: usize,
) -> (Vec<Rule>, PyFrequentItemsets) {
    let (itemset_counts, inventory) =
        itemset::generate_frequent_itemsets(raw_transactions, min_support, max_len);

    let rules = rules::generate_rules(&min_confidence, &itemset_counts);

    (
        wrapper::convert_rules(rules, inventory),
        wrapper::convert_itemset_counts(itemset_counts),
    )
}

/// Generate frequent itemsets from a list of transactions.
///
/// Args:
///     transactions: A list of list of items.
///     min_support: The minimum support.
///     max_len: Maximum no. of items in an association rule.
///
/// Returns:
///     A tuple of (i) frequent itemsets by size and (ii) a dictionary mapping of item ID to item name.
#[pyfunction]
#[pyo3(text_signature = "(/, *, transactions, min_support, max_len)")]
fn generate_frequent_itemsets(
    raw_transactions: Vec<RawTransaction>,
    min_support: f32,
    max_length: usize,
) -> (PyFrequentItemsets, Inventory) {
    let (itemset_counts, inventory) =
        itemset::generate_frequent_itemsets(raw_transactions, min_support, max_length);

    (wrapper::convert_itemset_counts(itemset_counts), inventory)
}

#[pyclass]
pub struct Rule {
    #[pyo3(get)]
    antecedent: HashSet<PyItemName>,
    #[pyo3(get)]
    consequent: HashSet<PyItemName>,
    #[pyo3(get)]
    confidence: f32,
}

#[pyproto]
impl PyObjectProtocol for Rule {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?} -> {:?}", &self.antecedent, &self.consequent))
    }
}
