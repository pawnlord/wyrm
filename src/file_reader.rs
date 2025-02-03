use std::{
    fmt::Debug,
    io::{Error, Read, ErrorKind},
    mem,
};

use crate::wasm_model::*;
use crate::instr_table::*;


fn read_global<T: Read + Debug>(state: &mut WasmDeserializeState<T>) -> Result<WasmGlobal, Error> {
    
    let wasm_type = state.read_sized(WasmTypeAnnotation { _type: 0 })?;
    let mutability = state.read_sized(0)?;
    let expr = state.read_expr()?.0;

    Ok(WasmGlobal {
        wasm_type,
        mutability,
        expr
    })
}

#[derive(Debug)]
struct WasmDeserializeState<T: Read + Debug> {
    buffer: T,
    raw_section: Vec<u8>,
    save_read: bool,
}

impl<T: Read + Debug> WasmDeserializeState<T> {

    fn start_raw_section(&mut self) {
        self.raw_section.clear();
        self.save_read = true;
    }

    
    fn end_raw_section(&mut self) {
        self.save_read = false;
    }

    // Read a value of known size
    fn read_sized<U: Sized + Clone>(&mut self, default: U) -> Result<U, Error> {
        let mut out: U = default.clone();
        // UNSAFE: This is used to make sure any data-type with a defined size is read
        // in the same way. The reasoning for unsafe is that this project is designed to crash on an error
        // anyway due to its purpose as a cli decompiler back-end.
        // This may be converted into something that checks for safety later if it is decided
        // that I want to use the same engine in another, less crash-oblivious program.
        unsafe {
            let buffer: &mut [u8] =
                std::slice::from_raw_parts_mut((&mut out as *mut U).cast(), mem::size_of::<U>());
            self.buffer.read_exact(buffer)?;
            if self.save_read {
                self.raw_section.append(buffer.to_vec().as_mut());
            }
        }

        return Ok(out);
    }

    // Read an int of dynamic size. See arcane_knowledge.md/#unsized_values for more information
    fn read_dynamic_uint(&mut self, default: usize) -> Result<usize, Error> {
        let mut out: usize = default.clone();
        let mut buffer: [u8; 1] = [0];

        self.buffer.read_exact(&mut buffer)?;
        
        if self.save_read {
            self.raw_section.append(buffer.to_vec().as_mut());
        }

        out += (buffer[0] & 0x7F) as usize;
        let mut bits = 7;
        while buffer[0] & 0x80 != 0 && bits < 64 {
            self.buffer.read_exact(&mut buffer)?;
        
            if self.save_read {
                self.raw_section.append(buffer.to_vec().as_mut());
            }

            out += ((buffer[0] & 0x7F & (!(1 << std::cmp::min(64 - bits, 7)))) as usize) << (7 * (bits / 7));
            bits += 7;
        }

        return Ok(out);
    }

    fn read_dynamic_int(&mut self, default: usize) -> Result<i64, Error> {
        let mut out: usize = default.clone();
        let mut buffer: [u8; 1] = [0];

        self.buffer.read_exact(&mut buffer)?;
        
        if self.save_read {
            self.raw_section.append(buffer.to_vec().as_mut());
        }

        out += (buffer[0] & 0x7F) as usize;
        let mut bits = 7;
        while buffer[0] & 0x80 != 0 && bits < 64 {
            self.buffer.read_exact(&mut buffer)?;

            out += ((buffer[0] & 0x7F & (std::cmp::min(64 - bits, 7))) as usize) << (7 * (bits / 7));
            bits += 7;
            
            if self.save_read {
                self.raw_section.append(buffer.to_vec().as_mut());
            }

        }
        
        if buffer[0] & 0x80 != 0 {
            return Err(Error::new(ErrorKind::InvalidData, "what"));
        }

        if buffer[0] & 0x40 != 0  && bits < 64{
            return Ok((out | (!0 << bits)) as i64);
        }

        return Ok(out as i64);

    }


