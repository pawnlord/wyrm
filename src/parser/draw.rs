// use std::collections::HashMap;

// use crate::parser::prs::{Derivation, EarleyState, GrammarTrait};

// pub struct EarleyGraph {
//     nodes: Vec<String>,
//     edges: Vec<(usize, usize)>,
// }



// impl<'a> dot2::Labeller<'a> for EarleyGraph {
//     type Node = String;
//     type Edge = (usize, usize);
//     type Subgraph = ();

//     fn graph_id(&'a self) -> dot2::Result<dot2::Id<'a>> {
//         dot2::Id::new("EarleyGraph")
//     }

//     fn node_id(&'a self, n: &Nd) -> dot2::Result<dot2::Id<'a>> {
//         dot2::Id::new(format!("N{}", *n))
//     }
//     // Add edge_label? idk
// }


// pub fn state_into_graph<'a, T: GrammarTrait + 'static>(root: EarleyState<'a, T>)  -> EarleyGraph {
//     let mut nodes = Vec::<String>::new();
//     let mut node_positions = HashMap::<String, usize>::new();



// }
