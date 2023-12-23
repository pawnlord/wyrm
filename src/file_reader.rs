use core::num;
use std::{
    default,
    fmt::Debug,
    fs::{self, File},
    io::{Error, Read, ErrorKind},
    mem, ptr,
};

#[derive(Debug)]
pub struct WasmHeader {
    magic_number: u32,
    version: u32,
}

// Section containing function types?
#[derive(Debug)]
pub struct WasmTypeSection {
    section_size: usize,
    num_types: usize,
    function_signatures: Vec<WasmFunctionType>,
}

// Type field for function signatures
#[derive(Debug, Clone, Copy)]
pub struct WasmTypeAnnotation {
    _type: u8,
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
    func: u8,
    num_params: usize,
    // of size num_params
    params: Vec<WasmTypeAnnotation>,
    num_results: usize,
    // of size num_results
    results: Vec<WasmTypeAnnotation>,
}

// Metadata about import section
#[derive(Debug)]
pub struct WasmImportSection {
    
    section_size: usize,
    num_imports: usize,
    imports: Vec<WasmImportHeader>
}

// Section describing imports
#[derive(Debug)]
pub struct WasmImportHeader {
    mod_name_length: usize,
    // of size mod_name_length
    import_module_name: Vec<u8>,
    import_field_len: usize,
    // of size emscripten_memcp_len
    import_field: Vec<u8>,
    import_kind: u8,
    import_signature_index: u8,
}

#[derive(Debug)]
pub struct WasmFunctionSection {
    
    section_size: usize,
    num_functions: usize,
    function_signature_indexes: Vec<u8>
}

#[derive(Debug)]
pub struct WasmTableSection {
    
    section_size: usize,
    num_tables: usize,
    tables: Vec<WasmTable>,
}
#[derive(Debug)]
pub struct WasmTable {
    funcref: u8,
    limits_flags: u8,
    limits_initial: usize,
    limits_max: usize,
}

#[derive(Debug)]
pub struct WasmMemorySection {
    section_size: usize,
    num_memories: usize,
    memories: Vec<WasmMemoryStruct>,
}

#[derive(Debug)]
pub struct WasmMemoryStruct {
    limits_flags: u8,
    limits_initial: usize,
    limits_max: usize,
}

#[derive(Debug)]
pub struct WasmGlobalSection {
    section_size: usize,
    num_globals: usize,
    globals: Vec<WasmGlobal>,
}
#[derive(Debug, Clone, Copy)]
pub struct WasmGlobal {
    wasm_type: WasmTypeAnnotation,
    mutability: u8,
    data: WasmTypedData,
}

#[derive(Debug)]
pub struct WasmExportSection {
    section_size: usize,
    num_exports: usize,
    exports: Vec<WasmExportHeader>
}

// Section describing imports
#[derive(Debug)]
pub struct WasmExportHeader {
    // of size emscripten_memcp_len
    export_name_len: usize,
    export_name: Vec<u8>,
    export_kind: u8,
    export_signature_index: u8,
}

fn read_global<T: Read + Debug>(state: &mut WasmDeserializeState<T>) -> Result<WasmGlobal, Error> {
    let mut global: WasmGlobal = WasmGlobal {
        wasm_type: WasmTypeAnnotation { _type: 0 },
        mutability: 0,
        data: WasmTypedData::F32(0.0),
    };
    global.wasm_type = state.read_sized(WasmTypeAnnotation { _type: 0 })?;
    global.mutability = state.read_sized(0)?;

    // This section is currently incomplete. The proper way of reading a global involves evaluating the expression
    // given before the end block. Hopefully as this project progresses we have an easy way of evaluating constant (non-
    // state dependent) expressions easily

    // For now we are assuming it's just going to put a constant on the stack, and that value is going to be our
    // global value. Hopefully I can find examples where it doesn't do this to get this to work in the future.

    match global.wasm_type {
        // f64
        WasmTypeAnnotation { _type: 0x7c } => {
            assert_eq!(
                state.read_sized(0)?,
                0x44 as u8,
                "Global is not a i32 const value"
            );
            global.data = WasmTypedData::F64(state.read_sized(0.0)?);
        }
        // TODO:: Confirm this is f32
        WasmTypeAnnotation { _type: 0x7d } => {
            assert_eq!(
                state.read_sized(0)?,
                0x43 as u8,
                "Global is not a i32 const value"
            );
            global.data = WasmTypedData::F32(state.read_sized(0.0)?);
        }
        // i64
        WasmTypeAnnotation { _type: 0x7e } => {
            assert_eq!(
                state.read_sized(0)?,
                0x42 as u8,
                "Global is not a i32 const value"
            );
            global.data = WasmTypedData::I64(state.read_dynamic_int(0)? as i64);
        }
        // i32
        WasmTypeAnnotation { _type: 0x7f } => {
            assert_eq!(
                state.read_sized(0)?,
                0x41 as u8,
                "Global is not a i32 const value"
            );
            global.data = WasmTypedData::I32(state.read_dynamic_int(0)? as i32);
        }
        _ => panic!(
            "No suitable type to read for global. Global struct: {:?}. State: {:?}",
            global, state
        ),
    }

    let end = state.read_sized::<u8>(0)?;
    // let end = state.read_sized::<u8>(0)?;
    assert_eq!(end, 0x0b);

    Ok(global)
}