    // Read in a vector of objects of which we know the size
    fn read_vector<U: Sized + Clone>(
        &mut self,
        default: U,
        num_elements: usize,
    ) -> Result<Vec<U>, Error> {
        let mut out = Vec::<U>::new();
        for _ in 0..num_elements {
            out.push(self.read_sized(default.clone())?);
        }
        return Ok(out);
    }

    // Read in a vector of objects of which we know the size
    fn read_vector_dynamic(
        &mut self,
        num_elements: usize,
    ) -> Result<Vec<usize>, Error> {
        let mut out = Vec::<usize>::new();
        for _ in 0..num_elements {
            out.push(self.read_dynamic_uint(0)?);
        }
        return Ok(out);
    }

    fn read_expr(&mut self) -> Result<(WasmExpr, Vec<u8>), Error>  {

        self.start_raw_section();

        let mut scope: Vec<Box<WasmExpr>> = vec![];
        let mut last_scope = WasmExpr::new_box();
        let mut expr_box = WasmExpr::new_box();
        let mut level: i32 = 0;
        let mut control_flow: Vec<InstrInfo> = Vec::new();
        while let Ok(byte) = self.read_sized::<u8>(0) {
            let info = INSTRS[byte as usize];
            let expr = &mut expr_box.expr_string;
            let mut instr_layout = vec![ExprSeg::Operation(info)];
            let special_case = get_edge_case(info);
            
            if special_case == SpecialInstr::BrTable {
                let num = self.read_dynamic_uint(0)?;
                let break_depths = self.read_vector_dynamic(num)?;
                let default = self.read_dynamic_uint(0)?;
                instr_layout.push(ExprSeg::BrTable(BrTableConst {
                    break_depths,
                    default
                }));
                expr.push(ExprSeg::Instr(instr_layout));
                continue;
            }

            if special_case == SpecialInstr::CallIndirect {
                instr_layout.push(ExprSeg::Int(self.read_dynamic_int(0)?));
                instr_layout.push(ExprSeg::Int(self.read_dynamic_int(0)?));
                expr.push(ExprSeg::Instr(instr_layout));
                continue;
            }

            if info.name == "" {
                println!("instruction not supported {:#x}", info.instr);
                todo!()
            }
            
            if info.takes_align {
                self.read_sized::<u8>(0)?;
            }
            
            for constant in info.constants {
                match constant {
                    Prim::F32 => {
                        instr_layout.push(ExprSeg::Float32(self.read_sized(0.0)?));
                    }
                    Prim::F64 => {
                        instr_layout.push(ExprSeg::Float64(self.read_sized(0.0)?));
                    }
                    Prim::Global => {
                        let num = self.read_dynamic_uint(0)?;
                        instr_layout.push(ExprSeg::Local(num));
                    }
                    Prim::Local => {
                        let num = self.read_dynamic_uint(0)?;
                        instr_layout.push(ExprSeg::Global(num));
                    }
                    Prim::Func => {
                        let num = self.read_dynamic_uint(0)?;
                        instr_layout.push(ExprSeg::Func(num));
                    }
                    Prim::Void => {
                        // void or align
                        let _num = self.read_sized::<u8>(0)?;
                    }
                    // Number
                    _ => {
                        let num = self.read_dynamic_int(0)?;
                        instr_layout.push(ExprSeg::Int(num));
                    }
                }
            }


            // Control flow is special when it comes to being an "instruction"
            if special_case == SpecialInstr::BeginBlock {
                control_flow.push(info);
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
                expr.push(ExprSeg::Operation(info));
                level -= 1;
                if level < 0 {
                    break;
                }
                // pop the scope
                let control_flow_context = control_flow.pop().unwrap();
                last_scope.expr_string.push(ExprSeg::ControlFlow(control_flow_context, expr_box, info));
                expr_box = last_scope;
                last_scope = scope.pop().unwrap();
                continue;
            }
            expr.push(ExprSeg::Instr(instr_layout));
        }
        self.end_raw_section();
        Ok((*expr_box, self.raw_section.clone()))

    }

