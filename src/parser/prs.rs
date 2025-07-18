use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use log::debug;

pub trait GrammarTrait: Hash + PartialEq + Eq + Clone + Debug {
    fn start_sym() -> Self;
    
    
    fn to_node_rep(&self, parent_sym: Option<Self>) -> String {
        format!("{:?}", self).to_string()
    }
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
pub enum Derivation<'a, T: GrammarTrait + 'static> {
    ScannedFrom {
        symbol: T,
        idx: usize
    },
    CompletedFrom {
        state: EarleyState<'a, T>,
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct PackedNode<'a, T: GrammarTrait + 'static> {
    left_child: Derivation<'a, T>,

    // Derivations cannot be empty
    right_child: Option<Derivation<'a, T>>
}

#[derive(Clone, Debug)]
pub struct EarleyState<'a, T: GrammarTrait + 'static> {
    pub from: T,
    to: &'a [T],
    origin: usize,
    end: usize,
    idx: usize,
}



impl<T: GrammarTrait + 'static> PartialEq for EarleyState<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.origin == other.origin && self.idx == other.idx && self.end == self.end
    }
}
impl<T: GrammarTrait + 'static> Eq for EarleyState<'_, T> {}

impl<T: GrammarTrait + 'static> Hash for EarleyState<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
        self.origin.hash(state);
        self.end.hash(state);
        self.idx.hash(state);
    }
}

type Children<'a, T> = Vec<PackedNode<'a, T>>;

impl<'a, T: GrammarTrait + 'static> EarleyState<'a, T> {

    fn is_finished(&self) -> bool {
        self.idx >= self.to.len()
    }

    // Step forward this state with the statements that completed it
    // as a sub node
    fn next(&self, completed: Self, end: usize) -> (Self, Children<'a, T>) {
        let mut new: Vec::<PackedNode<T>> = Vec::new();
        new.push(PackedNode {
            left_child: Derivation::CompletedFrom { state: self.clone() },
            right_child: Some(Derivation::CompletedFrom { state: completed.clone() })
        });

        (Self {
            from: self.from.clone(),
            to: self.to.clone(),
            origin: self.origin,
            end,
            idx: self.idx + 1,
        }, new)
    }
    
    // Step forward this state with the value that was scanned
    fn next_scanned(&self, val: T, idx: usize) -> (Self, Children<'a, T>) {
        let mut new: Vec::<PackedNode<T>> = Vec::new();
        

        if self.idx == 0 {
            new.push(PackedNode {
                left_child: Derivation::ScannedFrom { symbol: val.clone(), idx },
                right_child: None
            });
        } else {            
            new.push(PackedNode {
                left_child: Derivation::CompletedFrom { state: self.clone() },
                right_child: Some(Derivation::ScannedFrom { symbol: val.clone(), idx })
            });
        }

        (Self {
            from: self.from.clone(),
            to: self.to.clone(),
            origin: self.origin,
            end: idx,
            idx: self.idx + 1,
        }, new)
    }


    pub fn elem(&self) -> T {
        self.to[self.idx].clone()
    }

    // Traverses the parse tree, breadth first, for an ambiguity
    // Returns some if there is an ambiguity (the first node with 2 packed nodes), 
    // None otherwise
    pub fn find_ambiguity(&self, states: &States<'a, T>) ->  Option<EarleyState<'a, T>> {        
        let mut queue = VecDeque::<EarleyState<T>>::new();
        queue.push_back(self.clone());

        while let Some(item) = queue.pop_front() {
            let Some(packed_nodes) = states.get(&item) else {
                return None;
            };

            if packed_nodes.len() > 1 {
                return Some(item);
            }
            if packed_nodes.len() == 0 {
                continue;
            }

            if let Derivation::CompletedFrom { 
                state: ref next_state 
            } = packed_nodes[0].left_child {
                queue.push_back(next_state.clone());
            };
            
            if let Some(Derivation::CompletedFrom {
                state: ref next_state 
            }) = packed_nodes[0].right_child {
                queue.push_back(next_state.clone());
            };
        }

        return None;
    }
}

