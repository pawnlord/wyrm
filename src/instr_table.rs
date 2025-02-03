use crate::wasm_model::*;
use crate::parser::prs;
 

include!(concat!(env!("OUT_DIR"), "/instr_table.rs"));

const PARSER_GRAMMAR: prs::Grammar<u64> = prs::Grammar::<u64>::new(&[
    prs::rule!(u64, START, &[STMTS]),
    prs::rule!(u64, STMTS, &[STMT], &[STMTS, STMT]),
    prs::rule!(u64, STMT, &[ADD_U64_OP], &[INSTR]),
    prs::term_rule!(u64, INSTR, all_symbols),
]);

// how 2 constexpr???
pub fn get_instr(name: &str) -> Option<InstrInfo> {
    INSTRS.iter().find(|x| {x.name == name}).map(|x| {*x})
}