#[derive(Debug)]
pub struct WasmFile {
    wasm_header: WasmHeader,
    type_section: WasmTypeSection,
    import_section_header: WasmImportSection,
    function_section: WasmFunctionSection,
    table_section: WasmTableSection,
    memory_section: WasmMemorySection,
    global_section: WasmGlobalSection,
    export_section: WasmExportSection,
}

#[derive(Debug)]
struct WasmDeserializeState<T: Read + Debug> {
    buffer: T,
}

impl<T: Read + Debug> WasmDeserializeState<T> {
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
        }
        return Ok(out);
    }

    // Read an int of dynamic size. See arcane_knowledge.md/#unsized_values for more information
    fn read_dynamic_int(&mut self, default: usize) -> Result<usize, Error> {
        let mut out: usize = default.clone();
        let mut buffer: [u8; 1] = [0];

        self.buffer.read_exact(&mut buffer)?;
        out += (buffer[0] & 0x7F) as usize;
        let mut bytes = 1;
        while buffer[0] & 0x80 != 0 && bytes < 8 {
            self.buffer.read_exact(&mut buffer)?;
            out += ((buffer[0] & 0x7F) as usize) << (7 * bytes);
            bytes += 1;
        }

        return Ok(out);
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
    fn read_type_section(&mut self) -> Result<WasmTypeSection, Error> {
    
        // A section that describes the type signature of functions
        let mut type_section: WasmTypeSection = WasmTypeSection {
            section_size: self.read_dynamic_int(0)?,
            num_types: self.read_dynamic_int(0)?,
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
    
            sig.num_params = self.read_dynamic_int(0)?;
            sig.params = self.read_vector(WasmTypeAnnotation { _type: 0 }, sig.num_params)?;
    
            sig.num_results = self.read_dynamic_int(0)?;
            sig.results = self.read_vector(WasmTypeAnnotation { _type: 0 }, sig.num_results)?;
            type_section.function_signatures.push(sig)
        }
        Ok(type_section)
    }
    fn read_import_section(&mut self) -> Result<WasmImportSection, Error> {
        // A section containing a description of things imported from other sources.
        // Each import header has a name and a signature index
        let mut import_section_header: WasmImportSection = WasmImportSection {
            section_size: self.read_dynamic_int(0)?,
            num_imports: self.read_dynamic_int(0)?,
            imports: Vec::new()
        };

        for _ in 0..import_section_header.num_imports {
            let mut import: WasmImportHeader = WasmImportHeader {
                mod_name_length: 0,
                import_module_name: Vec::new(),
                import_field_len: 0,
                import_field: Vec::new(),
                import_kind: 0,
                import_signature_index: 0,
            };
            import.mod_name_length = self.read_dynamic_int(0)?;
            import.import_module_name = self.read_vector(0, import.mod_name_length)?;
            import.import_field_len = self.read_dynamic_int(0)?;
            import.import_field = self.read_vector(0, import.import_field_len)?;
            import.import_kind = self.read_sized(0)?;
            import.import_signature_index = self.read_sized(0)?;
            import_section_header.imports.push(import);
        }
        Ok(import_section_header)
    }

    fn read_function_section(&mut self) -> Result<WasmFunctionSection, Error> { 
        let mut function_section: WasmFunctionSection = WasmFunctionSection {
            section_size: self.read_dynamic_int(0)?,
            num_functions: self.read_dynamic_int(0)?,
            function_signature_indexes: Vec::new()
        };
        for _ in 0..function_section.num_functions {
            function_section.function_signature_indexes.push(self.read_sized(0)?);
        }
        Ok(function_section)
    }

    fn read_table_section(&mut self) -> Result<WasmTableSection, Error> { 
        
        let mut table_section: WasmTableSection = WasmTableSection {
            section_size: self.read_dynamic_int(0)?,
            num_tables: self.read_dynamic_int(0)?,
            tables: Vec::new()
        };

        for _ in 0..table_section.num_tables {
            let mut table: WasmTable = WasmTable {
                funcref: 0,
                limits_flags: 0,
                limits_initial: 0,
                limits_max: 0,
            };
            table.funcref = self.read_sized::<u8>(0)?;
            table.limits_flags = self.read_sized::<u8>(0)?;
            table.limits_initial = self.read_dynamic_int(0)?;
            table.limits_max = self.read_dynamic_int(0)?;
            table_section.tables.push(table);
        }

        Ok(table_section)
    }
    
    fn read_memory_section(&mut self) -> Result<WasmMemorySection, Error> { 
        let mut memory_section: WasmMemorySection = WasmMemorySection {
            section_size: self.read_dynamic_int(0)?,
            num_memories: self.read_dynamic_int(0)?,
            memories: Vec::new()
        };
    
        for _ in 0..memory_section.num_memories {
            let mut memory: WasmMemoryStruct = WasmMemoryStruct {
                limits_flags: 0,
                limits_initial: 0,
                limits_max: 0,
            };
            memory.limits_flags = self.read_sized::<u8>(0)?;
            memory.limits_initial = self.read_dynamic_int(0)?;
            memory.limits_max = self.read_dynamic_int(0)?;
            memory_section.memories.push(memory);
        }
        Ok(memory_section)
    }
    
    fn read_global_section(&mut self) -> Result<WasmGlobalSection, Error> {     
        let mut global_section = WasmGlobalSection {
            section_size: self.read_dynamic_int(0)?,
            num_globals: self.read_dynamic_int(0)?,
            globals: Vec::new()
        };

        for _ in 0..global_section.num_globals {
            let global = read_global(self)?;
            println!("{:?}", global);
            global_section.globals.push(global);
        }
        Ok(global_section)
    }
    fn read_export_section(&mut self) -> Result<WasmExportSection, Error> { 
        
        let mut export_section: WasmExportSection = WasmExportSection {
            section_size: self.read_dynamic_int(0)?,
            num_exports: self.read_dynamic_int(0)?,
            exports: Vec::new()
        };

        for _ in 0..export_section.num_exports {
            let mut export: WasmExportHeader = WasmExportHeader {
                export_name_len: 0,
                export_name: Vec::new(),
                export_kind: 0,
                export_signature_index: 0,
            };
            export.export_name_len = self.read_dynamic_int(0)?;
            export.export_name = self.read_vector(0, export.export_name_len)?;
            export.export_kind = self.read_sized(0)?;
            export.export_signature_index = self.read_sized(0)?;
            export_section.exports.push(export);
        }
        Ok(export_section)
    }
    
}
// Reads a WASM file to a WasmFile struct.
pub fn wasm_deserialize(buffer: impl Read + Debug) -> Result<WasmFile, Error> {
    let mut state = WasmDeserializeState { buffer };
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

    while let Ok(section_type) = state.read_sized::<u8>(0) {
        println!("{}", section_type);
        match section_type {
            0x01 => type_section = state.read_type_section()?,
            0x02 => import_section_header = state.read_import_section()?,
            0x03 => function_section = state.read_function_section()?,
            0x04 => table_section = state.read_table_section()?,
            0x05 => memory_section = state.read_memory_section()?,
            0x06 => global_section = state.read_global_section()?,
            0x07 => export_section = state.read_export_section()?,
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
    });
}
