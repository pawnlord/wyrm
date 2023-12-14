use core::num;
use std::{
    default,
    fmt::Debug,
    fs::{self, File},
    io::{Error, Read},
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
    section_code: u8,
    section_size: usize,
    num_types: usize,
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
    section_code: u8,
    section_size: usize,
    num_imports: usize,
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
    section_code: u8,
    section_size: usize,
    num_functions: usize,
}

#[derive(Debug)]
pub struct WasmTableSection {
    section_code: u8,
    section_size: usize,
    num_tables: usize,
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
    section_code: u8,
    section_size: usize,
    num_memories: usize,
}

#[derive(Debug)]
pub struct WasmMemoryStruct {
    limits_flags: u8,
    limits_initial: usize,
    limits_max: usize,
}

#[derive(Debug)]
pub struct WasmGlobalSection {
    section_code: u8,
    section_size: usize,
    num_globals: usize,
}
#[derive(Debug, Clone, Copy)]
pub struct WasmGlobal {
    wasm_type: WasmTypeAnnotation,
    mutability: u8,
    data: WasmTypedData,
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
    function_signatures: Vec<WasmFunctionType>,
    import_section_header: WasmImportSection,
    import_headers: Vec<WasmImportHeader>,
    function_section: WasmFunctionSection,
    function_signature_indexes: Vec<u8>,
    table_section: WasmTableSection,
    tables: Vec<WasmTable>,
    memory_section: WasmMemorySection,
    memories: Vec<WasmMemoryStruct>,
    global_section: WasmGlobalSection,
    globals: Vec<WasmGlobal>,
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

    // A section that describes the type signature of functions
    let type_section: WasmTypeSection = WasmTypeSection {
        section_code: state.read_sized::<u8>(0)?,
        section_size: state.read_dynamic_int(0)?,
        num_types: state.read_dynamic_int(0)?,
    };

    let mut function_signatures: Vec<WasmFunctionType> = Vec::new();
    for _ in 0..type_section.num_types {
        let mut sig: WasmFunctionType = WasmFunctionType {
            func: 0,
            num_params: 0,
            params: Vec::new(),
            num_results: 0,
            results: Vec::new(),
        };
        sig.func = state.read_sized::<u8>(0)?;
        assert_eq!(sig.func, 0x60, "Function format was incorrect");

        sig.num_params = state.read_dynamic_int(0)?;
        sig.params = state.read_vector(WasmTypeAnnotation { _type: 0 }, sig.num_params)?;

        sig.num_results = state.read_dynamic_int(0)?;
        sig.results = state.read_vector(WasmTypeAnnotation { _type: 0 }, sig.num_results)?;
        function_signatures.push(sig)
    }

    // A section containing a description of things imported from other sources.
    // Each import header has a name and a signature index
    let import_section_header: WasmImportSection = WasmImportSection {
        section_code: state.read_sized::<u8>(0)?,
        section_size: state.read_dynamic_int(0)?,
        num_imports: state.read_dynamic_int(0)?,
    };

    let mut import_headers: Vec<WasmImportHeader> = Vec::new();
    for _ in 0..import_section_header.num_imports {
        let mut import: WasmImportHeader = WasmImportHeader {
            mod_name_length: 0,
            import_module_name: Vec::new(),
            import_field_len: 0,
            import_field: Vec::new(),
            import_kind: 0,
            import_signature_index: 0,
        };
        import.mod_name_length = state.read_dynamic_int(0)?;
        import.import_module_name = state.read_vector(0, import.mod_name_length)?;
        import.import_field_len = state.read_dynamic_int(0)?;
        import.import_field = state.read_vector(0, import.import_field_len)?;
        import.import_kind = state.read_sized(0)?;
        import.import_signature_index = state.read_sized(0)?;
        import_headers.push(import);
    }

    let function_section: WasmFunctionSection = WasmFunctionSection {
        section_code: state.read_sized::<u8>(0)?,
        section_size: state.read_dynamic_int(0)?,
        num_functions: state.read_dynamic_int(0)?,
    };
    let mut function_signature_indexes: Vec<u8> = Vec::new();
    for _ in 0..function_section.num_functions {
        function_signature_indexes.push(state.read_sized(0)?);
    }

    let table_section: WasmTableSection = WasmTableSection {
        section_code: state.read_sized::<u8>(0)?,
        section_size: state.read_dynamic_int(0)?,
        num_tables: state.read_dynamic_int(0)?,
    };

    let mut tables: Vec<WasmTable> = Vec::new();
    for _ in 0..table_section.num_tables {
        let mut table: WasmTable = WasmTable {
            funcref: 0,
            limits_flags: 0,
            limits_initial: 0,
            limits_max: 0,
        };
        table.funcref = state.read_sized::<u8>(0)?;
        table.limits_flags = state.read_sized::<u8>(0)?;
        table.limits_initial = state.read_dynamic_int(0)?;
        table.limits_max = state.read_dynamic_int(0)?;
        tables.push(table);
    }

    let memory_section: WasmMemorySection = WasmMemorySection {
        section_code: state.read_sized::<u8>(0)?,
        section_size: state.read_dynamic_int(0)?,
        num_memories: state.read_dynamic_int(0)?,
    };

    let mut memories: Vec<WasmMemoryStruct> = Vec::new();
    for _ in 0..memory_section.num_memories {
        let mut memory: WasmMemoryStruct = WasmMemoryStruct {
            limits_flags: 0,
            limits_initial: 0,
            limits_max: 0,
        };
        memory.limits_flags = state.read_sized::<u8>(0)?;
        memory.limits_initial = state.read_dynamic_int(0)?;
        memory.limits_max = state.read_dynamic_int(0)?;
        memories.push(memory);
    }

    let global_section = WasmGlobalSection {
        section_code: state.read_sized::<u8>(0)?,
        section_size: state.read_dynamic_int(0)?,
        num_globals: state.read_dynamic_int(0)?,
    };
    let mut globals: Vec<WasmGlobal> = Vec::new();
    for _ in 0..global_section.num_globals {
        let global = read_global(&mut state)?;
        println!("{:?}", global);
        globals.push(global);
    }

    return Ok(WasmFile {
        wasm_header,
        type_section,
        function_signatures,
        import_section_header,
        import_headers,
        function_section,
        function_signature_indexes,
        table_section,
        tables,
        memory_section,
        memories,
        global_section,
        globals,
    });
}