    fn read_type_section(&mut self) -> Result<WasmTypeSection, Error> {
    
        // A section that describes the type signature of functions
        let mut type_section: WasmTypeSection = WasmTypeSection {
            section_size: self.read_dynamic_uint(0)?,
            num_types: self.read_dynamic_uint(0)?,
            function_signatures: Vec::new()
        };
    
        for _ in 0..type_section.num_types {
            let mut sig: WasmFunctionType = WasmFunctionType {
                func: 0,
                num_params: 0,
                params: Vec::new(),
                num_results: 0,
                results: Vec::new(),
            };
            sig.func = self.read_sized::<u8>(0)?;
            assert_eq!(sig.func, 0x60, "Function format was incorrect");
    
            sig.num_params = self.read_dynamic_uint(0)?;
            sig.params = self.read_vector(WasmTypeAnnotation { _type: 0 }, sig.num_params)?;
    
            sig.num_results = self.read_dynamic_uint(0)?;
            sig.results = self.read_vector(WasmTypeAnnotation { _type: 0 }, sig.num_results)?;
            type_section.function_signatures.push(sig)
        }
        Ok(type_section)
    }

    fn read_import_section(&mut self) -> Result<WasmImportSection, Error> {
        // A section containing a description of things imported from other sources.
        // Each import header has a name and a signature index
        let mut import_section_header: WasmImportSection = WasmImportSection {
            section_size: self.read_dynamic_uint(0)?,
            num_imports: self.read_dynamic_uint(0)?,
            imports: Vec::new()
        };

        for _ in 0..import_section_header.num_imports {
            let mut import: WasmImportHeader = WasmImportHeader {
                mod_name_length: 0,
                import_module_name: Vec::new(),
                import_field_len: 0,
                import_field: Vec::new(),
                import_kind: WasmImportType::Global,
                import_type: 0,
            };
            import.mod_name_length = self.read_dynamic_uint(0)?;
            import.import_module_name = self.read_vector(0, import.mod_name_length)?;
            import.import_field_len = self.read_dynamic_uint(0)?;
            import.import_field = self.read_vector(0, import.import_field_len)?;
            import.import_kind = num_to_import_type(self.read_sized(0)?);
            import.import_type = self.read_sized(0)?;
            import_section_header.imports.push(import);
        }
        Ok(import_section_header)
    }

    fn read_function_section(&mut self) -> Result<WasmFunctionSection, Error> { 
        let mut function_section: WasmFunctionSection = WasmFunctionSection {
            section_size: self.read_dynamic_uint(0)?,
            num_functions: self.read_dynamic_uint(0)?,
            function_signature_indexes: Vec::new()
        };
        for _ in 0..function_section.num_functions {
            function_section.function_signature_indexes.push(self.read_sized(0)?);
        }
        Ok(function_section)
    }

    fn read_table_section(&mut self) -> Result<WasmTableSection, Error> { 
        
        let mut table_section: WasmTableSection = WasmTableSection {
            section_size: self.read_dynamic_uint(0)?,
            num_tables: self.read_dynamic_uint(0)?,
            tables: Vec::new()
        };

        for _ in 0..table_section.num_tables {
            let mut table: WasmTable = WasmTable {
                wasm_type: 0,
                limits_flags: 0,
                limits_initial: 0,
                limits_max: 0,
            };
            table.wasm_type = self.read_sized::<u8>(0)?;
            table.limits_flags = self.read_sized::<u8>(0)?;
            table.limits_initial = self.read_dynamic_uint(0)?;
            table.limits_max = self.read_dynamic_uint(0)?;
            table_section.tables.push(table);
        }

        Ok(table_section)
    }
    
