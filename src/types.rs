use pyo3::{Py, types::PyDict};

use crate::{HashMap,HashSet};

pub type ItemId = usize;
pub type ItemName<'l> = &'l str;
pub type PyItemName = String;
pub type Itemset = Vec<ItemId>;

pub type ReverseLookup<'l> = HashMap<ItemName<'l>, ItemId>;
pub type Inventory<'l> = HashMap<ItemId, ItemName<'l>>;

pub type RawTransaction<'l> = HashSet<ItemName<'l>>;
pub type Transaction = Vec<ItemId>;

pub type ItemCounts = HashMap<ItemId, u32>;
pub type ItemsetCounts = HashMap<Itemset, u32>;

pub type ItemsetLength = usize;
pub type FrequentItemsets = HashMap<ItemsetLength, ItemsetCounts>;
pub type PyFrequentItemsets = Py<PyDict>;
