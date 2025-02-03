// An Earley parser for wasm specifically
use crate::{
    wasm_model::*,
    instr_table::*,
    parser::prs
};


pub const PARSER_GRAMMAR: prs::Grammar<u64> = prs::Grammar::<u64>::new(&[
    prs::rule!(u64, START, &[STMTS]),
    prs::rule!(u64, STMTS, &[STMT], &[STMTS, STMT]),
    prs::rule!(u64, STMT, &[ADD_U64_OP], &[INSTR]),
    prs::term_rule!(u64, INSTR, all_symbols),
    prs::rule!(u64, TERM_VOID, &[BYTE]),
    prs::rule!(u64, TERM_I32, &[LEB128]),
    prs::rule!(u64, TERM_I64, &[LEB128]),
    prs::rule!(u64, TERM_F32, &[DWORD]),
    prs::rule!(u64, TERM_F64, &[QWORD]),
    prs::rule!(u64, TERM_LOCAL, &[LEB128]),
    prs::rule!(u64, TERM_GLOBAL, &[LEB128]),
    prs::rule!(u64, TERM_GENERIC, &[LEB128]),
    prs::rule!(u64, TERM_FUNC, &[LEB128]),
    prs::rule!(u64, QWORD, &[BYTE, BYTE, BYTE, BYTE, BYTE, BYTE, BYTE, BYTE]),
    prs::rule!(u64, DWORD, &[BYTE, BYTE, BYTE, BYTE]),
    prs::term_rule!(u64, BYTE, bytes),
    prs::rule!(u64, LEB128, &[HIGH_BYTE, LEB128], &[LOW_BYTE]),
    prs::term_rule!(u64, LOW_BYTE, lower_bytes),
    prs::term_rule!(u64, HIGH_BYTE, upper_bytes),
]);