    fn read_memory_section(&mut self) -> Result<WasmMemorySection, Error> { 
        let mut memory_section: WasmMemorySection = WasmMemorySection {
            section_size: self.read_dynamic_uint(0)?,
            num_memories: self.read_dynamic_uint(0)?,
            memories: Vec::new()
        };
    
        for _ in 0..memory_section.num_memories {
            let mut memory: WasmMemoryStruct = WasmMemoryStruct {
                limits_flags: 0,
                limits_initial: 0,
                limits_max: 0,
            };
            memory.limits_flags = self.read_sized::<u8>(0)?;
            memory.limits_initial = self.read_dynamic_uint(0)?;
            memory.limits_max = self.read_dynamic_uint(0)?;
            memory_section.memories.push(memory);
        }
        Ok(memory_section)
    }
    
    fn read_global_section(&mut self) -> Result<WasmGlobalSection, Error> {     
        let mut global_section = WasmGlobalSection {
            section_size: self.read_dynamic_uint(0)?,
            num_globals: self.read_dynamic_uint(0)?,
            globals: Vec::new()
        };

        for _ in 0..global_section.num_globals {
            let global = read_global(self)?;
            global_section.globals.push(global);
        }
        Ok(global_section)
    }
    fn read_export_section(&mut self) -> Result<WasmExportSection, Error> { 
        
        let mut export_section: WasmExportSection = WasmExportSection {
            section_size: self.read_dynamic_uint(0)?,
            num_exports: self.read_dynamic_uint(0)?,
            exports: Vec::new()
        };

        for _ in 0..export_section.num_exports {
            let mut export: WasmExportHeader = WasmExportHeader {
                export_name_len: 0,
                export_name: Vec::new(),
                export_kind: 0,
                export_signature_index: 0,
            };
            export.export_name_len = self.read_dynamic_uint(0)?;
            export.export_name = self.read_vector(0, export.export_name_len)?;
            export.export_kind = self.read_sized(0)?;
            export.export_signature_index = self.read_sized(0)?;
            export_section.exports.push(export);
        }
        Ok(export_section)
    }

    fn create_elem_expr(ys: Vec<usize>) -> WasmExpr {
        let expr = ys.iter().flat_map(|y| {
            vec![
                ExprSeg::Operation(get_instr("ref.func").unwrap()),
                ExprSeg::Func(*y as usize),
                ExprSeg::Operation(get_instr("end").unwrap()),
            ]
        }).collect();

        WasmExpr {expr_string: expr}
    }

    fn read_elem(&mut self) -> Result<WasmElem, Error> {
        let id = self.read_sized::<u8>(0)? as u32;
        let _type: WasmRefType;
        let mut _init: WasmExpr;
        let _mode: WasmElemMode;
        match id {
            0 => {
                let e = self.read_expr()?.0;
                let y_size = self.read_dynamic_uint(0)?;
                let ys = self.read_vector_dynamic(y_size)?;
                Ok(WasmElem{
                    mode: WasmElemMode::Active(AcvtiveStruct {table: 0, offset_expr: e}),
                    _type: WasmRefType::FuncRef,
                    init: Self::create_elem_expr(ys),
                })
            }
            1 => {
                let et = self.read_sized::<u8>(0)?;
                let y_size = self.read_dynamic_uint(0)?;
                let ys = self.read_vector_dynamic(y_size)?;
                
                Ok(WasmElem{
                    mode: WasmElemMode::Passive,
                    _type: byte_to_reftype(et)?,
                    init: Self::create_elem_expr(ys),
                })
            }
            2 => {
                let x = self.read_dynamic_uint(0)?;
                let e = self.read_expr()?.0;
                let et = self.read_sized::<u8>(0)?;
                let y_size = self.read_dynamic_uint(0)?;
                let ys = self.read_vector_dynamic(y_size)?;

                Ok(WasmElem{
                    mode: WasmElemMode::Active(AcvtiveStruct {table: x as u32, offset_expr: e}),
                    _type: byte_to_reftype(et)?,
                    init: Self::create_elem_expr(ys),
                })
            }
            3 => {
                let et = self.read_sized::<u8>(0)?;
                let y_size = self.read_dynamic_uint(0)?;
                let ys = self.read_vector_dynamic(y_size)?;

                Ok(WasmElem{
                    mode: WasmElemMode::Declarative,
                    _type: byte_to_reftype(et)?,          
                    init: Self::create_elem_expr(ys)
                })               
            }
            4 => {
                todo!()
            }
            5 => {
                _mode = WasmElemMode::Passive;
                
                todo!()
            }
            6 => {
                todo!()
                
            }
            7 => {
                _mode = WasmElemMode::Declarative;

                todo!()
            }
            _ => {
                return Err(Error::new(ErrorKind::InvalidData, "Elem had invalid type"));
            }
        }
        

    }

