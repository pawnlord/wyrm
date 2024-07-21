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
    let mut core = vec!["CONTROL_OPCODE", "MISC_OPCODE", "LOAD_MEM_OPCODE", "STORE_MEM_OPCODE", 
        "MISC_MEM_OPCODE", "SIMPLE_EXTENDED_CONST_OPCODE", "SIMPLE_NON_CONST_OPCODE",
        "ASMJS_COMPAT_OPCODE"];
    let mut core: Vec<String> = core.iter().map(|x| x.to_string()).collect();

    {
        let config = File::open("instr_parser.json");
        if let Ok(config) = config {
            let mut reader = BufReader::new(config);
            
            let config: Value = serde_json::from_reader(reader)
                .expect("Failed to open instr_parser.json!!");
            
            extensions_enabled = config["extensions"].as_bool().unwrap();

            core = config["core"].as_array().unwrap().iter()
                    .map(|x| x.as_str().unwrap().to_string()).collect::<Vec<String>>();
        }
    }

    let instructions: Value = {
        let Ok(f) = File::open("instr_table.json") else {
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
        for (name, instr) in sec_instrs {
            instr_list.insert(instr["opcode"].as_i64().unwrap(), instr.clone());
        }
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
                let instr_string = format!(
                r#"    InstrInfo{{instr: {:#x}, name: "", in_types: &[], out_types: &[], constants: &[], takes_align: false}},"#, i);
                instruction_list += instr_string.as_str();                
                continue;
            }
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
        
        
        let _ = writer.write(format!("pub const INSTRS: [InstrInfo; 256] = [{}];
        ", instruction_list).as_bytes());

    }
}