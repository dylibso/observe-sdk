// use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use anyhow::{anyhow, Result};
use modsurfer_demangle::demangle_function_name;
use wasmtime::{Caller, FrameInfo, FuncType, Linker, Val, ValType, WasmBacktrace};
use std::collections::HashMap;

pub mod adapter;

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
        Receiver<Event>,
        Sender<Event>,
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
            events_rx,
            events_tx,
        )
    }

    pub async fn send_event(&self, ev: Event) -> Result<()> {
        self.events_tx.send(ev).await?;
        Ok(())
    }

    pub fn enter(&mut self, func_index : u32, func_name : &String) -> Result<()> {
        let mut fc = FunctionCall {
            index: func_index,
            start: SystemTime::now(),
            end: SystemTime::now(),
            within: Vec::new(),
            raw_name: None,
            name: None,
        };
        fc.name = Some(demangle_function_name(String::from(func_name)));
        fc.raw_name = Some(String::from(func_name));
        self.stack.push(fc);
        Ok(())
    }

    pub fn exit(&mut self, func_index : u32) -> Result<()> {
        if let Some(mut func) = self.stack.pop() {
            if func.index != func_index {
                return Err(anyhow!("we missed a function exit"));
            }
            func.end = SystemTime::now();

            if let Some(mut f) = self.stack.pop() {
                f.within.push(Event::Func(self.id, func.clone()));
                self.stack.push(f);
            }

            // only push the end of the final call onto the channel
            // this will contain all the other calls within it
            if self.stack.is_empty() {
                self.events_tx.try_send(Event::Func(self.id, func))?;
            }

            return Ok(());
        }
        Err(anyhow!("oh ohh the stack was empty"))
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

        // self.events_tx.blocking_send(ev)?;
        self.events_tx.try_send(ev)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Func(usize, FunctionCall),
    Alloc(usize, Allocation),
    Metadata(usize, String),
    Shutdown(usize),
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

pub(crate) fn instrument_enter<T>(
    caller: Caller<T>,
    _input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
    function_names: &FunctionNames
) -> anyhow::Result<()> {
    let func_id = _input[0].unwrap_i32() as u32;
    let func_name_offset = _input[1].unwrap_i32() as u32;
    let bt = WasmBacktrace::capture(&caller);

    if let Some(frame) = bt.frames().first() {
        if let Ok(mut cont) = ctx.lock() {
            /*let fid = frame.func_index();
            let mut jname = String::new();
            if let Some(name) = frame.func_name() {
                let mut oname = Some(demangle_function_name(name.to_string()));
                oname = Some(name.to_string());
                jname = oname.unwrap();
            }*/
            let mut printname = "placeholder".to_string();
            let temp = function_names.get(&func_id);
            if temp.is_some() {
                printname = temp.unwrap().to_string();
            }
            cont.enter(func_id, &printname)?;
        }
    }

    Ok(())
}

pub(crate) fn instrument_exit<T>(
    caller: Caller<T>,
    _input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let func_id = _input[0].unwrap_i32() as u32;
    let bt = WasmBacktrace::capture(&caller);

    if let Some(frame) = bt.frames().first() {
        if let Ok(mut cont) = ctx.lock() {
            cont.exit(func_id)?;
        }
    }

    Ok(())
}

pub(crate) fn instrument_memory_grow<T>(
    caller: Caller<T>,
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let amount_in_pages = input[0].unwrap_i32(); // The number of pages requested by `memory.grow` instruction
    let bt = WasmBacktrace::capture(&caller);

    // TODO: this should scream and die loudly if it can't actually get ahold of the frame
    if let Some(frame) = bt.frames().last() {
        if let Ok(mut cont) = ctx.lock() {
            cont.allocate(amount_in_pages as u32)?;
        }
    }
    Ok(())
}

const MODULE_NAME: &'static str = "dylibso_observe";
type EventChannels = (Receiver<Event>, Sender<Event>);
type FunctionNames = HashMap<u32, String>;

pub fn add_to_linker<T: 'static>(id: usize, linker: &mut Linker<T>, data : &Vec<u8>) -> Result<EventChannels> {
    let (ctx, events_rx, events_tx) = InstrumentationContext::new(id);

    // Build up a table of function id to name mapping
    //let mut function_names = Vec::new();
    let mut function_names = FunctionNames::new();
    let parser = wasmparser::Parser::new(0);
    for payload in parser.parse_all(&data) {
        let payload = payload?;
        match payload {
            wasmparser::Payload::CustomSection(custom) => {
                if custom.name() == "name" {
                    let name_reader =
                        wasmparser::NameSectionReader::new(custom.data(), custom.data_offset());
                    for x in name_reader.into_iter() {
                        let x = x?;
                        match x {
                            //wasmparser::Name::Module { name, name_range } => (),
                            wasmparser::Name::Function(f) => {
                                for k in f.into_iter() {
                                    let k = k?;
                                    function_names.insert(k.index, k.name.to_string());
                                }
                            },
                            _ => ()
                            //wasmparser::Name::Local(_) => (),
                            //wasmparser::Name::Label(_) => (),
                            //wasmparser::Name::Type(_) => (),
                            //wasmparser::Name::Table(_) => (),
                            //wasmparser::Name::Memory(_) => (),
                            //wasmparser::Name::Global(_) => (),
                            //wasmparser::Name::Element(_) => (),
                            //wasmparser::Name::Data(_) => (),
                            //wasmparser::Name::Unknown { ty, data, range } => (),
                        }
                    }
                    continue;
                }
            },
            _ => {

            }
        }
    }

    // TODO: figure out how to do with with less calls to `clone`

    // Enter
    let ctx1 = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "instrument_enter",
        FuncType::new([ValType::I32, ValType::I32], []),
        move |caller, params, results| instrument_enter(caller, params, results, ctx1.clone(), &function_names),
    )?;

    // Exit
    let ctx2 = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "instrument_exit",
        FuncType::new([ValType::I32], []),
        move |caller, params, results| instrument_exit(caller, params, results, ctx2.clone()),
    )?;

    // Trace
    let ctx3 = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "instrument_memory_grow",
        FuncType::new([ValType::I32], []),
        move |caller, params, results| {
            instrument_memory_grow(caller, params, results, ctx3.clone())
        },
    )?;

    Ok((events_rx, events_tx))
}