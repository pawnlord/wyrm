use std::fs::File;

mod wasm_model;
mod file_reader;


fn main() {
    let file = File::open("snake.wasm").unwrap();

    let wasm_file = file_reader::wasm_deserialize(file).unwrap();


    println!("{wasm_file:?}");
}
