use core::hash::{Hash, Hasher};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::{Display, Formatter, Result},
};
#[macro_use]
extern crate maplit;

type ItemId = usize;
type Itemset = Vec<ItemId>;
type Counts = HashMap<Itemset, usize>;
type ItemsetCounter = HashMap<usize, Counts>;

fn main() {
    let combi = vec![1, 2, 3, 4];
    let min_conf = 0.6;
    let counter: ItemsetCounter = hashmap! {
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
    println!("{:?}", counter);

    bfs(&combi, min_conf, &counter);
}

fn bfs(combi: &Itemset, min_conf: f32, counter: &ItemsetCounter) {
    let mut queue: VecDeque<Rule> = VecDeque::new();

    let rules = Rule::from_pattern(combi);
    let mut num_nodes_in_layer = rules.len() - 1;
    queue.extend(rules);
    let mut blacklist = vec![];
    let mut final_rules = vec![];

    while let Some(rule) = queue.pop_front() {
        let confidence = rule.compute_confidence(counter);

        if confidence >= min_conf {
            if let Some(new_rules) = rule.create_children(&blacklist, Some(&queue)) {
                queue.extend(new_rules);
            }
            println!("pushing {}", rule);
            final_rules.push(rule);
        } else {
            println!("blacklisting {}", rule);
            blacklist.push(rule);
        }

        println!("{}", num_nodes_in_layer);
        if num_nodes_in_layer == 1 {
            num_nodes_in_layer = queue.len() - 1;
        } else if num_nodes_in_layer == 0 {
            continue;
        } else {
            // pop
            num_nodes_in_layer -= 1;
        }
    }
    for r in &final_rules {
        println!("{}", r);
    }
}

#[derive(Debug)]
struct Rule<'mother> {
    reference: &'mother Vec<usize>,
    split: usize,
    combi: Vec<usize>,
}

impl<'mother> Rule<'mother> {
    fn from_pattern(pattern: &Vec<usize>) -> Vec<Rule> {
        let mother = Rule {
            reference: pattern,
            split: pattern.len(),
            combi: pattern.iter().copied().collect(),
        };
        mother.create_children(&[], None).unwrap()
    }
    fn create_children(
        &self,
        others: &[Self],
        explored: Option<&VecDeque<Self>>,
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
                reference: self.reference,
                split: new_split,
                combi,
            };

            if let Some(exploreds) = explored {
                if exploreds.contains(&rule) {
                    continue;
                }
            }
            if !others.iter().any(|other| rule.is_child_of(other)) {
                rules.push(rule);
            }
        }
        Some(rules)
    }

    fn get_antecedent(&self) -> &[usize] {
        &self.combi[..self.split]
    }
    fn get_consequent(&self) -> &[usize] {
        &self.combi[self.split..]
    }
    fn is_child_of(&self, parent: &Self) -> bool {
        if self.reference != parent.reference {
            return false;
        }
        if self.combi.len() != parent.combi.len() {
            return false;
        }
        if self.get_consequent().len() <= parent.get_consequent().len() {
            return false;
        }

        let conseq = self.get_consequent();
        parent.get_consequent().iter().all(|x| conseq.contains(x))
    }
    fn compute_confidence(&self, counter: &ItemsetCounter) -> f32 {
        let antecedent_support =
            counter[&self.get_antecedent().len()][self.get_antecedent()] as f32;
        // todo
        let union_support = counter[&self.combi.len()][self.reference] as f32;
        union_support / antecedent_support
    }
}

// fn get_stuff() {
//     let mut queue: VecDeque<Rule> = VecDeque::new();
//     let min_conf = 0.2;
//     let mother = vec![1, 2, 3, 4, 5];
//     let rules = Rule::from_pattern(&mother);
//     queue.extend(rules);

//     while let Some(rule) = queue.pop_front() {
//         println!("{}", rule);
//         let confidence = rule.compute_confidence();
//         if confidence >= min_conf {
//             if let Some(new_rules) = rule.create_children(&[], None) {
//                 queue.extend(new_rules);
//             }
//         } else {
//             continue;
//         }
//     }
// }

impl<'mother> Display for Rule<'mother> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{:?} => {:?}",
            &self.combi[..self.split],
            &self.combi[self.split..]
        )
    }
}

impl<'mother> PartialEq for Rule<'mother> {
    fn eq(&self, other: &Rule<'mother>) -> bool {
        // self.split == other.split
        //     && self.reference == other.reference
        //     && self.combi[..self.split] == other.combi[..self.split]
        self.combi[self.split..] == other.combi[self.split..]
    }
}

impl<'mother> Hash for Rule<'mother> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.combi[self.split..].hash(state);
    }
}

impl<'mother> Eq for Rule<'mother> {}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::Rule;
    #[test]
    fn test_rule() {
        let reference = vec![1, 2, 3, 4];
        let rule = Rule {
            reference: &reference,
            split: 2,
            combi: vec![1, 2, 3, 5],
        };
        println!("{}", rule);
    }
    #[test]
    fn test_rule_eq() {
        let reference = vec![1, 2, 3, 4];
        let rule1 = Rule {
            reference: &reference,
            split: 2,
            combi: vec![1, 2, 3, 5],
        };
        let rule2 = Rule {
            reference: &reference,
            split: 2,
            combi: vec![1, 2, 3, 5],
        };
        println!("{}", rule1 == rule2);
    }
    #[test]
    fn test_rule_hash() {
        let reference = vec![1, 2, 3, 4];
        let rule1 = Rule {
            reference: &reference,
            split: 2,
            combi: vec![1, 2, 3, 5],
        };
        let mut set: HashSet<Rule> = HashSet::new();
        set.insert(rule1);
        let rule2 = Rule {
            reference: &reference,
            split: 2,
            combi: vec![0, 0, 3, 5],
        };

        assert!(set.contains(&rule2));
    }

    #[test]
    fn test_rule_children() {
        let reference = vec![1, 2, 3, 4, 5];
        let rule = Rule {
            reference: &reference,
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
        let reference = vec![1, 2, 3, 4, 5];
        let parent = Rule {
            reference: &reference,
            split: 4,
            combi: vec![
                1, 2, 3, 4, // ante
                5, // conseq
            ],
        };
        let child = Rule {
            reference: &reference,
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
