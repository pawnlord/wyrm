use std::fs::File;

use wasm_model::INSTRS;

use crate::wat_emitter::emit_wat;

mod wasm_model;
mod file_reader;
mod wat_emitter;
mod pipeline;
mod usdm;
use std::env;

fn main() {
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
}