pub fn earley_state_id<T: GrammarTrait + 'static>(state: &EarleyState<T>) -> String {
    let mut repr = "".to_string();
    repr += format!("{}_", state.from.to_node_rep(None)).as_str();
    for i in 0..state.to.len() {
        repr += format!("{}_", state.to[i].to_node_rep(None)).as_str();
    }
    repr += format!("{}_{}_{}", state.origin, state.end, state.idx).as_str();
    repr
}

pub fn earley_state_repr<T: GrammarTrait + 'static>(state: &EarleyState<T>) -> String {
    let mut repr = "".to_string();
    repr += format!("{} -> ", state.from.to_node_rep(None)).as_str();
    for i in 0..state.idx {
        repr += format!("{} ", state.to[i].to_node_rep(None)).as_str();
    }
    repr += format!("*").as_str();  
    for i in state.idx..state.to.len() {
        repr += format!(" {}", state.to[i].to_node_rep(None)).as_str();
    }
    repr += format!(", {}, {}", state.origin, state.end).as_str();
    repr
}

pub fn print_earley_states<T: GrammarTrait + 'static, F>(states: &States<T>, _grammar: &Grammar<T>, end: usize, output: F)
    where F: Fn(String) -> () {

    for (i, state_pair) in states.clone().iter().enumerate() {
        let mut repr = format!("{}:\t", i);
        let state = state_pair.0;
        let packed_nodes = state_pair.1;
        repr += earley_state_repr(state).as_str();

        if packed_nodes.len() > 0 {
            repr += "\t packed_nodes: ";
            repr += packed_nodes.iter().map(|node| {
                let mut str = "".to_string();

                str += match &node.left_child {
                    Derivation::CompletedFrom { state } => {
                        earley_state_repr(state)
                    },
                    Derivation::ScannedFrom { symbol, idx } => {
                        format!("{} @ {}", symbol.to_node_rep(None), idx)
                    }
                }.as_str();

                if let Some(right_child) = &node.right_child {
                    str += " || ";
                    str += match right_child {
                        Derivation::CompletedFrom { state } => {
                            earley_state_repr(state)
                        },
                        Derivation::ScannedFrom { symbol, idx } => {
                            format!(" & {:?} @ {}", symbol.to_node_rep(None), idx)
                        }
                    }.as_str();
                }

                str
            }).collect::<Vec<String>>().join(" -- ").as_str();
        }

        output(repr);
    }
    
}

type States<'a, T> = HashMap<EarleyState<'a, T>, Children<'a, T>>;

pub struct EarleySppf<'a, T: GrammarTrait + 'static> {
    pub states: States<'a, T>,
    pub root: EarleyState<'a, T>
}


type TreeEdges<'a, T> = Vec<(Derivation<'a, T>, Derivation<'a, T>)>;

#[derive(Debug)]
pub struct EarleyTree<'a, T: GrammarTrait + 'static> {
    pub edges: TreeEdges<'a, T>,
    pub root: Derivation<'a, T>
}


fn find_state<'a, T: GrammarTrait + 'static>(
    state: &EarleyState<'a, T>, states: & Vec<States<'a, T>>
) -> Option<Children<'a, T>> {
    for step in states {
        let maybe_state = step.get(state);
        if maybe_state.is_some() {
            return Some(maybe_state.unwrap().clone());
        }
    }

    None
}

fn create_sppf<'a, T: GrammarTrait + 'static>(start_state: &EarleyState<'a, T>, states: &Vec<States<'a, T>>) -> EarleySppf<'a, T> {
    let mut new_states  = States::<'a, T>::new();
    let mut full_set  = States::<'a, T>::new();

    let mut queue = VecDeque::<EarleyState<T>>::new();
    queue.push_back(start_state.clone());

    // Go down the forest and find all the elements referenced.
    // Add any completed derivation to the queue until none are left
    while let Some(item) = queue.pop_front() {
        let item_packed_nodes = find_state(&item, states).unwrap();
        for p in &item_packed_nodes {
            match &p.left_child {
                Derivation::CompletedFrom { state } => {
                    queue.push_back(state.clone());
                },
                _ => ()
            }
         
            if let Some(right) = &p.right_child {
                match right {
                    Derivation::CompletedFrom { state } => {
                        queue.push_back(state.clone());
                    },
                    _ => ()
                }                    
            }
        }
        new_states.insert(item, item_packed_nodes);
    }

    EarleySppf::<'a, T> {
        states: new_states,
        root: start_state.clone()
    }
}

