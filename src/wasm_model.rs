use core::fmt;
use std::{
    fmt::{Debug, Display, Formatter},
    io::{Error, ErrorKind},
    slice::Iter,
};

use crate::{
    instr_table::*,
    usdm::{SpecialStackOp, StackOperation, UsdmFrontend, UsdmSegment},
    parser::prs
};

pub trait TypeTrait {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prim {
    Void,
    I32,
    I64,
    F32,
    F64,
    Local,
    Global,
    Generic,
    Func,
}

#[derive(Debug, Clone)]
pub struct BrTableConst {
    pub break_depths: Vec<usize>,
    pub default: usize,
}

/*
 * An IdiomPattern is made from the IdiomGrammar, and when found
 * creates a WasmIdiom
*/
#[derive(Debug, Clone)]
pub enum IdiomGrammar {
    StrictExpr(WasmExpr),
    // Single-segment wildcard
    InstrWildcard,
    // Multi-segment wildcard
    ExprWildcard,
}

#[derive(Debug, Clone)]
pub struct WasmIdiomPattern {
    pub pattern: Vec<IdiomGrammar>,
    pub idiom: WasmIdiom,
}

impl WasmIdiomPattern {
    fn independent_expr(expr: WasmExpr, idiom: WasmIdiom) -> Self {
        let expr = expr.parse_string().expect("Error parsing Idiom pattern");
        Self {
            pattern: vec![IdiomGrammar::StrictExpr(expr)],
            idiom,
        }
    }

    // An example of an idiom, will research more later
    pub fn double() -> Self {
        Self::independent_expr(
            new_expr(vec![
                get_op_seg("i32.const"),
                ExprSeg::Int(1),
                get_op_seg("i32.shl"),
            ]),
            WasmIdiom::Double,
        )
    }
}

#[derive(Debug, Clone)]
pub enum WasmIdiom {
    Double,
}

#[derive(Debug, Clone)]
pub enum ExprSeg {
    Operation(InstrInfo),
    ControlFlow(InstrInfo, Box<WasmExpr>, InstrInfo),
    // Raw bits of an int, signage and other things are figured out later (all ints are stored in the same manner)
    Int(i64),
    Float32(f32),
    Float64(f64),
    Local(usize),
    Global(usize),
    Func(usize),
    BrTable(BrTableConst),
    Instr(Vec<ExprSeg>),
    // TODO: Parse for idioms
    Idiom(WasmIdiom),
}

impl ExprSeg {
    pub fn emit_wat(&self, mut wat: String, state: EmitterState) -> String {
        match self {
            ExprSeg::Operation(info) => {
                wat += format!("{:}", info.name).as_str();
            }
            ExprSeg::Int(i) => {
                wat += format!("{:}", i).as_str();
            }
            ExprSeg::Float32(f) => {
                wat += format!("{:}", f).as_str();
            }
            ExprSeg::Float64(f) => {
                wat += format!("{:}", f).as_str();
            }
            ExprSeg::Global(idx) => {
                wat += format!("$global{:}", idx).as_str();
            }
            ExprSeg::Local(idx) => {
                wat += format!("$var{:}", idx).as_str();
            }
            ExprSeg::Func(idx) => {
                wat += format!("$func{:}", idx).as_str();
            }
            ExprSeg::BrTable(_table_const) => {}
            ExprSeg::ControlFlow(info, expr, end_info) => {
                // Add extra characters for indentation
                wat += &format!("{:} $label{}\n  ", info.name, state.label);

                let (_, new_emit) = expr.emit_block_wat(EmitterState {
                    start_segment: 0,
                    label: state.label + 1,
                });

                wat += new_emit.replace("\n", "\n  ").as_str();
                wat += format!("\n{:} $label{}\n", end_info.name, state.label).as_str();
            }
            ExprSeg::Instr(instr_expr) => {
                for seg in instr_expr {
                    wat = seg.emit_wat(wat, state) + " ";
                }
                wat = wat + "\n";
            }
            _ => {}
        }
        wat
    }
}

impl UsdmSegment for ExprSeg {
    type Type = Prim;

