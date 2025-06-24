use crate::wasm_model::*;
use crate::parser::prs;
 

include!(concat!(env!("OUT_DIR"), "/instr_table.rs"));


// how 2 constexpr???
pub fn get_instr(name: &str) -> Option<InstrInfo> {
    INSTRS.iter().find(|x| {x.name == name}).map(|x| {*x})
}

pub fn get_instr_from_op(opcode: u64) -> Option<InstrInfo> {
    if (opcode as usize) < INSTRS.len() {
        Some(INSTRS[opcode as usize])
    } else {
        None
    }
}

pub fn get_special_sim(sym: u64) -> Option<String> {
    if ((u64::MAX - sym) as usize) < SPECIAL_SIMS.len() {
        Some(SPECIAL_SIMS[(u64::MAX - sym) as usize].to_string())
    } else {
        None
    }
}