use std::collections::VecDeque;

use crate::{
    rules::rule::Rule,
    types::{FrequentItemsets, ItemId, Itemset},
};

/// Generate rules based on frequent itemsets
pub fn generate_rules(min_conf: &f32, counter: &FrequentItemsets, N: usize) -> Vec<Rule> {
    let N = N as f32;
    counter
        .iter()
        .filter_map(|(&itemset_size, itemset_counts)| {
            if itemset_size > 1 {
                Some(itemset_counts)
            } else {
                None
            }
        })
        .flat_map(|itemset_counts| {
            itemset_counts
                .iter()
                .flat_map(|(combi, _)| {
                    let combi: Itemset = combi.iter().copied().collect();
                    bfs(&combi, min_conf, counter, N)
                })
                .collect::<Vec<Rule>>()
        })
        .collect()
}

/// Given a combination, find a list of rules that can be generated from it
pub fn bfs(combi: &[ItemId], &min_conf: &f32, counter: &FrequentItemsets, N: f32) -> Vec<Rule> {
    let mut queue: VecDeque<Rule> = VecDeque::new();
    let mut blacklist = vec![];
    let mut final_rules = vec![];

    let rules = Rule::from_pattern(combi);
    queue.extend(rules);

    while let Some(mut rule) = queue.pop_front() {
        if rule.is_a_child_of_a_blacklisted_rule(&blacklist) {
            continue;
        }

        rule.compute_confidence(counter, combi, N);

        if rule.confidence >= min_conf {
            if let Some(new_rules) = rule.create_children(&blacklist, Some(&queue)) {
                queue.extend(new_rules);
            }
            final_rules.push(rule);
        } else {
            blacklist.push(rule);
        }
    }

    final_rules
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    use crate::types::FrequentItemsets;

    #[test]
    fn test_1() {
        let counter: FrequentItemsets = hashmap! {
            1 => hashmap! {
                vec![1] => 9,
                vec![2] => 8,
                vec![3] => 12,
                vec![4] => 13,
            },
            2 => hashmap! {
                vec![1, 2] => 4,
                vec![1, 3] => 5,
                vec![1, 4] => 6,
                vec![2, 3] => 3,
                vec![2, 4] => 5,
                vec![3, 4] => 3,
            },
            3 => hashmap! {
                vec![1, 2, 3] => 3,
                vec![1, 2, 4] => 3,
                vec![1, 3, 4] => 3,
                vec![2, 3, 4] => 3,
            },
            4 => hashmap! {
                vec![1, 2, 3, 4] => 2,
            },
        };
        let min_conf = 0.8;

        let assoc_rules = generate_rules(&min_conf, &counter, 1);

        for r in &assoc_rules {
            println!("{}", r);
        }
    }

    #[test]
    fn test_2() {
        let counter: FrequentItemsets = hashmap! {
            1 => hashmap! {
                vec![1] => 9,
                vec![2] => 8,
                vec![3] => 12,
                vec![4] => 13,
            },
            2 => hashmap! {
                vec![1, 2] => 4,
                vec![1, 3] => 5,
                vec![1, 4] => 6,
                vec![2, 3] => 3,
                vec![2, 4] => 5,
                vec![3, 4] => 3,
            },
            3 => hashmap! {
                vec![1, 2, 3] => 3,
                vec![1, 2, 4] => 3,
                vec![1, 3, 4] => 3,
                vec![2, 3, 4] => 3,
            },
            4 => hashmap! {
                vec![1, 2, 3, 4] => 2,
            },
        };
        let min_conf = 0.8;

        let assoc_rules = generate_rules(&min_conf, &counter, 1);

        for r in &assoc_rules {
            println!("{}", r);
        }
    }
}
