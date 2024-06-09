use std::io::Read;

use crate::wasm_model::*;
use std::str::from_utf8;

pub fn type_to_str(wasm_type: WasmTypeAnnotation) -> String {
    match wasm_type {
        // f64
        WasmTypeAnnotation { _type: 0x7c } => "f64",
        // f32
        WasmTypeAnnotation { _type: 0x7d } => "f32",
        // i64
        WasmTypeAnnotation { _type: 0x7e } => "i64",
        // i32
        WasmTypeAnnotation { _type: 0x7f } => "i32",
        // funcref
        WasmTypeAnnotation { _type: 0x70 } => "funcref",
        _ => ""
    }.to_string()
} 

pub fn vec_to_string(vec: Vec<u8>) -> String {
    let mut str = "".to_string();
    for u in vec.clone() {
        str = str + (u as char).to_string().as_str();
    }
    str
}

pub fn indent(s: String, indent: u32) -> String {
    let indentation = (0..indent).fold("".to_string(), |acc, _| ("  ".to_string() + acc.as_str()));
    let indented = indentation.clone() + &s.replace("\n", &("\n".to_string() + &indentation));
    indented.strip_suffix(&indentation).unwrap_or(&indented).to_string()
}

pub fn sig_to_import_wat(f: &WasmFunctionType) -> String {
    let mut wat: String = "".to_string();
    if f.num_params != 0 {
        wat += "(param ";
        wat += f.params.clone().iter().map(|x| type_to_str(*x)).collect::<Vec<String>>().join(" ").as_str();
        wat += ")";
        if f.num_results != 0 {
            wat += " ";
        }
    }

    if f.num_results != 0 {
        wat += "(result ";
        wat += f.results.clone().iter().map(|x| type_to_str(*x)).collect::<Vec<String>>().join(" ").as_str();
        wat += ")";
    }
    wat
}

pub fn sig_to_wat(f: &WasmFunctionType) -> (usize, String) {
    let mut wat: String = "".to_string();
    if f.num_params != 0 {
        wat += f.params.clone().iter().enumerate()
                               .map(|(i, x)| format!("(param $var{} {})", i, type_to_str(*x))).collect::<Vec<String>>().join(" ").as_str();
        if f.num_results != 0 {
            wat += " ";
        }
    }

    if f.num_results != 0 {
        wat += "(result ";
        wat += f.results.clone().iter().map(|x| type_to_str(*x)).collect::<Vec<String>>().join(" ").as_str();
        wat += ")";
    }
    (f.params.len(), wat)
}

pub fn emit_wat(wasm: WasmFile) -> String {
    let mut wat: String = "(module\n".to_string();
    for (i, import) in wasm.import_section_header.imports.iter().enumerate() {
        let type_str = match import.import_kind {
            WasmImportType::Func => "func",
            WasmImportType::Table => "table",
            WasmImportType::Mem => "mem",
            WasmImportType::Global => "global",
            _ => ""
        };
        wat += &indent(format!("({} ${}{} (import \"{}\" \"{}\") {})\n",
            type_str, type_str,
            i,
            vec_to_string(import.import_module_name.clone()),
            vec_to_string(import.import_field.clone()),
            sig_to_import_wat(wasm.get_import_sig(import))
        ), 1);
    }

    for (i, table) in wasm.table_section.tables.iter().enumerate() {
        wat += &indent(format!("(table {} {} {})\n", table.limits_initial, table.limits_max, type_to_str(WasmTypeAnnotation { _type: table.wasm_type})), 1);
    }

    for (i, export) in wasm.export_section.exports.iter().enumerate() {
        // TODO: Need to find list of these export kinds
        if export.export_kind == 0 {
            wat += &indent(format!("(export \"{}\" (func $func{}))\n", from_utf8(export.export_name.as_slice()).unwrap(), export.export_signature_index), 1);
        }
        if export.export_kind == 2 {
            wat += &indent(format!("(export \"{}\" (memory $memory{}))\n", from_utf8(export.export_name.as_slice()).unwrap(), export.export_signature_index), 1);
        }
    }

    for (i, elem) in wasm.elem_section.elems.iter().enumerate() {
        let reftype = match elem._type {
            WasmRefType::FuncRef => "funcref",
            WasmRefType::ExternRef => "externref",
        };

        match &elem.mode {
            WasmElemMode::Passive => {
                wat += &indent(format!("(elem $elem{:} {:} {:})\n", i, reftype, elem.init), 1);
            },
            WasmElemMode::Active(active_struct) => {
                wat += &indent(format!("(elem $elem{:} {:} {:} {:})\n", i, active_struct.offset_expr, reftype, elem.init), 1);

            },
            WasmElemMode::Declarative => {
                wat += &indent(format!("(elem $elem{:} {:} {:})\n", i, reftype, elem.init), 1);
            }
        }
    }

    for (i, func) in wasm.code_section.functions.iter().enumerate() {
        wat += &indent(format!("(func $func{:}", i), 1);
        let (num_params, t) = sig_to_wat(wasm.get_func_sig(i));
        wat += &t;
        wat += "\n";

        wat += &indent(func.locals.iter().enumerate()
                        .map(|(i, x)| format!("(local $var{} {})", i + num_params, &type_to_str(x._type)))
                        .reduce(|acc, s| (acc + " " + &s).to_string())
                        .unwrap_or("".to_string()).to_string() + "\n", 2);

        wat += &indent(format!("{:}\nend\n", func.body.emit_block_wat(blank_emitter()).1), 2);
        wat += &indent(format!(")\n"), 1);
    }

    for (_, data) in wasm.data_section.data_segs.iter().enumerate() {
        wat += &indent(format!("(data {:} {:})\n", data.header.expr, from_utf8(data.data.as_slice()).unwrap_or("<PARSE ERROR>")), 1);
    }

    wat += ")";

    wat
}

