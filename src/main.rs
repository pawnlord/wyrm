#![allow(dead_code)]
use std::fs::File;


use log::debug;
use wasm_model::WasmIdiomPattern;

use crate::instr_table::INSTRS;
use crate::wat_emitter::emit_wat;
use std::env;
use prs::{earley_parser, print_earley_states, user_rule};
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
    // println!("{:?}", wasm_file.code_section.functions[59].raw_body.as_slice());
    // println!("{:?}", wasm_file.code_section.functions[59].raw_body.len());

    let result = earley_parser(wasm_file.code_section.functions[59].raw_body.clone(), &PARSER_GRAMMAR);
    

    // assert!(result.is_some());
    println!("{}", result.is_some());
    println!("{:?}", result.unwrap().root.find_ambiguity())
    // use AmbigSymbols::*;
    // let sentence = vec![One, Plus, One, Plus, One];
    // println!("TESTING AMBIGUSOUS GRAMMAR");
    // let result = earley_parser(sentence, &AMBIGUOUS_GRAMMAR);
    // assert!(result.is_some());
    // print_earley_states(&result.unwrap().states, &AMBIGUOUS_GRAMMAR, 0, |x| debug!("{}", x));
}