impl<'a, T: GrammarTrait + 'static> EarleySppf<'a, T> {
    pub fn to_tree(&self) -> EarleyTree<'_, T> {
        let mut tree_states: TreeEdges<'a, T> = TreeEdges::<'a, T>::new();
        let mut queue = VecDeque::<EarleyState<T>>::new();
        let root_state = Derivation::CompletedFrom {
            state: self.root.clone()
        };

        queue.push_back(self.root.clone());

        while let Some(item) = queue.pop_front() {
            let deriv = Derivation::CompletedFrom { state: item.clone() };
            let new_node = self.states.get(&item)
                .expect("Root of SPPF not found in states");
            
            if new_node.len() == 0 {
                continue;
            }

            let packed_node = &new_node[0];

            

            match &packed_node.left_child {
                Derivation::CompletedFrom {state: next_state } => {
                    if next_state.idx != 0 {
                        queue.push_back(next_state.clone());
                        tree_states.push((deriv.clone(), Derivation::CompletedFrom {
                            state: next_state.clone() 
                        }));
                    }
                },
                Derivation::ScannedFrom {symbol, idx } => {
                    tree_states.push((deriv.clone(), Derivation::ScannedFrom {
                        idx: *idx, symbol: symbol.clone()
                    }));
                },
                _ => {}
            }
            
            match &packed_node.right_child {
                Some(Derivation::CompletedFrom {
                    state: next_state 
                }) => {
                    queue.push_back(next_state.clone());
                    tree_states.push((deriv.clone(), Derivation::CompletedFrom {
                        state: next_state.clone() 
                    }));
                },
                Some(Derivation::ScannedFrom {
                    symbol, idx 
                }) => {
                    tree_states.push((deriv.clone(), Derivation::ScannedFrom {
                        idx: *idx, symbol: symbol.clone()
                    }));
                },
                _ => {}
            }

        }

        EarleyTree {
            edges: tree_states,
            root: root_state
        }
    }
}

pub fn earley_parser<'a, T: GrammarTrait + 'static>(sentence: Vec<T>, grammar: &Grammar<T>) -> Option<EarleySppf<'a, T>> {
    let mut states = Vec::<States<'a, T>>::new();
    for _ in 0..(sentence.len() + 1) {
        states.push(HashMap::new());
    }

    // The initial rules, from which all rules will be derived
    for rule in grammar.get_rules(T::start_sym()) {
        for value in rule.right_hand {
            let len = states[0].len();
            states[0].insert(EarleyState::<'_, T> {
                from: rule.left_hand.clone(),
                to: value,
                origin: 0,
                end: 0,
                idx: 0,
            }, vec![]);
        }
    }

    // Earley parsing algorithm
    // https://en.wikipedia.org/wiki/Earley_parser
    for i in 0..(sentence.len() + 1) {
        // This keeps track of what states we haven't yet tried in the parser
        // as this can grow for a specific step during that step
        let mut queue = VecDeque::<EarleyState<T>>::new();

        for (state, _) in states[i].clone() {
            queue.push_back(state);
        }

        while let Some(item) = queue.pop_front() {

            if !item.is_finished() {
                let symbol = item.elem();

                if grammar.is_nonterm(&symbol) {
                    // Predictor
                    // Check what rules could potentially complete the statement in this step,
                    // then add them to this step 
                    for rule in grammar.rules.clone() {
                        if symbol == rule.left_hand {
                            for possibility in rule.right_hand.clone() {
                                let len = states[i].len();
                                let new_state = EarleyState::<'_, T> {
                                    from: symbol.clone(),
                                    to: possibility,
                                    origin: i,
                                    end: i,
                                    idx: 0,
                                };

                                // This seems slow, look at for speed in the future
                                if !states[i].contains_key(&new_state) {
                                    queue.push_back(new_state.clone());
                                    states[i].insert(new_state, vec![]);
                                }
                            }
                        }
                    }
                } else {
                    // Scanner
                    // See if the symbol at this step completes the rule
                    if i >= sentence.len() {
                        continue;
                    }
                
                    if sentence[i] == item.elem() {
                        let len = states[i + 1].len();
                        let (next, mut children) = item.next_scanned(sentence[i].clone(), i + 1);

                        if let Some(other)  = states[i + 1].get(&next) {
                            children.append(&mut other.clone());
                        }
                        states[i + 1].insert(next, children);
                    }
                }
            } else {
                // Completer
                // Check if any of our predictions were correct and push
                // it to the next step
                for (state, _) in states[item.origin].clone() {
                    if state.is_finished() {
                        continue;
                    }

                    if item.clone().from == state.elem() {
                        let (next, mut children) = state.next(item.clone(), i);
                        
                        if let Some(other)  = states[i].get(&next) {
                            children.append(&mut other.clone());
                        }
                        states[i].insert(next.clone(), children);
                        queue.push_back(next);
                    }
                }
            }
        }

        // debug!("states {}", i);
        // debug!("------------------------");
        // print_earley_states(&states[i], grammar, i, |x| debug!("{}", x));
    }

    let start_rule = grammar.get_rules(T::start_sym())[0].right_hand[0];
    // print_earley_states(&states[i], grammar, i, |x| debug!("{}", x));
    // println!("{:?}", states[sentence.len()]);
    let end_state = EarleyState::<'_, T> {
        from: T::start_sym(),
        to: &start_rule,
        origin: 0,
        end: sentence.len(),
        idx: start_rule.len(),
    };
    states[sentence.len()].get(&end_state)
        .map(|x| create_sppf(&end_state, &states))
}