    fn get_stack_operation(&self) -> StackOperation<Self> {
        match self {
            Self::Operation(info) => info.get_stack_operation(),
            Self::Instr(segs) => {
                if segs.len() == 0 {
                    return StackOperation::new();
                }

                let Self::Operation(info) = segs[0] else {
                    return StackOperation::new();
                };

                info.get_stack_operation()
            }
            _ => StackOperation::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WasmExpr {
    pub expr_string: Vec<ExprSeg>,
}

#[derive(Clone, Copy)]
pub struct EmitterState {
    start_segment: usize,
    label: usize,
}

pub fn blank_emitter() -> EmitterState {
    return EmitterState {
        start_segment: 0,
        label: 0,
    };
}

impl WasmExpr {
    pub fn new_box() -> Box<Self> {
        Box::new(Self {
            expr_string: vec![],
        })
    }

    fn parse_error() -> Error {
        Error::new(
            ErrorKind::InvalidData,
            "WasmExpr::parse_string: Error while parsing, likely data was misordered",
        )
    }

    // This really needs to be cleaned up
    pub fn parse_string(&self) -> Result<WasmExpr, Error> {
        let mut scope: Vec<Box<WasmExpr>> = vec![];
        let mut last_scope = WasmExpr::new_box();
        let mut expr_box = WasmExpr::new_box();
        let mut level: i32 = 0;
        let mut control_flow: Vec<InstrInfo> = Vec::new();
        let mut iter = self.expr_string.iter();
        while let Some(seg) = iter.next() {
            let info = match seg {
                ExprSeg::Operation(info) => info,
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "WasmExpr::parse_string: Expected instruction, didn't find one",
                    ));
                }
            };

            let expr = &mut expr_box.expr_string;
            let mut instr_layout = vec![ExprSeg::Operation(*info)];
            let special_case = get_edge_case(*info);

            if special_case == SpecialInstr::BrTable {
                // Likely to crash, oh well
                let br_table = iter.next().unwrap();
                instr_layout.push(br_table.clone());
                expr.push(ExprSeg::Instr(instr_layout));
                continue;
            }

            if special_case == SpecialInstr::CallIndirect {
                instr_layout.push(iter.next().unwrap().clone());
                instr_layout.push(iter.next().unwrap().clone());
                expr.push(ExprSeg::Instr(instr_layout));
                continue;
            }

            for _ in info.constants {
                instr_layout.push(iter.next().unwrap().clone());
            }

            // Control flow is special when it comes to being an "instruction"
            if special_case == SpecialInstr::BeginBlock {
                control_flow.push(*info);
                level += 1;
                // Push the scope
                scope.push(last_scope);
                last_scope = expr_box;
                // Create a new scope
                expr_box = WasmExpr::new_box();
                continue;
            }

            if special_case == SpecialInstr::EndBlock {
                // TODO: This is dirty, change later
                expr.push(ExprSeg::Operation(*info));
                level -= 1;
                if level < 0 {
                    break;
                }
                // pop the scope
                let control_flow_context = control_flow.pop().unwrap();
                last_scope.expr_string.push(ExprSeg::ControlFlow(
                    control_flow_context,
                    expr_box,
                    *info,
                ));
                expr_box = last_scope;
                last_scope = scope.pop().unwrap();
                continue;
            }
            expr.push(ExprSeg::Instr(instr_layout));
        }

        Ok(*expr_box)
    }

    pub fn emit_block_wat(&self, state: EmitterState) -> (usize, String) {
        let mut wat = "".to_string();
        let mut emit_until = 0;

        for (i, seg) in self
            .expr_string
            .iter()
            .skip(state.start_segment)
            .enumerate()
        {
            match seg {
                ExprSeg::Operation(info) => {
                    let special_case = get_edge_case(*info);

                    if special_case == SpecialInstr::EndBlock {
                        return (state.start_segment + i, wat);
                    }

                    wat += format!("{:}", info.name).as_str();

                    // Figure out how many expression segments come after this one
                    emit_until = if special_case == SpecialInstr::CallIndirect {
                        2
                    } else if special_case == SpecialInstr::BeginBlock {
                        1
                    } else if info.constants.len() != 0 {
                        1
                    } else {
                        0
                    }
                }
                ExprSeg::ControlFlow(info, expr, end_info) => {
                    // Add extra characters for indentation
                    wat += &format!("{} $label{}\n  ", info.name, state.label);

                    let (_, new_emit) = expr.emit_block_wat(EmitterState {
                        start_segment: 0,
                        label: state.label + 1,
                    });

                    wat += new_emit.replace("\n", "\n  ").trim_end();
                    wat += format!("\n{:} $label{}\n", end_info.name, state.label).as_str();
                }
                _ => {
                    wat = seg.emit_wat(wat, state);
                }
            }

            if emit_until > 0 {
                wat += " ";
                emit_until -= 1;
            }
        }
        (self.expr_string.len() - 1, wat)
    }

