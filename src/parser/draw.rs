use std::collections::{HashMap, HashSet};

use log::debug;

use crate::parser::prs::{earley_state_id, earley_state_repr, Derivation, EarleySppf, EarleyState, EarleyTree, GrammarTrait};

type Dot2Node<'a, T> = Derivation<'a, T>;
type Dot2Edge<'a, T> = (Dot2Node<'a, T>, Dot2Node<'a, T>);
type Edges<'a, T> = EarleyTree<'a, T>;

impl<'a, T: GrammarTrait + 'static> dot2::Labeller<'a> for Edges<'a, T> {
    type Node = Dot2Node<'a, T>;
    type Edge = Dot2Edge<'a, T>;
    type Subgraph = ();

    fn graph_id(&'a self) -> dot2::Result<dot2::Id<'a>> {
        dot2::Id::new("TemporaryName")
    }

    fn node_id(&'a self, n: &Dot2Node<'a, T>) -> dot2::Result<dot2::Id<'a>> {
        match n {
            Derivation::CompletedFrom { 
                state
            } => {
                dot2::Id::new(format!("N{}", earley_state_id(state)))
            },
            Derivation::ScannedFrom { 
                symbol, idx
            } => {
                dot2::Id::new(format!("N{}_{}", symbol.to_node_rep(None), idx))
            }
        }
    }

    fn node_label(&'a self, n: &Dot2Node<'a, T>) -> dot2::Result<dot2::label::Text<'a>> {
        match n {
            Derivation::CompletedFrom { 
                state
            } => {
                Ok(dot2::label::Text::label(format!("{}", earley_state_repr(state))))
            },
            Derivation::ScannedFrom { 
                symbol, idx
            } => {
                // Scanned can never be the root
                let deriv = Derivation::ScannedFrom { symbol: symbol.clone(), idx: *idx };
                let edge = self.edges.iter().find(|e| e.1 == deriv).unwrap();
                let parent_sym = if let Derivation::CompletedFrom { state } = &edge.0 {
                    Some(state.from.clone())
                } else {
                    None
                };
                Ok(dot2::label::Text::label(format!("{}, {}", symbol.to_node_rep(parent_sym), idx)))
            }
        }
    }
}


impl<'a, T: GrammarTrait + 'static> dot2::GraphWalk<'a> for Edges<'a, T> {
    type Node = Dot2Node<'a, T>;
    type Edge = Dot2Edge<'a, T>;
    type Subgraph = ();

    
    fn nodes(&self) -> dot2::Nodes<'a,Dot2Node<'a, T>> {
        // (assumes that |N| \approxeq |E|)
        let v = &self.edges;
        let mut seen = HashSet::with_capacity(v.len());
        let mut nodes = Vec::with_capacity(v.len());

        for (s,t) in v {
            if !seen.contains(s) {
                nodes.push(s.clone())
            }
            if !seen.contains(t) {
                nodes.push(t.clone())
            }
            seen.insert(s.clone());
            seen.insert(t.clone());
        }
        nodes.into()
    }
    
    fn edges(&'a self) -> dot2::Edges<'a,Dot2Edge<'a, T>> {
        let edges = &self.edges;
        (&edges[..]).into()
    }

    fn source(&self, e: &Dot2Edge<'a, T>) -> Dot2Node<'a, T> {
        let (s,_) = e.clone();

        s
    }

    fn target(&self, e: &Dot2Edge<'a, T>) -> Dot2Node<'a, T> {
        let (_,t) = e.clone();

        t
    }
}