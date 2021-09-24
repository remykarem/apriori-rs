use std::{
    collections::VecDeque,
    fmt::{Display, Formatter, Result},
};

use std::collections::HashMap;

type ItemId = usize;
type Itemset = Vec<ItemId>;
type ItemsetCounts<'l> = HashMap<Itemset, u32>;
type FrequentItemsets<'l> = HashMap<usize, ItemsetCounts<'l>>;

pub fn generate_rules(min_conf: &f32, counter: &FrequentItemsets) -> Vec<Rule> {
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
                    let combi = combi.iter().copied().collect();
                    bfs(&combi, min_conf, counter)
                })
                .collect::<Vec<Rule>>()
        })
        .collect()
}

fn bfs(combi: &Itemset, &min_conf: &f32, counter: &FrequentItemsets) -> Vec<Rule> {
    let mut queue: VecDeque<Rule> = VecDeque::new();
    let rules = Rule::from_pattern(combi);
    queue.extend(rules);
    let mut blacklist = vec![];
    let mut final_rules = vec![];

    while let Some(rule) = queue.pop_front() {
        println!("Analysing rule: {}", &rule);

        if rule.is_a_child_of_a_blacklisted_rule(&blacklist) {
            println!(
                " Skipping {} because it's a child of a blacklisted rule",
                &rule
            );
            continue;
        }

        let confidence = rule.compute_confidence(counter, combi);

        if confidence >= min_conf {
            if let Some(new_rules) = rule.create_children(&blacklist, Some(&queue)) {
                queue.extend(new_rules);
            }
            final_rules.push(rule);
        } else {
            println!(" Blacklisting because low confidence");
            blacklist.push(rule);
        }
    }

    final_rules
}

#[derive(Debug)]
pub struct Rule {
    split: usize,
    combi: Vec<usize>,
}

impl Rule {
    fn from_pattern(pattern: &Vec<usize>) -> Vec<Rule> {
        let mother = Rule {
            split: pattern.len(),
            combi: pattern.iter().copied().collect(),
        };
        mother.create_children(&[], None).unwrap()
    }
    fn create_children(
        &self,
        blacklist: &[Self],
        to_create: Option<&VecDeque<Self>>,
    ) -> Option<Vec<Self>> {
        if self.split <= 1 {
            return None;
        }

        let new_split = self.split - 1;
        let mut rules = Vec::with_capacity(new_split);
        let mut tmp_combi = self.combi.to_owned();

        for _ in 0..self.split {
            let window = &mut tmp_combi[..self.split];
            window.rotate_left(1);

            let mut combi = tmp_combi.clone();
            let antecd = &mut combi[..new_split];
            antecd.sort_unstable();
            let conseq = &mut combi[new_split..];
            conseq.sort_unstable();

            let rule = Self {
                split: new_split,
                combi,
            };

            if rule.is_going_to_be_created(to_create) {
                println!(" Skipping {} because it's queued to be created", &rule);
                continue;
            }

            if rule.is_a_child_of_a_blacklisted_rule(blacklist) {
                println!(
                    " Skipping {} because it's a child of a blacklisted rule",
                    &rule
                );
            } else {
                println!(" Creating {}", rule);
                rules.push(rule);
            }
        }

        Some(rules)
    }

    fn is_going_to_be_created(&self, to_create: Option<&VecDeque<Self>>) -> bool {
        if let Some(to_create) = to_create {
            if to_create.contains(self) {
                return true;
            }
        }
        false
    }

    fn is_a_child_of_a_blacklisted_rule(&self, blacklist: &[Self]) -> bool {
        blacklist
            .iter()
            .any(|blacklisted_rule| self.is_child_of(blacklisted_rule))
    }

    fn get_antecedent(&self) -> &[usize] {
        &self.combi[..self.split]
    }
    fn get_consequent(&self) -> &[usize] {
        &self.combi[self.split..]
    }
    fn is_child_of(&self, parent: &Self) -> bool {
        if self.combi.len() != parent.combi.len() {
            return false;
        }
        if self.get_consequent().len() <= parent.get_consequent().len() {
            return false;
        }

        let conseq = self.get_consequent();
        parent.get_consequent().iter().all(|x| conseq.contains(x))
    }
    fn compute_confidence(&self, counter: &FrequentItemsets, combi: &Itemset) -> f32 {
        let antecedent_support =
            counter[&self.get_antecedent().len()][self.get_antecedent()] as f32;
        // todo
        let union_support = counter[&self.combi.len()][combi] as f32;
        union_support / antecedent_support
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{:?} => {:?}",
            &self.combi[..self.split],
            &self.combi[self.split..]
        )
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Rule) -> bool {
        // assumes same pattern
        self.split == other.split && self.combi[self.split..] == other.combi[self.split..]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn mains() {
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

        let assoc_rules = generate_rules(&min_conf, &counter);

        for r in &assoc_rules {
            println!("{}", r);
        }
    }

    #[test]
    fn test_rule_eq_1() {
        let rule1 = Rule {
            split: 2,
            combi: vec![1, 2, 3, 5],
        };
        let rule2 = Rule {
            split: 2,
            combi: vec![1, 2, 3, 5],
        };
        assert!(rule1 == rule2);
    }
    #[test]
    fn test_rule_eq_2() {
        let rule1 = Rule {
            split: 2,
            combi: vec![1, 2, 3, 5],
        };
        let rule2 = Rule {
            split: 2,
            combi: vec![9, 10, 3, 5],
        };
        assert!(rule1 == rule2);
    }
    #[test]
    fn test_rule_eq_3() {
        let rule1 = Rule {
            split: 2,
            combi: vec![1, 2, 3, 5],
        };
        let rule2 = Rule {
            split: 2,
            combi: vec![9, 10, 5],
        };
        assert!(rule1 != rule2);
    }
    #[test]
    fn test_rule_contains() {
        let rules = VecDeque::from(vec![Rule {
            split: 3,
            combi: vec![1, 3, 4, 2],
        }]);
        let rule = Rule {
            split: 2,
            combi: vec![3, 5, 1, 2],
        };
        assert!(!rules.contains(&rule));
    }

    #[test]
    fn test_rule_children() {
        let rule = Rule {
            split: 4,
            combi: vec![1, 2, 3, 4, 5],
        };
        let mut children = rule.create_children(&[], None).unwrap();
        let child = children.pop().unwrap();
        // for child in children {
        // println!("{}", child);
        // assert!(child.get_antecedent().iter().sorted());
        // assert!(child.get_consequent().is_sorted());
        // }
        println!("{}", child);
        let children = child.create_children(&[], None).unwrap();
        for child in children {
            println!("{}", child);
            // assert!(child.get_antecedent().iter().sorted());
            // assert!(child.get_consequent().is_sorted());
        }
    }
    #[test]
    fn test_heritage() {
        let parent = Rule {
            split: 4,
            combi: vec![
                1, 2, 3, 4, // ante
                5, // conseq
            ],
        };
        let child = Rule {
            split: 3,
            combi: vec![
                1, 2, 3, // ante
                4, 5, //conseq
            ],
        };
        assert!(child.is_child_of(&parent));
    }
    #[test]
    fn test_create_children() {
        let pattern = vec![1, 2, 3, 4, 5];
        let rules = Rule::from_pattern(&pattern);
        for rule in rules {
            println!("{}", rule);
        }
    }
}
