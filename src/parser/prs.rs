use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;
use log::debug;

pub trait GrammarTrait: Hash + PartialEq + Eq + Clone + Debug {
    fn start_sym() -> Self;
}


#[derive(Clone)]
pub struct Rule<T: GrammarTrait + 'static> {
    pub left_hand: T,
    pub right_hand: &'static [&'static [T]],
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
        Self { rules: rules }
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
pub struct ParseValue<'a, T: GrammarTrait + 'static> {
    from: T,
    to: &'a [T],
    pub start: usize,
    pub end: usize
}

enum ParseNode<T: GrammarTrait + 'static> {
    Terminal(T),
    Nonterminal {
        symbol: T,
        children: Vec<Box<ParseNode<T>>>
    }
}

#[derive(Clone, Debug)]
struct EarleyState<'a, T: GrammarTrait + 'static> {
    from: T,
    to: &'a [T],
    origin: usize,
    idx: usize,
    parse_vals: Vec<ParseValue<'a, T>>
}

impl<T: GrammarTrait + 'static> PartialEq for EarleyState<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.origin == other.origin && self.idx == other.idx
    }
}


impl<'a, T: GrammarTrait + 'static> EarleyState<'a, T> {
    fn is_finished(&self) -> bool {
        self.idx >= self.to.len()
    }

    fn next(&self, val: Option<ParseValue<'a, T>>) -> Self {
        let mut new = self.parse_vals.clone();
        val.map(|x|  new.push(x));

        Self {
            from: self.from.clone(),
            to: self.to.clone(),
            origin: self.origin,
            idx: self.idx + 1,
            parse_vals: new,
        }
    }

    fn elem(&self) -> T {
        self.to[self.idx].clone()
    }
}

fn print_earley_states<T: GrammarTrait + 'static, F>(states: &Vec<EarleyState<T>>, _grammar: &Grammar<T>, end: usize, output: F)
    where F: Fn(String) -> () {
    for state in states.clone() {
        let mut repr = "".to_string();
        repr += format!("{:?} -> ", state.from).as_str();
        for i in 0..state.idx {
            repr += format!("{:?} ", state.to[i]).as_str();
        }
        repr += format!("*").as_str();
        for i in state.idx..state.to.len() {
            repr += format!(" {:?}", state.to[i]).as_str();
        }
        repr += format!(", {}, {}", state.origin, end).as_str();

        if state.parse_vals.len() > 0 {
            repr += " vals: ";
            repr += state.parse_vals.iter().map(|parse_val| {
                let mut str = format!("({:?} ->", parse_val.from);
                for i in 0..parse_val.to.len() {
                    str += format!(" {:?}", parse_val.to[i]).as_str();
                }
                str += format!(", {}, {})", parse_val.start, parse_val.end).as_str();        
                str
            }).collect::<Vec<String>>().join(", ").as_str();
        }

        output(repr);
    }

}

pub fn earley_parser<T: GrammarTrait + 'static>(sentence: Vec<T>, grammar: &Grammar<T>) -> bool {
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
                idx: 0,
                parse_vals: Vec::new(),
            });
        }
    }

    for i in 0..(sentence.len() + 1) {
        let mut queue = VecDeque::<EarleyState<T>>::new();

        for state in states[i].clone() {
            queue.push_back(state);
        }

        while let Some(item) = queue.pop_front() {

            if !item.is_finished() {
                let symbol = item.elem();

                if grammar.is_nonterm(&symbol) {
                    // Predictor
                    for rule in grammar.rules.clone() {
                        if symbol == rule.left_hand {
                            for possibility in rule.right_hand.clone() {
                                let new_state = EarleyState::<'_, T> {
                                    from: symbol.clone(),
                                    to: possibility,
                                    origin: i,
                                    idx: 0,
                                    parse_vals: Vec::new(),
                                };

                                if !states[i].contains(&new_state) {
                                    queue.push_back(new_state.clone());
                                    states[i].push(new_state);
                                }
                            }
                        }
                    }
                } else {
                    // Scanner
                    if i >= sentence.len() {
                        continue;
                    }
                
                    if sentence[i] == item.elem() {
                        states[i + 1].push(item.next(Some(ParseValue {
                            from: item.clone().from,
                            to: item.to,
                            start: item.origin,
                            end: i
                        })));
                    }
                }
            } else {
                // Completer
                for state in states[item.origin].clone() {
                    if state.is_finished() {
                        continue;
                    }

                    if item.clone().from == state.elem() {
                        let val = Some(ParseValue {
                            from: item.clone().from,
                            to: item.to,
                            start: item.origin,
                            end: i
                        });
                        queue.push_back(state.next(val.clone()));
                        states[i].push(state.next(val));
                    }
                }
            }
        }

        debug!("states {}", i);
        debug!("------------------------");
        print_earley_states(&states[i], grammar, i, |x| debug!("{}", x));
    }

    let start_rule = grammar.get_rules(T::start_sym())[0].right_hand[0];
    // println!("{:?}", states[sentence.len()]);
    states[sentence.len()].contains(&EarleyState::<'_, T> {
        from: T::start_sym(),
        to: &start_rule,
        origin: 0,
        idx: start_rule.len(),
        parse_vals: Vec::new(),
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
pub(crate) use rule;

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

    const GRAMMAR: prs::Grammar<Symbols> = prs::Grammar::<Symbols>::new(&[
        rule!(Symbols, P, &[S]),
        rule!(Symbols, S, &[S, Plus, M], &[M]),
        rule!(Symbols, M, &[M, Times, T], &[T]),
        rule!(Symbols, T, &[One], &[Two], &[Three], &[Four]),
    ]);

    #[test]
    fn earley_parser_test() {
        use Symbols::*;
        let sentence = vec![Two, Plus, Three, Times, Four];
        println!("TESTING GRAMMAR");
        assert!(earley_parser(sentence, &GRAMMAR));
    }

    #[derive(Hash, PartialEq, Eq, Clone, Debug)]
    pub enum AmbigSymbols {
        P,
        S,
        Plus,
        One,
    }

    impl prs::GrammarTrait for AmbigSymbols {
        fn start_sym() -> Self {
            Self::P
        }
    }

    const AMBIGUOUS_GRAMMAR: prs::Grammar<AmbigSymbols> = prs::Grammar::<AmbigSymbols>::new(&[
        rule!(AmbigSymbols, P, &[S]),
        rule!(AmbigSymbols, S, &[S, Plus, S], &[One]),
    ]);
    
    #[test]
    fn ambiguous_earley_parser_test() {
        use AmbigSymbols::*;
        let sentence = vec![One, Plus, One, Plus, One];
        println!("TESTING AMBIGUSOUS GRAMMAR");
        assert!(earley_parser(sentence, &AMBIGUOUS_GRAMMAR));
    }

    
    #[derive(Hash, PartialEq, Eq, Clone, Debug)]
    pub enum SimpleGrammar {
        P,
        S,
        A,
    }

    impl prs::GrammarTrait for SimpleGrammar {
        fn start_sym() -> Self {
            Self::P
        }
    }

    const SIMPLE_GRAMMAR: prs::Grammar<SimpleGrammar> = prs::Grammar::<SimpleGrammar>::new(&[
        rule!(SimpleGrammar, P, &[S]),
        rule!(SimpleGrammar, S, &[S, S], &[A]),
    ]);
    
    #[test]
    fn simple_ambiguous_earley_parser_test() {
        use SimpleGrammar::*;
        let sentence = vec![A, A, A];
        println!("TESTING SIMPLE GRAMMAR");
        assert!(earley_parser(sentence, &SIMPLE_GRAMMAR));
    }
}
