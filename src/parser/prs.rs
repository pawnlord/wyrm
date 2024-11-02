use std::collections::{HashSet, VecDeque};
use std::hash::Hash;


pub trait GrammarTrait : Hash + PartialEq + Eq + Clone {
    fn start_sym() -> Self;
}

#[derive(Clone)]
pub struct Rule<T: GrammarTrait + 'static> {
    pub left_hand: T,
    pub right_hand: &'static [&'static [T]]
}

pub struct Grammar<T: GrammarTrait + 'static> {
    pub rules: &'static [Rule<T>],
    pub nonterminals: HashSet<T>,
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
        let mut nonterminals = HashSet::<T>::new();
        
        for rule in rules.clone() {
            nonterminals.insert(rule.left_hand);
        }

        Self {
            rules: rules,
            nonterminals
        }
    }

    pub fn is_nonterm(&self, elem: &T) -> bool {
        self.nonterminals.contains(elem)
    }
}

#[derive(Clone)]
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

pub fn earley_parser<T: GrammarTrait + 'static>(sentence: Vec<T>, grammar: &Grammar<T>) {
    let mut states = Vec::<Vec<EarleyState<T>>>::new();
    for _ in 0..(sentence.len() + 1) {
        states.push(Vec::new());
    }
    
    for rule in grammar.get_rules(T::start_sym()) {
        for value in rule.right_hand {
            states[0].push(EarleyState::<'_, T> {
                from: rule.left_hand.clone(),
                to: value,
                origin: 0,
                idx: 0
            });
        }
    }

    for i in 0..(sentence.len() + 1) {
        let mut queue = VecDeque::<EarleyState<T>>::new();
        
        queue.append(&mut states[0].clone().into());

        while let Some(item) = queue.pop_front() {
            if !item.is_finished() {
                let symbol = item.elem();
                if grammar.is_nonterm(&symbol) {
                    for rule in grammar.rules.clone() {
                        if symbol == rule.left_hand {

                        }
                    }

                } else {

                }
            } else {
                for state in states[item.origin].clone() {
                    if item.from == state.elem() {
                        states[i].push(state.next());
                    }
                }
            }
        }
    }

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
    
    #[derive(Hash, PartialEq, Eq, Clone)]
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

    const grammar: prs::Grammar<Symbols> = prs::Grammar::<Symbols>::new(
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
        earley_parser(sentence, &grammar);
    }
}