    pub fn emit_expression_wat(&self) -> String {
        let mut wat = "".to_string();
        let mut i = 0;
        while i < self.expr_string.len() - 1 {
            wat += "(";
            let new_emit: String;
            (i, new_emit) = self.emit_block_wat(EmitterState {
                start_segment: i,
                label: 0,
            });
            i += 1;
            wat += new_emit.trim_end();
            wat += ") ";
        }
        // Remove extraneous space
        wat.pop();

        wat
    }
}

impl Display for WasmExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&self.emit_expression_wat().as_str())
    }
}

fn get_op_seg(name: &str) -> ExprSeg {
    ExprSeg::Operation(get_instr(name).unwrap())
}

fn new_expr(expr_string: Vec<ExprSeg>) -> WasmExpr {
    // TODO: Parse String into instrs
    WasmExpr { expr_string }
}

impl From<Vec<ExprSeg>> for WasmExpr {
    fn from(expr_string: Vec<ExprSeg>) -> Self {
        Self { expr_string }
    }
}

impl UsdmFrontend for WasmExpr {
    type Type = Prim;
    type Segment = ExprSeg;
    type SegmentIterator<'a> = Iter<'a, ExprSeg>;
    fn iter<'a>(&'a self) -> Self::SegmentIterator<'a> {
        self.expr_string.iter()
    }
}

pub fn type_values(t: Prim) -> (i32, String) {
    match t {
        Prim::Void => (0, "void".to_string()),
        Prim::I32 => (1, "i32".to_string()),
        Prim::I64 => (2, "i64".to_string()),
        Prim::F32 => (3, "f32".to_string()),
        Prim::F64 => (4, "f64".to_string()),
        Prim::Local => (5, "local".to_string()),
        Prim::Global => (6, "global".to_string()),
        Prim::Func => (7, "funcidx".to_string()),
        Prim::Generic => (8, "generic".to_string()),
    }
}

#[derive(Debug)]
pub struct WasmHeader {
    pub magic_number: u32,
    pub version: u32,
}

// Section containing function types?
#[derive(Debug)]
pub struct WasmTypeSection {
    pub section_size: usize,
    pub num_types: usize,
    pub function_signatures: Vec<WasmFunctionType>,
}

