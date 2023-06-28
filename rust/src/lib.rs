use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use adapter::TelemetryId;
use anyhow::{anyhow, Result};
use log::error;
use modsurfer_demangle::demangle_function_name;
use std::collections::HashMap;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use version::{WASM_INSTR_VERSION_MAJOR, WASM_INSTR_VERSION_MINOR};
use wasmtime::{Caller, FuncType, Linker, Val, ValType};

pub mod adapter;
mod version;

#[derive(Clone)]
pub struct InstrumentationContext {
    id: usize,
    events_tx: Sender<Event>,
    stack: Vec<FunctionCall>,
}

impl InstrumentationContext {
    pub fn new(
        id: usize,
    ) -> (
        Arc<Mutex<InstrumentationContext>>,
        Sender<Event>,
        Receiver<Event>,
    ) {
        // TODO: decide how big the buffer for this channel should be
        // this channel will block the module if it fills
        let (events_tx, events_rx) = channel(128);
        (
            Arc::new(Mutex::new(InstrumentationContext {
                id,
                events_tx: events_tx.clone(),
                stack: Vec::new(),
            })),
            events_tx,
            events_rx,
        )
    }

    pub fn enter(&mut self, func_index: u32, func_name: Option<&str>) -> Result<()> {
        let mut fc = FunctionCall {
            index: func_index,
            start: SystemTime::now(),
            end: SystemTime::now(),
            within: Vec::new(),
            raw_name: None,
            name: None,
        };
        if let Some(name) = func_name {
            fc.name = Some(demangle_function_name(String::from(name)));
            fc.raw_name = Some(String::from(name));
        }
        self.stack.push(fc);
        Ok(())
    }

    pub fn exit(&mut self, func_index: u32) -> Result<()> {
        if let Some(mut func) = self.stack.pop() {
            if func.index != func_index {
                return Err(anyhow!("missed a function exit"));
            }
            func.end = SystemTime::now();

            if let Some(mut f) = self.stack.pop() {
                f.within.push(Event::Func(self.id, func.clone()));
                self.stack.push(f);
            }

            // only push the end of the final call onto the channel
            // this will contain all the other calls within it
            if self.stack.is_empty() {
                if let Err(e) = self.events_tx.try_send(Event::Func(self.id, func)) {
                    error!("error recording function exit: {}", e);
                };
            }

            return Ok(());
        }
        Err(anyhow!("empty stack in exit"))
    }

    pub fn allocate(&mut self, amount: u32) -> Result<()> {
        let ev = Event::Alloc(
            self.id,
            Allocation {
                ts: SystemTime::now(),
                amount,
            },
        );

        if let Some(mut f) = self.stack.pop() {
            f.within.push(ev.clone());
            self.stack.push(f);
        }

        if let Err(e) = self.events_tx.try_send(ev) {
            error!("error recording memory allocation: {}", e);
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Func(usize, FunctionCall),
    Alloc(usize, Allocation),
    Metadata(usize, Metadata),
    Shutdown(usize),
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub key: String,
    pub value: TelemetryId,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: Option<String>,
    pub raw_name: Option<String>,
    pub index: u32,
    pub start: SystemTime,
    pub end: SystemTime,
    pub within: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Allocation {
    pub ts: SystemTime,
    pub amount: u32,
}

pub(crate) fn instrument_enter(
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
    function_names: &FunctionNames,
) -> anyhow::Result<()> {
    let func_id = input[0].unwrap_i32() as u32;
    let printname = function_names.get(&func_id);
    if let Ok(mut cont) = ctx.lock() {
        cont.enter(func_id, printname.map(|x| x.as_str()))?;
    }
    Ok(())
}

pub(crate) fn instrument_exit(
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let func_id = input[0].unwrap_i32() as u32;
    if let Ok(mut cont) = ctx.lock() {
        cont.exit(func_id)?;
    }
    Ok(())
}

pub(crate) fn instrument_memory_grow(
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let amount_in_pages = input[0].unwrap_i32(); // The number of pages requested by `memory.grow` instruction
    if let Ok(mut cont) = ctx.lock() {
        cont.allocate(amount_in_pages as u32)?;
    }
    Ok(())
}

const MODULE_NAME: &str = "dylibso_observe";

type EventChannel = (Sender<Event>, Receiver<Event>);
type FunctionNames = HashMap<u32, String>;

// Static info from instrumentation
struct WasmInstrInfo {
    function_names: FunctionNames,
    maj_version: Option<u32>,
    min_version: Option<u32>,
}

impl WasmInstrInfo {
    fn new(data: &[u8]) -> Result<Self> {
        let mut function_names = FunctionNames::new();
        let mut maj_index: Option<u32> = None;
        let mut min_index: Option<u32> = None;
        let mut globals = HashMap::<u32, u32>::new();
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
        });
    }
}

/// Link observability import functions required by instrumented wasm code
pub fn add_to_linker<T: 'static>(
    id: usize,
    linker: &mut Linker<T>,
    data: &[u8],
) -> Result<EventChannel> {
    let (ctx, events_tx, events_rx) = InstrumentationContext::new(id);

    // load the static wasm-instr info
    let wasm_instr_info = WasmInstrInfo::new(data)?;

    // For now tolerate version info not being present
    if let WasmInstrInfo {maj_version: Some(maj_num), min_version: Some(min_num), ..} = &wasm_instr_info {
        if maj_num != WASM_INSTR_VERSION_MAJOR {
            anyhow::bail!("wasm wasm-instr major version {maj_num} is not equal to {WASM_INSTR_VERSION_MAJOR}!")
        }
        if maj_num < WASM_INSTR_VERSION_MINOR {
            anyhow::bail!(
                "wasm wasm-instr minor version {min_num} is less than {WASM_INSTR_VERSION_MINOR}!"
            )
        }
    }

    let t = FuncType::new([ValType::I32], []);

    let enter_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "instrument_enter",
        t.clone(),
        move |_caller: Caller<T>, params, results| {
            instrument_enter(
                params,
                results,
                enter_ctx.clone(),
                &wasm_instr_info.function_names,
            )
        },
    )?;

    let exit_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "instrument_exit",
        t.clone(),
        move |_caller, params, results| instrument_exit(params, results, exit_ctx.clone()),
    )?;

    linker.func_new(
        MODULE_NAME,
        "instrument_memory_grow",
        t,
        move |_caller, params, results| instrument_memory_grow(params, results, ctx.clone()),
    )?;

    // if the wasm was automatically instrumented using Dylibso's compiler, there will be some
    // metadata added to enforce compatibility with the SDK. This metadata is stored as a module
    // global export, which by default can cause wasmtime to return an error during instantiation.
    linker.allow_unknown_exports(true);
    Ok((events_tx, events_rx))
}