    fn read_elem_section(&mut self) -> Result<WasmElemSection, Error> {
        
        let mut elem_section: WasmElemSection = WasmElemSection {
            section_size: self.read_dynamic_uint(0)?,
            num_elems: self.read_dynamic_uint(0)?,
            elems: Vec::new()
        };

        for _ in 0..elem_section.num_elems {
            elem_section.elems.push(self.read_elem()?);
        }
        Ok(elem_section)
    }
    
    fn read_data_count_section(&mut self) -> Result<WasmDataCountSection, Error> {
        Ok(WasmDataCountSection {
            section_size: self.read_dynamic_uint(0)?,
            datacount: self.read_dynamic_uint(0)?,
        })
    }

    fn read_locals(&mut self) -> Result<(Vec<WasmLocal>,  Vec<(u8, usize)>), Error> {
        let num_decs = self.read_dynamic_uint(0)?;
        
        let mut local_types = Vec::<(u8, usize)>::new();
        let mut locals = Vec::<WasmLocal>::new();
        for _ in 0..num_decs {
            let num_type = self.read_dynamic_uint(0)?;
            let _type = self.read_sized::<u8>(0)?;
            let mut locals_of_type = 
                (0.._type).map(
                    |_| WasmLocal { _type: WasmTypeAnnotation { _type } }
                ).collect();
            local_types.push((_type, num_type));
            locals.append(&mut locals_of_type);
        }

        Ok((locals, local_types))
    }

    fn read_function(&mut self) -> Result<WasmFunction, Error> {
        let size = self.read_dynamic_uint(0)?;
        let (locals, local_types) = self.read_locals()?;
        let body = self.read_expr()?; 
        Ok(WasmFunction{
            size,
            local_types,
            locals,
            body: body.0,
            raw_body: body.1.into_iter().map(|x| x as u64).collect()
        })        
    }
    
