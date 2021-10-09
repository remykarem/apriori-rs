#![allow(non_snake_case)]

use crate::types::{FrequentItemsets, ItemId};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Rule {
    pub split: usize,
    pub combi: Vec<ItemId>,
    pub confidence: f32,
    pub lift: f32,
}

impl Rule {
    pub fn from_pattern(pattern: &[ItemId]) -> Vec<Rule> {
        let mother = Rule {
            split: pattern.len(),
            combi: pattern.iter().copied().collect(),
            confidence: 0.0,
            lift: 0.0,
        };
        mother.create_children(&[], None).unwrap()
    }
    pub fn create_children(
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
                confidence: 0.0,
                lift: 0.0,
            };

            if rule.is_going_to_be_created(to_create) {
                continue;
            }

            if !rule.is_a_child_of_a_blacklisted_rule(blacklist) {
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

    pub fn is_a_child_of_a_blacklisted_rule(&self, blacklist: &[Self]) -> bool {
        blacklist
            .iter()
            .any(|blacklisted_rule| self.is_child_of(blacklisted_rule))
    }

    pub fn get_antecedent(&self) -> &[ItemId] {
        &self.combi[..self.split]
    }
    pub fn get_consequent(&self) -> &[ItemId] {
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
    pub fn compute_confidence(&mut self, counter: &FrequentItemsets, combi: &[ItemId], N: f32) {
        let antecedent_support_count =
            counter[&self.get_antecedent().len()][self.get_antecedent()] as f32;
        let consequent_support_count =
            counter[&self.get_consequent().len()][self.get_consequent()] as f32;
        let union_support_count = counter[&self.combi.len()][combi] as f32;
        self.confidence = union_support_count / antecedent_support_count;
        self.lift = union_support_count / (antecedent_support_count * consequent_support_count) * N
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
    use std::fmt::{Display, Formatter, Result};

    use super::*;

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

    #[test]
    fn test_rule_eq_1() {
        let rule1 = Rule {
            split: 2,
            combi: vec![1, 2, 3, 5],
            confidence: 0.0,
            lift: 0.0,
        };
        let rule2 = Rule {
            split: 2,
            combi: vec![1, 2, 3, 5],
            confidence: 0.0,
            lift: 0.0,
        };
        assert!(rule1 == rule2);
    }
    #[test]
    fn test_rule_eq_2() {
        let rule1 = Rule {
            split: 2,
            combi: vec![1, 2, 3, 5],
            confidence: 0.0,
            lift: 0.0,
        };
        let rule2 = Rule {
            split: 2,
            combi: vec![9, 10, 3, 5],
            confidence: 0.0,
            lift: 0.0,
        };
        assert!(rule1 == rule2);
    }
    #[test]
    fn test_rule_eq_3() {
        let rule1 = Rule {
            split: 2,
            combi: vec![1, 2, 3, 5],
            confidence: 0.0,
            lift: 0.0,
        };
        let rule2 = Rule {
            split: 2,
            combi: vec![9, 10, 5],
            confidence: 0.0,
            lift: 0.0,
        };
        assert!(rule1 != rule2);
    }
    #[test]
    fn test_rule_contains() {
        let rules = VecDeque::from(vec![Rule {
            split: 3,
            combi: vec![1, 3, 4, 2],
            confidence: 0.0,
            lift: 0.0,
        }]);
        let rule = Rule {
            split: 2,
            combi: vec![3, 5, 1, 2],
            confidence: 0.0,
            lift: 0.0,
        };
        assert!(!rules.contains(&rule));
    }

    #[test]
    fn test_rule_children() {
        let rule = Rule {
            split: 4,
            combi: vec![1, 2, 3, 4, 5],
            confidence: 0.0,
            lift: 0.0,
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
            confidence: 0.0,
            lift: 0.0,
        };
        let child = Rule {
            split: 3,
            combi: vec![
                1, 2, 3, // ante
                4, 5, //conseq
            ],
            confidence: 0.0,
            lift: 0.0,
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
