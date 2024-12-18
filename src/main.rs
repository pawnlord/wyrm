#![allow(dead_code)]
use std::fs::File;


use wasm_model::WasmIdiomPattern;

use crate::instr_table::INSTRS;
use crate::wat_emitter::emit_wat;
use std::env;
use prs::{earley_parser, rule};
use crate::parser::*;
use simple_logger::SimpleLogger;


mod wasm_model;
mod file_reader;
mod instr_table;
mod wat_emitter;
mod usdm;
mod wasm_parser;
mod parser;


    
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

    WasmIdiomPattern::double();

    let wasm_file = file_reader::wasm_deserialize(file).unwrap();
    println!("{:}", emit_wat(&wasm_file));

    use SimpleGrammar::*;
    let sentence = vec![A, A, A];
    println!("TESTING SIMPLE GRAMMAR");
    assert!(earley_parser(sentence, &SIMPLE_GRAMMAR));

}
