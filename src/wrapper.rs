use crate::types::{FrequentItemsets, Inventory};
use crate::rule;
use crate::Rule;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict, PyFrozenSet};
use std::cmp::Ordering::Equal;

macro_rules! pyfrozenset {
    ($py:expr,$x:expr) => {{
        let set: Py<PyFrozenSet> = PyFrozenSet::new($py, &$x).unwrap().into();
        set
    }};
}

pub fn convert_itemset_counts_id(itemset_counts: FrequentItemsets) -> Py<PyDict> {
    Python::with_gil(|py| {
        itemset_counts
            .into_iter()
            .map(|(size, itemset_counts)| {
                let py_itemset_counts: Py<PyDict> = itemset_counts
                    .into_iter()
                    .map(|(itemset, count)| (pyfrozenset![py, itemset], count))
                    .collect::<Vec<(Py<PyFrozenSet>, u32)>>()
                    .into_py_dict(py)
                    .into();
                (size, py_itemset_counts)
            })
            .into_py_dict(py)
            .into()
    })
}

pub fn convert_itemset_counts(itemset_counts: FrequentItemsets) -> Py<PyDict> {
    Python::with_gil(|py| {
        itemset_counts
            .into_iter()
            .map(|(size, itemset_counts)| {
                let py_itemset_counts: Py<PyDict> = itemset_counts
                    .into_iter()
                    .map(|(itemset, count)| (pyfrozenset![py, itemset], count))
                    .collect::<Vec<(Py<PyFrozenSet>, u32)>>()
                    .into_py_dict(py)
                    .into();
                (size, py_itemset_counts)
            })
            .into_py_dict(py)
            .into()
    })
}

pub fn convert_rules(rules: Vec<rule::Rule>, inventory: Inventory) -> Vec<Rule> {
    let mut pyrules: Vec<Rule> = rules
        .into_iter()
        .map(|x| Rule {
            antecedent: x
                .get_antecedent()
                .iter()
                .map(|item_id| String::from(inventory[item_id]))
                .collect(),
            consequent: x
                .get_consequent()
                .iter()
                .map(|item_id| String::from(inventory[item_id]))
                .collect(),
            confidence: x.confidence,
            lift: x.lift,
        })
        .collect();
    pyrules.sort_by(|a, b| (-a.confidence).partial_cmp(&-b.confidence).unwrap_or(Equal));
    pyrules
}
