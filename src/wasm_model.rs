use core::fmt;
use std::fmt::{Debug, Formatter};



pub trait TypeTrait {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prim{
    Void,
    I32,
    I64,
    F32,
    F64,
    FuncIdx
}

#[derive(Debug, Clone)]
pub struct BrTableConst {
    pub break_depths: Vec<usize>, 
    pub default: usize,
}

#[derive(Debug, Clone)]
pub enum ExprSeg {
    Instr(InstrInfo),
    // Raw bits of an int, signage and other things are figured out later (all ints are stored in the same manner)
    Int(u64),
    Float32(f32),
    Float64(f64),
    BrTable(BrTableConst),
}

#[derive(Debug, Clone)]
pub struct WasmExpr{
    pub expr: Vec<ExprSeg>    
}

pub fn type_values(t: Prim) -> (i32, String) {
    match t {
        Prim::Void => (0, "void".to_string()),
        Prim::I32 => (1, "i32".to_string()),
        Prim::I64 => (2, "i64".to_string()),
        Prim::F32 => (3, "f32".to_string()),
        Prim::F64 => (4, "f64".to_string()),
        Prim::FuncIdx => (5, "funcidx".to_string()),
    }
} 

// TODO: Model how the instruction affects the stack
#[derive(Clone, Copy)]
pub struct InstrInfo {
    pub instr: u8,
    pub name: &'static str,
    pub in_type: Prim,
    pub out_type: Prim,
    pub has_const: bool,
    pub takes_align: bool,
}

impl Debug for InstrInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { 
        let rettype = if self.out_type == Prim::Void {
            "".to_string()
        } else {
            format!(" -> {:?}", self.out_type)
        };

        let arg = if self.out_type == Prim::Void {
            "".to_string()
        } else {
            format!("{:?}", self.in_type)
        };
        
        let constant = self.has_const.then(|| "[constant]").unwrap_or("");

        write!(f, "{:#x}: {}{}({}){}" , self.instr, self.name, constant, arg, rettype)
    }
}

#[derive(PartialEq, Eq)]
pub enum SpecialInstr {
    None,
    BrTable, 
    BeginBlock,
    EndBlock,
    CallIndirect,
}

pub fn get_edge_case(info: InstrInfo) -> SpecialInstr {
    match info.instr {
        0x0e => SpecialInstr::BrTable,
        0x02 | 0x03 | 0x04 => SpecialInstr::BeginBlock,
        0x0b | 0x05 => SpecialInstr::EndBlock,
        0x11 => SpecialInstr::CallIndirect, 
        _ => SpecialInstr::None
    }
}

/**
 * Cases that aren't currently getting scraped:
 * 1. ref.func: doesn't appear directly in any scraped output, but is needed for elements
 */
