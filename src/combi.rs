use std::collections::{HashMap, HashSet};

use itertools::Itertools;

type ItemId = usize;
type Itemset = Vec<ItemId>;

type FrequentItemset = Itemset;
type NonfrequentItemset = Itemset;

/// https://github.com/tommyod/Efficient-Apriori/blob/master/efficient_apriori/itemsets.py
pub fn join_step(itemsets: &mut [Itemset]) -> Vec<Itemset> {
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

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;

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
    fn test_join_step() {
        let mut itemsets: Vec<Itemset> = vec![
            vec![1, 2, 3],
            vec![1, 2, 4],
            vec![1, 3, 4],
            vec![1, 3, 5],
            vec![2, 3, 4],
        ];
        let y = join_step(&mut itemsets);
        assert_eq!(y.len(), 2);
        assert!(y.contains(&vec![1, 2, 3, 4]));
        assert!(y.contains(&vec![1, 3, 4, 5]));
    }

    #[test]
    fn test_join_step_2() {
        let mut itemsets: Vec<Itemset> =
            vec![vec![1, 2, 3], vec![1, 2, 4], vec![1, 3, 4], vec![2, 3, 4]];
        let y = join_step(&mut itemsets);
        assert_eq!(y.len(), 1);
        assert!(y.contains(&vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_get_blacklist() {
        let keys = vec![vec![11], vec![12], vec![20]];
        let keys: &Vec<&Itemset> = &keys.iter().collect();
        let blacklist = vec![vec![11, 20], vec![20, 23]];
        let blacklist: &Vec<&Itemset> = &blacklist.iter().collect();
        let h = get_blacklist(keys, blacklist);

        assert_eq!(h.len(), 2);
        assert_eq!(
            h,
            hashmap! {
                vec![11] => vec![20],
                vec![20] => vec![11, 23],
            }
        );
    }

    #[test]
    fn test_get_combinations() {
        let prevs = vec![vec![0], vec![1], vec![2]];
        let prevs: &Vec<&Itemset> = &prevs.iter().collect();
        let nonfrequent = vec![vec![0, 2], vec![2, 3]];
        let nonfrequent: &Vec<&Itemset> = &nonfrequent.iter().collect();
        let currs = vec![vec![0, 1], vec![0, 3], vec![1, 2], vec![1, 3]];
        let currs: &Vec<&Itemset> = &currs.iter().collect();
        let combis = get_combinations(prevs, currs, nonfrequent);
        
        assert_eq!(combis, hashset![vec![0, 1, 3]]);
    }
}
