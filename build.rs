use std::{collections::HashMap, env, fs::File, io::{BufReader, BufWriter, Write}};

use serde_json::Value;


fn value_to_type_signature(value: Value) -> String {

    let json_sig: Vec<String> = value.as_array().unwrap().iter()
        .map(|x| format!("Prim::{}", x.as_str().unwrap())).collect();

    let sig = json_sig.join(", ");

    return format!("&[{}]", sig);
}


fn main() {
    println!("cargo::rerun-if-changed=instr_table.json");
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut extensions_enabled = false;
    
    // Core part of instruction set, pretty much 0-256
    let core = vec!["CONTROL_OPCODE", "MISC_OPCODE", "LOAD_MEM_OPCODE", "STORE_MEM_OPCODE", 
        "MISC_MEM_OPCODE", "SIMPLE_EXTENDED_CONST_OPCODE", "SIMPLE_NON_CONST_OPCODE",
        "ASMJS_COMPAT_OPCODE"];
    
    let mut core: Vec<String> = core.iter().map(|x| x.to_string()).collect();

    {
        // Config letting us know what we want to be using
        // There are a few wasm extensions, don't really want to go through them all right now
        // so this is here
        // No config defaults to a simple set for the core, and no extensions enabled
        let config = File::open("instr_parser.json");
        if let Ok(config) = config {
            let reader = BufReader::new(config);
            
            let config: Value = serde_json::from_reader(reader)
                .expect("Failed to open instr_parser.json!!");
            
            extensions_enabled = config["extensions"].as_bool().unwrap();

            core = config["core"].as_array().unwrap().iter()
                    .map(|x| x.as_str().unwrap().to_string()).collect::<Vec<String>>();
        }
    }

    let instructions: Value = {
        let Ok(f) = File::open("instr_table.json") else {
            // If we don't have anything, just write an empty array
            // The build will still happen, but it won't be usable
            println!("cargo::warning=No instruction table provided, minimal data being written");
            let f = File::open(out_dir + "/instr_table.rs")
                .expect("Could not open ouput instr_table.rs!!");
            let mut writer = BufWriter::new(f);
            let _ = writer.write("
                use crate::wasm_model::*;

                pub const INSTRS: [InstrInfo; 0] = []
            ".as_bytes());

            return;
        };

        let reader = BufReader::new(f);
        serde_json::from_reader(reader)
            .expect("Failed to open instr_table.json!!")
    };

    // collect instructions
    let mut instr_list: HashMap<i64, Value> = HashMap::new();
    for sec in core {
        let sec_instrs = instructions[sec].as_object().unwrap();
        for (_name, instr) in sec_instrs {
            instr_list.insert(instr["opcode"].as_i64().unwrap(), instr.clone());
        }
    }

    if extensions_enabled {
        todo!("WASM Extensions not yet supported");
    }

    {
        let f = File::create(out_dir + "/instr_table.rs")
            .expect("Could not open ouput instr_table.rs!!");

        let mut writer = BufWriter::new(f);

        let mut instruction_list = "".to_string();

        for i in 0..=255 {
            instruction_list += "\n";            
            let instr = instr_list.get(&i).clone();
            if instr.is_none() {
                // No instruction, but we need to keep the instructions in order...
                let instr_string = format!(
                r#"    InstrInfo{{instr: {:#x}, name: "", in_types: &[], out_types: &[], constants: &[], takes_align: false}},"#, i);
                instruction_list += instr_string.as_str();                
                continue;
            }

            
            // Instruction table JSON format is 
            // <section> : { <instruction name> : {"name": <name>, 
            //                  signature: [<in>, <out>, <constants>],
            //                  opcode: <opcode>
            //                },...
            //              }, ...
            let instr = instr.unwrap();

            let name = instr["name"].as_str().unwrap();

            let signature = instr["signature"].as_array().unwrap();
            let in_types = value_to_type_signature(signature[0].clone());
            let out_types = value_to_type_signature(signature[1].clone());
            let constants = value_to_type_signature(signature[2].clone());
            
            // TODO: find takes_align information
            let takes_align = false;


            let instr_string = format!(
                r#"    InstrInfo{{instr: {:#x}, name: "{}", in_types: {}, out_types: {}, constants: {}, takes_align: {}}},"#,
                i, name,
                in_types, out_types, constants,
                takes_align);
            instruction_list += instr_string.as_str();                   
        }

        let mut symbols = "".to_string();        
        let mut all_symbols = Vec::<String>::new();
        let mut stack_nops = Vec::<String>::new();
        // Things that change the stack, but not up or down: effects what's on top
        let mut stack_push1 = Vec::<String>::new();
        let mut stack_pop1 = Vec::<String>::new();
        let mut stack_push2 = Vec::<String>::new();
        for (_i, instr) in instr_list {
            let name = instr["name"].as_str().unwrap().to_string();
            let opcode = instr["opcode"].as_u64().unwrap();
            
            let normalized_ident = name.replace(".", "_").replace("@", "_").to_uppercase();

            let instr_string = "#[allow(dead_code)]\n".to_string() + format!(r#"const {}: u64 = {};"#, normalized_ident, opcode.to_string()).as_str() + "\n";
            symbols += instr_string.as_str();

            all_symbols.push(normalized_ident);
            let signature = instr["signature"].as_array().unwrap();
            let in_types = signature[0].clone().as_array().unwrap();
            let out_types = signature[1].clone().as_array().unwrap();
            // if in_types.len() == 0 && out_types.len() == 0 {
            //     // Does not effect the stack
            //     stack_nops.push(normalized_ident);
            // }
            // if out_types.len() == 1 {
            //     stack_push1.push(normalized_ident);
            // } 
            // if in_types.len() == 1 {
            //     stack_pop1.push(normalized_ident);
            // } 
            // if out_types.len() == 2 {
            //     stack_push2.push(normalized_ident);
            // } 
        }
        
        let num_symbols = all_symbols.len();
        let all_symbols = "[&[".to_string() + all_symbols.join("], &[").as_str() + "]]";

        let _ = writer.write(format!("pub const INSTRS: [InstrInfo; 256] = [{}];
                                            {}
                                            const all_symbols: [&[u64]; {}] = {};
                                            ", instruction_list,
                                            symbols,
                                            num_symbols,
                                            all_symbols).as_bytes());


    }
}