pub const INSTRS: [InstrInfo; 256] = [
    InstrInfo{instr: 0x00, name: "unreachable", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x01, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x02, name: "block", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x03, name: "loop", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x04, name: "if", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x05, name: "else", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x06, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x07, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x08, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x09, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x0a, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x0b, name: "end", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x0c, name: "br", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x0d, name: "br_if", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x0e, name: "br_table", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x0f, name: "return", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x10, name: "call", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x11, name: "call_indirect", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x12, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x13, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x14, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x15, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x16, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x17, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x18, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x19, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x1a, name: "drop", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x1b, name: "select", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x1c, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x1d, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x1e, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x1f, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x20, name: "local.get", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x21, name: "local.set", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x22, name: "local.tee", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x23, name: "global.get", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x24, name: "global.set", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x25, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x26, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x27, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x28, name: "i32.load", in_type: Prim::Void, out_type: Prim::I32, has_const: true, takes_align: true},
    InstrInfo{instr: 0x29, name: "i64.load", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x2a, name: "f32.load", in_type: Prim::Void, out_type: Prim::F32, has_const: true, takes_align: true},
    InstrInfo{instr: 0x2b, name: "f64.load", in_type: Prim::Void, out_type: Prim::F64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x2c, name: "i32.load8_s", in_type: Prim::Void, out_type: Prim::I32, has_const: true, takes_align: true},
    InstrInfo{instr: 0x2d, name: "i32.load8_u", in_type: Prim::Void, out_type: Prim::I32, has_const: true, takes_align: true},
    InstrInfo{instr: 0x2e, name: "i32.load16_s", in_type: Prim::Void, out_type: Prim::I32, has_const: true, takes_align: true},
    InstrInfo{instr: 0x2f, name: "i32.load16_u", in_type: Prim::Void, out_type: Prim::I32, has_const: true, takes_align: true},
    InstrInfo{instr: 0x30, name: "i64.load8_s", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x31, name: "i64.load8_u", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x32, name: "i64.load16_s", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x33, name: "i64.load16_u", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x34, name: "i64.load32_s", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x35, name: "i64.load32_u", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x36, name: "i32.store", in_type: Prim::Void, out_type: Prim::I32, has_const: true, takes_align: true},
    InstrInfo{instr: 0x37, name: "i64.store", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x38, name: "f32.store", in_type: Prim::Void, out_type: Prim::F32, has_const: true, takes_align: true},
    InstrInfo{instr: 0x39, name: "f64.store", in_type: Prim::Void, out_type: Prim::F64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x3a, name: "i32.store8", in_type: Prim::Void, out_type: Prim::I32, has_const: true, takes_align: true},
    InstrInfo{instr: 0x3b, name: "i32.store16", in_type: Prim::Void, out_type: Prim::I32, has_const: true, takes_align: true},
    InstrInfo{instr: 0x3c, name: "i64.store8", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x3d, name: "i64.store16", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x3e, name: "i64.store32", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: true},
    InstrInfo{instr: 0x3f, name: "memory.size", in_type: Prim::Void, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0x40, name: "void", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x41, name: "i32.const", in_type: Prim::Void, out_type: Prim::I32, has_const: true, takes_align: false},
    InstrInfo{instr: 0x42, name: "i64.const", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: false},
    InstrInfo{instr: 0x43, name: "f32.const", in_type: Prim::Void, out_type: Prim::F32, has_const: true, takes_align: false},
    InstrInfo{instr: 0x44, name: "f64.const", in_type: Prim::Void, out_type: Prim::F64, has_const: true, takes_align: false},
    InstrInfo{instr: 0x45, name: "i32.eqz", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x46, name: "i32.eq", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x47, name: "i32.ne", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x48, name: "i32.lt_s", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x49, name: "i32.lt_u", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x4a, name: "i32.gt_s", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x4b, name: "i32.gt_u", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x4c, name: "i32.le_s", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x4d, name: "i32.le_u", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x4e, name: "i32.ge_s", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x4f, name: "i32.ge_u", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x50, name: "i64.eqz", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x51, name: "i64.eq", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x52, name: "i64.ne", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x53, name: "i64.lt_s", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x54, name: "i64.lt_u", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x55, name: "i64.gt_s", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x56, name: "i64.gt_u", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x57, name: "i64.le_s", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x58, name: "i64.le_u", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x59, name: "i64.ge_s", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x5a, name: "i64.ge_u", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x5b, name: "f32.eq", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x5c, name: "f32.ne", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x5d, name: "f32.lt", in_type: Prim::Void, out_type: Prim::F32, has_const: true, takes_align: false},
    InstrInfo{instr: 0x5e, name: "f32.gt", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x5f, name: "f32.le", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x60, name: "f32.ge", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x61, name: "f64.eq", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x62, name: "f64.ne", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x63, name: "f64.lt", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x64, name: "f64.gt", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x65, name: "f64.le", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x66, name: "f64.ge", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x67, name: "i32.clz", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x68, name: "i32.ctz", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x69, name: "i32.popcnt", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x6a, name: "i32.add", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x6b, name: "i32.sub", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x6c, name: "i32.mul", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x6d, name: "i32.div_s", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x6e, name: "i32.div_u", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x6f, name: "i32.rem_s", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x70, name: "i32.rem_u", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x71, name: "i32.and", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x72, name: "i32.or", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x73, name: "i32.xor", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x74, name: "i32.shl", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x75, name: "i32.shr_s", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x76, name: "i32.shr_u", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x77, name: "i32.rotl", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x78, name: "i32.rotr", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x79, name: "i64.clz", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x7a, name: "i64.ctz", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x7b, name: "i64.popcnt", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: false},
    InstrInfo{instr: 0x7c, name: "i64.add", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x7d, name: "i64.sub", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x7e, name: "i64.mul", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x7f, name: "i64.div_s", in_type: Prim::Void, out_type: Prim::I64, has_const: true, takes_align: false},
    InstrInfo{instr: 0x80, name: "i64.div_u", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x81, name: "i64.rem_s", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x82, name: "i64.rem_u", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x83, name: "i64.and", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x84, name: "i64.or", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x85, name: "i64.xor", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x86, name: "i64.shl", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x87, name: "i64.shr_s", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x88, name: "i64.shr_u", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x89, name: "i64.rotl", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x8a, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x8b, name: "f32.abs", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x8c, name: "f32.neg", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x8d, name: "f32.ceil", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x8e, name: "f32.floor", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x8f, name: "f32.trunc", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x90, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x91, name: "f32.sqrt", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x92, name: "f32.add", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x93, name: "f32.sub", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x94, name: "f32.mul", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x95, name: "f32.div", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x96, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x97, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0x98, name: "f32.copysign", in_type: Prim::Void, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0x99, name: "f64.abs", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x9a, name: "f64.neg", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x9b, name: "f64.ceil", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x9c, name: "f64.floor", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x9d, name: "f64.trunc", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x9e, name: "f64.nearest", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0x9f, name: "f64.sqrt", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xa0, name: "f64.add", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xa1, name: "f64.sub", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xa2, name: "f64.mul", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xa3, name: "f64.div", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xa4, name: "f64.min", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xa5, name: "f64.max", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xa6, name: "f64.copysign", in_type: Prim::Void, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xa7, name: "i32.wrap_i64", in_type: Prim::I64, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xa8, name: "i32.trunc_f32_s", in_type: Prim::F32, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xa9, name: "i32.trunc_f32_u", in_type: Prim::F32, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xaa, name: "i32.trunc_f64_s", in_type: Prim::F64, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xab, name: "i32.trunc_f64_u", in_type: Prim::F64, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xac, name: "i64.extend_i32_s", in_type: Prim::I32, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xad, name: "i64.extend_i32_u", in_type: Prim::I32, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xae, name: "i64.trunc_f32_s", in_type: Prim::F32, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xaf, name: "i64.trunc_f32_u", in_type: Prim::F32, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xb0, name: "i64.trunc_f64_s", in_type: Prim::F64, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xb1, name: "i64.trunc_f64_u", in_type: Prim::F64, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xb2, name: "f32.convert_i32_s", in_type: Prim::I32, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xb3, name: "f32.convert_i32_u", in_type: Prim::I32, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xb4, name: "f32.convert_i64_s", in_type: Prim::I64, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xb5, name: "f32.convert_i64_u", in_type: Prim::I64, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xb6, name: "f32.demote_f64", in_type: Prim::F64, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xb7, name: "f64.convert_i32_s", in_type: Prim::I32, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xb8, name: "f64.convert_i32_u", in_type: Prim::I32, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xb9, name: "f64.convert_i64_s", in_type: Prim::I64, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xba, name: "f64.convert_i64_u", in_type: Prim::I64, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xbb, name: "f64.promote_f32", in_type: Prim::F32, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xbc, name: "i32.reinterpret_f32", in_type: Prim::F32, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xbd, name: "i64.reinterpret_f64", in_type: Prim::F64, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xbe, name: "f32.reinterpret_i32", in_type: Prim::I32, out_type: Prim::F32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xbf, name: "f64.reinterpret_i64", in_type: Prim::I64, out_type: Prim::F64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xc0, name: "i32.extend8_s", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xc1, name: "i32.extend16_s", in_type: Prim::Void, out_type: Prim::I32, has_const: false, takes_align: false},
    InstrInfo{instr: 0xc2, name: "i64.extend8_s", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xc3, name: "i64.extend16_s", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xc4, name: "i64.extend32_s", in_type: Prim::Void, out_type: Prim::I64, has_const: false, takes_align: false},
    InstrInfo{instr: 0xc5, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xc6, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xc7, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xc8, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xc9, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xca, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xcb, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xcc, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xcd, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xce, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xcf, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xd0, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xd1, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xd2, name: "ref.func", in_type: Prim::FuncIdx, out_type: Prim::Void, has_const: true, takes_align: false},
    InstrInfo{instr: 0xd3, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xd4, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xd5, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xd6, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xd7, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xd8, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xd9, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xda, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xdb, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xdc, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xdd, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xde, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xdf, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xe0, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xe1, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xe2, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xe3, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xe4, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xe5, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xe6, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xe7, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xe8, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xe9, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xea, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xeb, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xec, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xed, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xee, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xef, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xf0, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xf1, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xf2, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xf3, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xf4, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xf5, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xf6, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xf7, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xf8, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xf9, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xfa, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xfb, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xfc, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xfd, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xfe, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
    InstrInfo{instr: 0xff, name: "", in_type: Prim::Void, out_type: Prim::Void, has_const: false, takes_align: false},
];


pub fn get_instr(name: &str) -> Option<InstrInfo> {
    INSTRS.iter().find(|x| {x.name == name}).map(|x| {*x})
}


// pub fn build_expr(segments: Vec<ExprSeg>) -> WasmExpr {
//     let mut expr = Vec::<u8>::new();
//     for segment in segments {
//         match segment {
//             ExprSeg::Expr(e) => {
//                 expr.append(&mut e.expr.clone())
//             }
//             ExprSeg::Instr(name) => {
//                 if let Some(instr) = get_instr(name) {
//                     expr.push(instr.instr);
//                 }
//             }
//             ExprSeg::Num(mut n) => {
//                 while n > 0 {
//                     let low_byte = (n & 0x7f) | if n >> 7 == 0 {
//                         0
//                     } else {
//                         0x80
//                     };
//                     expr.push(low_byte as u8);
//                     n >>= 7;
//                 }
//             }
//             _ => {}
//         }
//     }
//     WasmExpr{expr}
// }


// pub fn build_expr_func(segments: Vec<ExprSeg>) -> Vec<u8> {
//     segments.iter().flat_map(|segment| {
//         match segment {
//             ExprSeg::Expr(e) => {
//                  e.expr.clone()
//             }
//             ExprSeg::Instr(name) => {
//                 get_instr(name.to_string())
//                     .map(|instr| vec![instr.instr])
//                     .unwrap_or(vec![])
//             }
//             ExprSeg::Num(mut n) => {
//                 let mut num_vec = Vec::<u8>::new();
//                 while n > 0 {
//                     let low_byte = (n & 0x7f) | if n >> 7 == 0 {
//                         0
//                     } else {
//                         0x80
//                     };
//                     num_vec.push(low_byte as u8);
//                     n >>= 7;
//                 }
//                 num_vec
//             }
//         }
//     }).collect()
// }