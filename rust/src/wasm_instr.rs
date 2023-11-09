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
    pub maj_version: Option<u32>,
    pub min_version: Option<u32>,
    pub old_api: bool,
}

impl WasmInstrInfo {
    // uninstrumented modules are allowed as long as they don't have the old
    // dylibso_observe imports
    pub fn check_version(&self) -> Result<()> {
        if self.maj_version.is_some() || self.min_version.is_some() {
            let maj_num = self.maj_version.unwrap();
            let min_num = self.min_version.unwrap();

            if maj_num != WASM_INSTR_VERSION_MAJOR {
                bail!("Module wasm_instr_version_major {maj_num} is not equal to {WASM_INSTR_VERSION_MAJOR}!
Please reinstrument your module with compatible wasm-instr.")
            }

            if min_num < WASM_INSTR_VERSION_MINOR {
                bail!(
                    "Module wasm_instr_version_minor {min_num} is less than {WASM_INSTR_VERSION_MINOR}!
Please reinstrument your module with the new version of wasm_instr."
                );
            }
        }

        if self.old_api {
            bail!(
                "Module imports from removed api, dylibso_observe!
Please rebuild your module using the updated Observe API or reinstrument with the new version of wasm-instr."
            );
        }

        Ok(())
    }

    pub fn new(data: &[u8]) -> Result<Self> {
        let mut function_names = HashMap::new();
        let mut maj_index: Option<u32> = None;
        let mut min_index: Option<u32> = None;
        let mut globals = HashMap::<u32, u32>::new();
        let mut old_api = false;
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
                wasmparser::Payload::GlobalSection(globalsec) => {
                    for (i, aglob) in globalsec.into_iter().enumerate() {
                        let glob = aglob.unwrap();
                        if glob.ty.content_type != wasmparser::ValType::I32 {
                            continue;
                        }
                        let mut reader = glob.init_expr.get_binary_reader();
                        let opcode = match reader.read_u8() {
                            Ok(opcode) => opcode,
                            Err(_) => continue,
                        };
                        // i32.const
                        if opcode != 0x41 {
                            continue;
                        }
                        // due to binaryen limitations u32 version values are encoded as a signed LEB128
                        // integers so they must be casted back to unsigned
                        let iv = reader.read_var_i32().unwrap();
                        let uv = iv as u32;
                        globals.insert(i as u32, uv);
                    }
                }
                wasmparser::Payload::ExportSection(exportsec) => {
                    for export in exportsec.into_iter() {
                        let export = export?;
                        if export.kind != wasmparser::ExternalKind::Global {
                            continue;
                        }
                        match export.name {
                            "wasm_instr_version_major" => {
                                maj_index = Some(export.index);
                            }
                            "wasm_instr_version_minor" => {
                                min_index = Some(export.index);
                            }
                            _ => {}
                        }
                    }
                }
                wasmparser::Payload::ImportSection(importsec) => {
                    for import in importsec.into_iter() {
                        let import = import?;
                        if import.module == "dylibso_observe" {
                            old_api = true;
                            break;
                        }
                    }
                }
                _ => (),
            }
        }
        let maj_version = match maj_index {
            Some(maj_index) => Some(*globals.get(&maj_index).unwrap()),
            None => None,
        };
        let min_version = match min_index {
            Some(min_index) => Some(*globals.get(&min_index).unwrap()),
            None => None,
        };
        return Ok(Self {
            function_names,
            maj_version,
            min_version,
            old_api,
        });
    }
}
