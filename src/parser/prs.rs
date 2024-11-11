use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;


pub trait GrammarTrait : Hash + PartialEq + Eq + Clone + Debug {
    
    fn start_sym() -> Self;
}

#[derive(Clone)]
pub struct Rule<T: GrammarTrait + 'static> {
    pub left_hand: T,
    pub right_hand: &'static [&'static [T]]
}

pub struct Grammar<T: GrammarTrait + 'static> {
    pub rules: &'static [Rule<T>],
}

impl<T: GrammarTrait + 'static> Grammar<T> {
    pub fn get_rules(&self, lhs: T) -> Vec<Rule<T>> {
        let mut found = Vec::<Rule<T>>::new();
        for rule in self.rules.clone() {
            if rule.left_hand == lhs {
                found.push(rule.clone());
            }
        }
        found
    }

    pub const fn new(rules: &'static [Rule<T>]) -> Self {
        Self {rules: rules,}
    }

    pub fn is_nonterm(&self, elem: &T) -> bool {
        // This is annoyingly slow, but becuase I want grammars to be constant its needed.
        for rule in self.rules.clone() {
            if rule.left_hand == *elem {
                return true;
            }
        }
        false
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
struct EarleyState<'a, T: GrammarTrait + 'static> {
    from: T,
    to: &'a [T],
    origin: usize,
    idx: usize,
}

impl<T: GrammarTrait + 'static> EarleyState<'_, T> {
    fn is_finished(&self) -> bool {
        self.idx >= self.to.len()
    }

    fn next(&self) -> Self {
        Self {
            from: self.from.clone(),
            to: self.to.clone(),
            origin: self.origin,
            idx: self.idx + 1
        }
    }

    fn elem(&self) -> T {        
        self.to[self.idx].clone()
    }
}

pub fn earley_parser<T: GrammarTrait + 'static>(sentence: Vec<T>, grammar: &Grammar<T>) -> bool {
    let mut states = Vec::<HashSet<EarleyState<T>>>::new();
    for _ in 0..(sentence.len() + 1) {
        states.push(HashSet::new());
    }
    
    for rule in grammar.get_rules(T::start_sym()) {
        for value in rule.right_hand {
            states[0].insert(EarleyState::<'_, T> {
                from: rule.left_hand.clone(),
                to: value,
                origin: 0,
                idx: 0
            });
        }
    }

    println!("test");
    for i in 0..(sentence.len() + 1) {
        let mut queue = VecDeque::<EarleyState<T>>::new();
        
        for state in states[i].clone() {
            queue.push_back(state);
        }

        while let Some(item) = queue.pop_front() {
            println!("{:?}", queue);
            if !item.is_finished() {
                let symbol = item.elem();
                if grammar.is_nonterm(&symbol) {
                    for rule in grammar.rules.clone() {
                        if symbol == rule.left_hand {
                            for possibility in rule.right_hand.clone() {
                                let new_state = EarleyState::<'_, T> {
                                    from: symbol.clone(),
                                    to: possibility,
                                    origin: i,
                                    idx: 0
                                };
                                if !states[i].contains(&new_state) {
                                    queue.push_back(new_state.clone());
                                    states[i].insert(new_state);
                                }
                            }
                        }
                    }

                } else {
                    if i >= sentence.len() {
                        continue
                    }
                    if sentence[i] == item.elem() {
                        states[i + 1].insert(item.next());
                    }
                }
            } else {
                println!("complete");
                for state in states[item.origin].clone() {
                    if item.from == state.elem() {
                        queue.push_back(state.next());
                        states[i].insert(state.next());
                    }
                }
            }
        }
    }

    let start_rule = grammar.get_rules(T::start_sym())[0].right_hand[0];
    println!("{:?}", states[sentence.len()]);
    states[sentence.len()].contains(&EarleyState::<'_, T> {
        from: T::start_sym(),
        to: &start_rule,
        origin: 0,
        idx: start_rule.len()
    })

}

macro_rules! rule {
    ($t:ident, $lhs:ident, $($rhs:expr),+) => {
        {    
            use $t::*;
            prs::Rule::<$t> {
                left_hand: $lhs,
                right_hand: &[$($rhs),+]
            }
        }
    };
}


#[cfg(test)]
mod tests {
    use prs::earley_parser;

    use crate::parser::*;
    
    #[derive(Hash, PartialEq, Eq, Clone, Debug)]
    pub enum Symbols {
        P,
        S,
        M,
        T,
        One,
        Two,
        Three,
        Four,
        Times,
        Plus,
    }
    impl prs::GrammarTrait for Symbols {
        fn start_sym() -> Self {
            Self::P
        }
    }

    const GRAMMAR: prs::Grammar<Symbols> = prs::Grammar::<Symbols>::new(
        &[
            rule!(Symbols, P, &[S]),
            rule!(Symbols, S, &[S, Plus, M], &[M]),
            rule!(Symbols, M, &[M, Times, T], &[T]),
            rule!(Symbols, T, &[One], &[Two], &[Three], &[Four]),
        ]
    );


    #[test]
    fn earley_parser_test() {
        use Symbols::*;
        let sentence = vec![Two, Plus, Three, Times, Four];
        assert!(earley_parser(sentence, &GRAMMAR));
    }
}