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

    use Symbols::*;
    let sentence = vec![Two, Plus, Three, Times, Four];
    println!("TESTING GRAMMAR");
    assert!(earley_parser(sentence, &GRAMMAR));
}
