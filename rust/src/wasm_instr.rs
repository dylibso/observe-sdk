use std::collections::HashMap;

use anyhow::{bail, Result};

// these control the versions of instrumented wasm supported
// WASM_INSTR_VERSION_MAJOR must match the instrumented wasm
// wasmInstrVersionMinor must be <= the value in instrumented wasm
pub const WASM_INSTR_VERSION_MAJOR: u32 = 0;
pub const WASM_INSTR_VERSION_MINOR: u32 = 4; // TODO: bump this to match compiler when ready

// Static info from instrumentation
pub struct WasmInstrInfo {
    pub function_names: HashMap<u32, String>,
}

impl WasmInstrInfo {
    pub fn new(data: &[u8]) -> Result<Self> {
        let mut function_names = HashMap::new();
        let mut warn_on_dylibso_observe = true;
        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(data) {
            match payload? {
                wasmparser::Payload::CustomSection(custom) => {
                    if custom.name() == "name" {
                        let name_reader =
                            wasmparser::NameSectionReader::new(custom.data(), custom.data_offset());
                        for x in name_reader.into_iter() {
                            if let wasmparser::Name::Function(f) = x? {
                                for k in f.into_iter() {
                                    let k = k?;
                                    function_names.insert(k.index, k.name.to_string());
                                }
                            }
                        }
                        continue;
                    }
                }
                wasmparser::Payload::ImportSection(importsec) => {
                    for import in importsec.into_iter() {
                        let import = import?;
                        if import.module == "dylibso_observe" {
                            if warn_on_dylibso_observe {
                                warn_on_dylibso_observe = false;
                                log::warn!(
                                "Module uses deprecated namespace \"dylibso_observe\"!
Please rebuild your module using the updated Observe API or reinstrument with the new version of wasm-instr."
                                );
                            }
                            for fname in ["span_enter", "span_tags", "metric", "log", "span_exit"] {
                                if import.name == fname {
                                    bail!(
                                        "Module uses old version of Observe API!
Please rebuild your module using the updated Observe API."
                                    );
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }

        return Ok(Self { function_names });
    }
}