macro_rules! user_rule {
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

macro_rules! rule {
    ($t:ident, $lhs:ident, $($rhs:expr),+) => {
        prs::Rule::<$t> {
            left_hand: $lhs,
            right_hand: &[$($rhs),+]
        }
    };
}

macro_rules! term_rule {
    ($t:ident, $lhs:ident, $rhs:ident) => {
        prs::Rule::<$t> {
            left_hand: $lhs,
            right_hand: &$rhs
        }
    };
}
pub(crate) use user_rule;
pub(crate) use rule;
pub(crate) use term_rule;

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
        user_rule!(Symbols, P, &[S]),
        user_rule!(Symbols, S, &[S, Plus, M], &[M]),
        user_rule!(Symbols, M, &[M, Times, T], &[T]),
        user_rule!(Symbols, T, &[One], &[Two], &[Three], &[Four]),
    ]);

    #[test]
    fn earley_parser_test() {
        use Symbols::*;
        let sentence = vec![Two, Plus, Three, Times, Four];
        println!("TESTING GRAMMAR");
        assert!(earley_parser(sentence, &GRAMMAR).is_some());
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
        user_rule!(AmbigSymbols, P, &[S]),
        user_rule!(AmbigSymbols, S, &[S, Plus, S], &[One]),
    ]);
    

    /*
     * Test for an ambiguous grammar, to see if the sppf is working
     */
    #[test]
    fn ambiguous_earley_parser_test() {
        use AmbigSymbols::*;
        let sentence = vec![One, Plus, One, Plus, One];
        println!("TESTING AMBIGUSOUS GRAMMAR");
        let result = earley_parser(sentence, &AMBIGUOUS_GRAMMAR);
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.root.find_ambiguity(&result.states).is_some());

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
        user_rule!(SimpleGrammar, P, &[S]),
        user_rule!(SimpleGrammar, S, &[S, S], &[A]),
    ]);
    

    /*
     * As simpler ambiguous grammar.
     */
    #[test]
    fn simple_ambiguous_earley_parser_test() {
        use SimpleGrammar::*;
        let sentence = vec![A, A, A];
        println!("TESTING SIMPLE GRAMMAR");
        let result = earley_parser(sentence, &SIMPLE_GRAMMAR);
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.root.find_ambiguity(&result.states).is_some());
    }
}
