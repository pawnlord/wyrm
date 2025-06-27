#![allow(dead_code)]
use std::fs::File;


use dot2::{GraphWalk, Labeller};
use log::debug;
use wasm_model::WasmIdiomPattern;

use crate::instr_table::INSTRS;
use crate::parser::prs::Derivation;
use crate::wat_emitter::emit_wat;
use std::env;
use prs::{earley_parser, print_earley_states, user_rule, GrammarTrait};
use crate::parser::*;
use crate::wasm_parser::PARSER_GRAMMAR;
use simple_logger::SimpleLogger;


mod wasm_model;
mod file_reader;
mod instr_table;
mod wat_emitter;
mod wasm_parser;
mod parser;


    
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum AmbigSymbols {
    S,
    Plus,
    One,
}

impl prs::GrammarTrait for AmbigSymbols {
    fn start_sym() -> Self {
        Self::S
    }
}

const AMBIGUOUS_GRAMMAR: prs::Grammar<AmbigSymbols> = prs::Grammar::<AmbigSymbols>::new(&[
    user_rule!(AmbigSymbols, S, &[S, Plus, S], &[One]),
]);

fn main() {
    SimpleLogger::new().init().unwrap();

    println!("Logging Level: {}", log::STATIC_MAX_LEVEL);

    log::debug!("Is this debugging");

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let info = INSTRS[2];
    println!("{:?}", info);

    let file = if args.len() < 2 {
        File::open("snake.wasm").unwrap()
    } else {
        File::open(args[1].as_str()).unwrap()

    };


    let wasm_file = file_reader::wasm_deserialize(file).unwrap();
    println!("{:}", emit_wat(&wasm_file));

    let result = earley_parser(wasm_file.code_section.functions[2].raw_body.clone(), &PARSER_GRAMMAR);
    

    // assert!(result.is_some());
    println!("{}", result.is_some());
    let sppf = result.unwrap();
    println!("{:?}", sppf.root.find_ambiguity(&sppf.states));
    let mut f = std::fs::File::create("example2.dot").unwrap();
    let tree = sppf.to_tree();
    println!("{:?}", wasm_file.code_section.functions[2].raw_body.len());


    
    // use AmbigSymbols::*;
    // let sentence = vec![One, Plus, One, Plus, One];
    // println!("TESTING AMBIGUSOUS GRAMMAR");  
    // let result = earley_parser(sentence, &AMBIGUOUS_GRAMMAR);
    // assert!(result.is_some());
    // let sppf = result.unwrap();
    // assert!(sppf.root.find_ambiguity(&sppf.states).is_some());
    // print_earley_states(&sppf.states, &AMBIGUOUS_GRAMMAR, 0, |x| debug!("{}", x));
oh    
    // for d in tree.nodes().iter() {
    //     match d {
    //         Derivation::CompletedFrom {state: s} => {
    //             debug!("d: {}", prs::earley_state_id(s));
    //         },
    //         Derivation::ScannedFrom { 
    //             symbol, idx
    //         }  => {
    //             // Scanned can never be the root
    //             let deriv = Derivation::ScannedFrom { symbol: symbol.clone(), idx: *idx };
    //             let edge = tree.edges.iter().find(|e| e.1 == deriv).unwrap();
    //             let parent_sym = if let Derivation::CompletedFrom { state } = &edge.0 {
    //                 Some(state.from.clone())
    //             } else {
    //                 None
    //             };
    //             debug!("N{}_{}", symbol.to_node_rep(parent_sym), idx);
    //         },
    //     }
    // }

    dot2::render(&tree, &mut f).unwrap();

}
