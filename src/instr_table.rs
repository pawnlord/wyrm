use crate::wasm_model::*;

include!(concat!(env!("OUT_DIR"), "/instr_table.rs"));

// how 2 constexpr???
pub fn get_instr(name: &str) -> Option<InstrInfo> {
    INSTRS.iter().find(|x| {x.name == name}).map(|x| {*x})
}