// Type field for function signatures
#[derive(Debug, Clone, Copy)]
pub struct WasmTypeAnnotation {
    pub _type: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum WasmTypedData {
    Void,
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

// Section containing the signature of a function
#[derive(Debug)]
pub struct WasmFunctionType {
    pub func: u8,
    pub num_params: usize,
    // of size num_params
    pub params: Vec<WasmTypeAnnotation>,
    pub num_results: usize,
    // of size num_results
    pub results: Vec<WasmTypeAnnotation>,
}

// Metadata about import section
#[derive(Debug)]
pub struct WasmImportSection {
    pub section_size: usize,
    pub num_imports: usize,
    pub imports: Vec<WasmImportHeader>,
}

#[derive(Debug, Clone, Copy)]
pub enum WasmImportType {
    Func,
    Table,
    Mem,
    Global,
}

pub fn num_to_import_type(num: u8) -> WasmImportType {
    match num {
        0x00 => WasmImportType::Func,
        0x01 => WasmImportType::Table,
        0x02 => WasmImportType::Mem,
        0x03 => WasmImportType::Global,
        _ => WasmImportType::Global,
    }
}

// Section describing imports
#[derive(Debug)]
pub struct WasmImportHeader {
    pub mod_name_length: usize,
    // of size mod_name_length
    pub import_module_name: Vec<u8>,
    pub import_field_len: usize,
    // of size emscripten_memcp_len
    pub import_field: Vec<u8>,
    pub import_kind: WasmImportType,
    pub import_type: u8,
}

#[derive(Debug)]
pub struct WasmFunctionSection {
    pub section_size: usize,
    pub num_functions: usize,
    pub function_signature_indexes: Vec<u8>,
}

#[derive(Debug)]
pub struct WasmTableSection {
    pub section_size: usize,
    pub num_tables: usize,
    pub tables: Vec<WasmTable>,
}
#[derive(Debug)]
pub struct WasmTable {
    pub wasm_type: u8,
    pub limits_flags: u8,
    pub limits_initial: usize,
    pub limits_max: usize,
}

#[derive(Debug)]
pub struct WasmMemorySection {
    pub section_size: usize,
    pub num_memories: usize,
    pub memories: Vec<WasmMemoryStruct>,
}

#[derive(Debug)]
pub struct WasmMemoryStruct {
    pub limits_flags: u8,
    pub limits_initial: usize,
    pub limits_max: usize,
}
#[derive(Debug, Clone)]
pub struct WasmGlobal {
    pub wasm_type: WasmTypeAnnotation,
    pub mutability: u8,
    pub expr: WasmExpr,
}

#[derive(Debug)]
pub struct WasmGlobalSection {
    pub section_size: usize,
    pub num_globals: usize,
    pub globals: Vec<WasmGlobal>,
}

#[derive(Debug)]
pub struct WasmExportSection {
    pub section_size: usize,
    pub num_exports: usize,
    pub exports: Vec<WasmExportHeader>,
}

#[derive(Debug)]
pub enum WasmRefType {
    FuncRef,
    ExternRef,
}

pub fn byte_to_reftype(byte: u8) -> Result<WasmRefType, Error> {
    match byte {
        0x70 => Ok(WasmRefType::FuncRef),
        0x6F => Ok(WasmRefType::ExternRef),

        _ => Err(Error::new(
            ErrorKind::InvalidData,
            format!("Invalid RefType byte: {:?}", byte),
        )),
    }
}

// Section describing imports
#[derive(Debug)]
pub struct WasmExportHeader {
    // of size emscripten_memcp_len
    pub export_name_len: usize,
    pub export_name: Vec<u8>,
    pub export_kind: u8,
    pub export_signature_index: u8,
}

#[derive(Debug)]
pub struct AcvtiveStruct {
    pub table: u32,
    pub offset_expr: WasmExpr,
}

#[derive(Debug)]
pub enum WasmElemMode {
    Passive,
    Active(AcvtiveStruct),
    Declarative,
}

#[derive(Debug)]
pub struct WasmElem {
    pub _type: WasmRefType,
    pub init: WasmExpr,
    pub mode: WasmElemMode,
}

#[derive(Debug)]
pub struct WasmElemSection {
    pub section_size: usize,
    pub num_elems: usize,
    pub elems: Vec<WasmElem>,
}

#[derive(Debug)]
pub struct WasmCodeSection {
    pub section_size: usize,
    pub num_functions: usize,
    pub functions: Vec<WasmFunction>,
}

#[derive(Debug)]
pub struct WasmDataSection {
    pub section_size: usize,
    pub num_data_segs: usize,
    pub data_segs: Vec<WasmDataSeg>,
}

#[derive(Debug)]
pub struct WasmDataSegHeader {
    pub header_flags: u8,
    pub expr: WasmExpr,
    pub data_size: usize,
}

#[derive(Debug)]
pub struct WasmDataSeg {
    pub header: WasmDataSegHeader,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct WasmDataCountSection {
    pub section_size: usize,
    pub datacount: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct WasmLocal {
    pub _type: WasmTypeAnnotation,
}

pub struct WasmFunction {
    pub size: usize,
    pub local_types: Vec<(u8, usize)>,
    pub locals: Vec<WasmLocal>,
    pub body: WasmExpr,
}

impl Debug for WasmFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmFunction")
            .field("size", &self.size)
            .field(
                "locals",
                &self
                    .local_types
                    .iter()
                    .map(|local_type| format!("[{:}; {:}]", local_type.0, local_type.1))
                    .collect::<Vec<String>>()
                    .join(", "),
            )
            .field("body", &self.body)
            .finish()
    }
}

#[derive(Debug)]
pub struct WasmFile {
    pub wasm_header: WasmHeader,
    pub type_section: WasmTypeSection,
    pub import_section_header: WasmImportSection,
    pub function_section: WasmFunctionSection,
    pub table_section: WasmTableSection,
    pub memory_section: WasmMemorySection,
    pub global_section: WasmGlobalSection,
    pub export_section: WasmExportSection,
    pub elem_section: WasmElemSection,
    pub code_section: WasmCodeSection,
    pub data_section: WasmDataSection,
    pub data_count_section: WasmDataCountSection,
}

#[derive(Clone, Copy)]
pub struct InstrInfo {
    // The opcode
    pub instr: u8,
    // The identifier
    pub name: &'static str,
    // What we take (non-immediates, from the stack)
    pub in_types: &'static [Prim],
    // What we output (to the stack)
    pub out_types: &'static [Prim],
    // What we take from the file
    pub constants: &'static [Prim],
    // Does an align byte follow?
    pub takes_align: bool,
}

impl InstrInfo {
    pub fn get_stack_operation(&self) -> StackOperation<ExprSeg> {
        let created_type = if self.instr == get_instr("i32.const").unwrap().instr {
            Prim::I32
        } else if self.instr == get_instr("i64.const").unwrap().instr {
            Prim::I64
        } else if self.instr == get_instr("f32.const").unwrap().instr {
            Prim::F32
        } else if self.instr == get_instr("f64.const").unwrap().instr {
            Prim::F64
        } else if self.instr == get_instr("ref.func").unwrap().instr {
            Prim::Func
        } else {
            Prim::Void
        };

        if created_type == Prim::Void {
            StackOperation {
                in_types: self.in_types.clone().to_vec(),
                out_types: self.out_types.clone().to_vec(),
                special_op: SpecialStackOp::<ExprSeg>::None,
            }
        } else {
            StackOperation {
                in_types: self.in_types.clone().to_vec(),
                out_types: self.out_types.clone().to_vec(),
                special_op: SpecialStackOp::<ExprSeg>::CreateVar(created_type),
            }
        }
    }
}

impl Debug for InstrInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let out_string: String = self
            .out_types
            .iter()
            .map(|prim_type| format!("{:?}", prim_type))
            .collect::<Vec<String>>()
            .join(", ");

        let rettype = if self.out_types.len() == 0 {
            "".to_string()
        } else {
            format!(" -> {:}", out_string)
        };

        let arg_string: String = self
            .in_types
            .iter()
            .map(|prim_type| format!("{:?}", prim_type))
            .collect::<Vec<String>>()
            .join(", ");

        let arg = if self.in_types.len() == 0 {
            "".to_string()
        } else {
            arg_string
        };

        let constant: String = self
            .constants
            .iter()
            .map(|prim_type| format!("{:?}", prim_type))
            .collect::<Vec<String>>()
            .join(", ");

        let constant = if self.constants.len() != 0 {
            format!("[{}]", constant)
        } else {
            constant
        };

        write!(
            f,
            "{:#x}: {}{}({}){}",
            self.instr, self.name, constant, arg, rettype
        )
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
        0x0b => SpecialInstr::EndBlock,
        0x11 => SpecialInstr::CallIndirect,
        _ => SpecialInstr::None,
    }
}

