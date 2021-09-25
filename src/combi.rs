use itertools::Itertools;

use crate::types::Itemset;

// type FrequentItemset = Itemset;
// type NonfrequentItemset = Itemset;

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

/// Cross prev_frequent with curr_frequent.
/// If cross product has curr_nonfrequent, remove it.
/// Assumes prev_frequent has itemsets of length k
/// and curr_frequent (and curr_nonfrequent) has itemsets of length k+1.
/// O(n^3)
///
/// prev_frequent | curr_frequent
///           [0] | [0,1]
///           [1] | [0,3]
///           [2] | [1,2]
///               | [1,3]
/// curr_nonfrequent [0,2] [2,3]
///
/// • [0,1] (not enough)
/// • [0,3] (not enough)
/// • [0,1,2] (not okay; contains [0,2])
/// • [0,1,3] (okay)      <-- 1
/// • [0,1] (not enough)
/// • [0,1,3] (already in 1)
/// • [1,2] (not enough)
/// • [1,3] (not enough)
/// • [0,1,2] (not okay; contains [0,2])
/// • [0,2,3] (not okay; contains [2,3])
/// • [1,2] (not enough)
/// • [1,2,3] (not okay; contains [2,3])
// fn get_combinations(
//     prev_frequent: &[&Itemset],
//     curr_frequent: &[&Itemset],
//     curr_nonfrequent: &[&Itemset],
// ) -> HashSet<Itemset> {
//     let mut combis = HashSet::new();

//     println!("getting combis...");
//     let blacklist = get_blacklist(prev_frequent, curr_nonfrequent);

//     for &prev_frequent in prev_frequent {
//         for &curr_frequent in curr_frequent {
//             // if the cross-product will result in k items (instead of k+1)
//             if prev_frequent
//                 .iter()
//                 .any(|item| curr_frequent.contains(item))
//             {
//                 continue;
//             }

//             if blacklist.contains_key(prev_frequent)
//                 && curr_frequent
//                     .iter()
//                     .any(|item| curr_frequent.contains(item))
//             {
//                 continue;
//             }

//             let mut combi: Itemset = curr_frequent.to_vec();
//             combi.extend(&(*prev_frequent).clone());
//             combi.sort_unstable();
//             combis.insert(combi);
//         }
//     }
//     combis
// }

/// Assume sorted
// fn get_blacklist(
//     frequent_itemset: &[&Itemset],
//     nonfrequent_itemset: &[&Itemset],
// ) -> HashMap<FrequentItemset, NonfrequentItemset> {
//     let mut dict: HashMap<Itemset, Itemset> = HashMap::new();

//     for &frequent_itemset in frequent_itemset {
//         for &nonfrequent_itemset in nonfrequent_itemset {
//             // if k is len 1

//             let frequent_itemset1: HashSet<usize> = frequent_itemset.iter().cloned().collect();
//             let nonfrequent_itemset1: HashSet<usize> =
//                 nonfrequent_itemset.iter().cloned().collect();

//             if frequent_itemset1.is_subset(&nonfrequent_itemset1) {
//                 let mut nonfrequent_items: Vec<usize> = frequent_itemset1
//                     .symmetric_difference(&nonfrequent_itemset1)
//                     .copied()
//                     .collect();

//                 nonfrequent_items.sort_unstable();

//                 if !nonfrequent_items.is_empty() {
//                     if dict.contains_key(frequent_itemset) {
//                         let existing_nonfrequent_items = dict.get_mut(frequent_itemset).unwrap();
//                         existing_nonfrequent_items.extend(nonfrequent_items);
//                         existing_nonfrequent_items.sort_unstable();
//                     } else {
//                         dict.insert(frequent_itemset.to_vec(), nonfrequent_items);
//                     }
//                 }
//             }
//         }
//     }

//     dict
// }

#[cfg(test)]
mod test {
    use super::*;

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
        assert!(!y.is_empty());
        assert!(y.contains(&vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_join_step_3() {
        let mut itemsets: Vec<Itemset> =
            vec![vec![1, 2], vec![2, 3], vec![1, 3], vec![1, 4], vec![3, 4]];
        let y = join_step(&mut itemsets);
        println!("{:?}", y);
        assert!(y.len() >= 2);
        assert!(y.contains(&vec![1, 2, 3]));
        assert!(y.contains(&vec![1, 3, 4]));
    }

    // #[test]
    // fn test_get_blacklist() {
    //     let keys = vec![vec![11], vec![12], vec![20]];
    //     let keys: &Vec<&Itemset> = &keys.iter().collect();
    //     let blacklist = vec![vec![11, 20], vec![20, 23]];
    //     let blacklist: &Vec<&Itemset> = &blacklist.iter().collect();
    //     let h = get_blacklist(keys, blacklist);

    //     assert_eq!(h.len(), 2);
    //     assert_eq!(
    //         h,
    //         hashmap! {
    //             vec![11] => vec![20],
    //             vec![20] => vec![11, 23],
    //         }
    //     );
    // }

    // #[test]
    // fn test_get_combinations() {
    //     let prevs = vec![vec![0], vec![1], vec![2]];
    //     let prevs: &Vec<&Itemset> = &prevs.iter().collect();
    //     let currs = vec![vec![0, 1], vec![0, 3], vec![1, 2], vec![1, 3]];
    //     let currs: &Vec<&Itemset> = &currs.iter().collect();
    //     let nonfrequent = vec![vec![0, 2], vec![2, 3]];
    //     let nonfrequent: &Vec<&Itemset> = &nonfrequent.iter().collect();

    //     let combis = get_combinations(prevs, currs, nonfrequent);

    //     assert_eq!(combis, hashset![vec![0, 1, 3]]);
    // }
}