    fn read_code_section(&mut self) -> Result<WasmCodeSection, Error> {
        
        let mut code_section: WasmCodeSection = WasmCodeSection {
            section_size: self.read_dynamic_uint(0)?,
            num_functions: self.read_dynamic_uint(0)?,
            functions: vec![]
        };


        for _ in 0..code_section.num_functions {
            code_section.functions.push(self.read_function()?);
        }
        Ok(code_section)
    }
    fn read_data_section(&mut self) -> Result<WasmDataSection, Error> {
        
        let mut data_section: WasmDataSection = WasmDataSection {
            section_size: self.read_dynamic_uint(0)?,
            num_data_segs: self.read_dynamic_uint(0)?,
            data_segs: vec![]
        };


        for _ in 0..data_section.num_data_segs {
            let header_flags = self.read_sized::<u8>(0)?;
            let expr = self.read_expr()?.0;
            let data_size = self.read_dynamic_uint(0)?;
            let header = WasmDataSegHeader  { header_flags, expr, data_size };

            let data: Vec<u8> = self.read_vector(0, header.data_size)?;
            data_section.data_segs.push(WasmDataSeg {
                header,
                data
            });
        }
        Ok(data_section)
    }
}
// Reads a WASM file to a WasmFile struct.
pub fn wasm_deserialize(buffer: impl Read + Debug) -> Result<WasmFile, Error> {
    let mut state = WasmDeserializeState { buffer, raw_section: Vec::new(), save_read: false };
    let mut wasm_header: WasmHeader = WasmHeader {
        magic_number: 0,
        version: 0,
    };

    // Assert that this is a WASM file by checking it's header
    wasm_header.magic_number = state.read_sized::<u32>(0)?;
    // TODO: Figure out the difference between versions.
    // Most WASM is version 1 so we focus on that for now.
    wasm_header.version = state.read_sized::<u32>(0)?;
    assert_eq!(
        wasm_header.magic_number, 0x6d736100,
        "Magic number was incorrect"
    );


    let mut type_section = WasmTypeSection{
        section_size: 0,
        num_types: 0,
        function_signatures: Vec::new(),
    };
    let mut import_section_header = WasmImportSection {
        section_size: 0,
        num_imports: 0,
        imports: Vec::new(),
    };
    let mut function_section = WasmFunctionSection {
        section_size: 0,
        num_functions: 0,
        function_signature_indexes: Vec::new(),
    };
    let mut table_section = WasmTableSection {
        section_size: 0,
        num_tables: 0,
        tables: Vec::new(),
    };
    let mut memory_section = WasmMemorySection {
        section_size: 0,
        num_memories: 0,
        memories: Vec::new(),
    };
    let mut global_section = WasmGlobalSection {
        section_size: 0,
        num_globals: 0,
        globals: Vec::new(),
    };
    let mut export_section = WasmExportSection{
        section_size: 0,
        num_exports: 0,
        exports: Vec::new(),
    };
    let mut elem_section = WasmElemSection{
        section_size: 0,
        num_elems: 0,
        elems: Vec::new(),
    };
    let mut data_count_section = WasmDataCountSection {
        section_size: 0,
        datacount: 0
    };
    let mut code_section = WasmCodeSection { 
        section_size: 0, 
        num_functions: 0, 
        functions: vec![] 
    };
    let mut data_section = WasmDataSection { 
        section_size: 0, 
        num_data_segs: 0, 
        data_segs: vec![] 
    };

    while let Ok(section_type) = state.read_sized::<u8>(0) {
        match section_type {
            0x01 => println!(" = state.read_type_section()?"),
            0x02 => println!(" = state.read_import_section()?"),
            0x03 => println!(" = state.read_function_section()?"),
            0x04 => println!(" = state.read_table_section()?"),
            0x05 => println!(" = state.read_memory_section()?"),
            0x06 => println!(" = state.read_global_section()?"),
            0x07 => println!(" = state.read_export_section()?"),
            0x09 => println!(" = state.read_elem_section()?"),
            0x0a => println!(" = state.read_code_section()?"),
            0x0b => println!(" = state.read_data_section()?"),
            0x0c => println!(" = state.read_data_count_section()?"),
            _ => {
                break
            }
        }

        match section_type {
            0x01 => type_section = state.read_type_section()?,
            0x02 => import_section_header = state.read_import_section()?,
            0x03 => function_section = state.read_function_section()?,
            0x04 => table_section = state.read_table_section()?,
            0x05 => memory_section = state.read_memory_section()?,
            0x06 => global_section = state.read_global_section()?,
            0x07 => export_section = state.read_export_section()?,
            0x09 => elem_section = state.read_elem_section()?,
            0x0a => code_section = state.read_code_section()?,
            0x0b => data_section = state.read_data_section()?,
            0x0c => data_count_section = state.read_data_count_section()?,
            _ => {
                break
            }
        }
    }

    return Ok(WasmFile {
        wasm_header,
        type_section,
        import_section_header,
        function_section,
        table_section,
        memory_section,
        global_section,
        export_section,
        elem_section, 
        code_section,
        data_section,
        data_count_section,
    });
}