pub fn calc_dyn_size(mut i: i64) -> usize {
    let mut count = 1;
    if i == i64::MIN {
        i = i + 1;
    }
    i = i.abs();
    while i != 0 {
        i >>= 7;
        count += 1;
    }
    count
}

pub fn calculate_body_len(expr: &WasmExpr) -> usize {
    let mut total = 0;
    for seg in expr.expr_string.clone() {
        total += match seg {
            ExprSeg::Operation(_) => 1,
            ExprSeg::Int(i) => calc_dyn_size(i),
            ExprSeg::Float32(_) => 4,
            ExprSeg::Float64(_) => 8,
            ExprSeg::BrTable(tab) => {
                calc_dyn_size(tab.default as i64)
                    + tab
                        .break_depths
                        .iter()
                        .fold(0, |acc: usize, i| acc + calc_dyn_size(*i as i64))
            }
            ExprSeg::ControlFlow(_, block, _) => calculate_body_len(block.as_ref()),
            _ => 0,
        }
    }
    total
}

// Control symbols
pub const START: u64 = u64::MAX - 1;
pub const STMT: u64 = u64::MAX - 2;
pub const STMTS: u64 = u64::MAX - 3;
pub const INSTR: u64 = u64::MAX - 4;
pub const ADD_U64_OP: u64 = u64::MAX - 5;

impl prs::GrammarTrait for u64 {
    fn start_sym() -> Self {
        START
    }
}

impl WasmFile {
    pub fn get_import_sig(&self, import: &WasmImportHeader) -> &WasmFunctionType {
        &self.type_section.function_signatures[import.import_type as usize]
    }
    pub fn get_func_sig(&self, func: usize) -> &WasmFunctionType {
        let func_sig_idx = self.function_section.function_signature_indexes[func] as usize;
        &self.type_section.function_signatures[func_sig_idx]
    }
}
