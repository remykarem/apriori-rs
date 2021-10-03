use itertools::Itertools;

use crate::types::{ItemId, Itemset};

/// https://github.com/tommyod/Efficient-Apriori/blob/master/efficient_apriori/itemsets.py
pub fn join_step(mut itemsets: Vec<Itemset>) -> Vec<Itemset> {
    if itemsets.is_empty() {
        return vec![];
    }

    itemsets.sort_unstable();

    let mut final_itemsets: Vec<Itemset> = Vec::with_capacity(1024); // arbitrary
    let mut itemset_first_tuple: Itemset = Vec::with_capacity(itemsets[0].len() + 1);
    let mut tail_items: Vec<ItemId> = Vec::with_capacity(itemsets.len()); // based on analysis of the first for loop

    let mut i = 0;
    while i < itemsets.len() {
        let mut skip = 1;

        let (itemset_first, itemset_last) = itemsets[i].split_at(itemsets[i].len() - 1);
        let itemset_last = itemset_last.to_owned().pop().unwrap();

        tail_items.clear();
        tail_items.push(itemset_last);

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
            itemset_first_tuple.clear();
            itemset_first_tuple.extend(itemset_first);
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_join_step() {
        let itemsets: Vec<Itemset> = vec![
            vec![1, 2, 3],
            vec![1, 2, 4],
            vec![1, 3, 4],
            vec![1, 3, 5],
            vec![2, 3, 4],
        ];
        let y = join_step(itemsets);
        assert_eq!(y.len(), 2);
        assert!(y.contains(&vec![1, 2, 3, 4]));
        assert!(y.contains(&vec![1, 3, 4, 5]));
    }

    #[test]
    fn test_join_step_2() {
        let itemsets: Vec<Itemset> =
            vec![vec![1, 2, 3], vec![1, 2, 4], vec![1, 3, 4], vec![2, 3, 4]];
        let y = join_step(itemsets);
        assert!(!y.is_empty());
        assert!(y.contains(&vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_join_step_3() {
        let itemsets: Vec<Itemset> =
            vec![vec![1, 2], vec![2, 3], vec![1, 3], vec![1, 4], vec![3, 4]];
        let y = join_step(itemsets);
        println!("{:?}", y);
        assert!(y.len() >= 2);
        assert!(y.contains(&vec![1, 2, 3]));
        assert!(y.contains(&vec![1, 3, 4]));
    }
}
