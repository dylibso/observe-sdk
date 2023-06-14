use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use anyhow::{anyhow, Result};
use log::error;
use modsurfer_demangle::demangle_function_name;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use wasmtime::{Caller, FrameInfo, FuncType, Linker, Val, ValType, WasmBacktrace, AsContextMut};

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

    pub fn enter(&mut self, fi: &FrameInfo) -> Result<()> {
        let mut fc = FunctionCall {
            index: fi.func_index(),
            start: SystemTime::now(),
            end: SystemTime::now(),
            within: Vec::new(),
            raw_name: None,
            name: None,
        };
        if let Some(name) = fi.func_name() {
            fc.name = Some(demangle_function_name(name.to_string()));
            fc.raw_name = Some(name.to_string());
        };
        self.stack.push(fc);
        Ok(())
    }

    pub fn exit(&mut self, fi: &FrameInfo) -> Result<()> {
        if let Some(mut func) = self.stack.pop() {
            if func.index != fi.func_index() {
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

    pub fn allocate(&mut self, _fi: &FrameInfo, amount: u32) -> Result<()> {
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

    pub fn write_statsd(&mut self, stat: &[u8]) -> Result<()> {
        //let stat = from_utf8(stat)?.to_string();
        let ev = Event::Stats(stat.to_owned());

        if let Err(e) = self.events_tx.try_send(ev) {
            error!("error recording stat: {}", e);
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Func(usize, FunctionCall),
    Alloc(usize, Allocation),
    Stats(Vec<u8>),
    Metadata(usize, Metadata),
    Shutdown(usize),
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub key: String,
    pub value: u64,
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
) -> anyhow::Result<()> {
    let bt = WasmBacktrace::capture(caller);

    if let Some(frame) = bt.frames().first() {
        if let Ok(mut cont) = ctx.lock() {
            cont.enter(frame)?;
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
    let bt = WasmBacktrace::capture(caller);

    if let Some(frame) = bt.frames().first() {
        if let Ok(mut cont) = ctx.lock() {
            cont.exit(frame)?;
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
    let bt = WasmBacktrace::capture(caller);

    // TODO: this should scream and die loudly if it can't actually get ahold of the frame
    if let Some(frame) = bt.frames().last() {
        if let Ok(mut cont) = ctx.lock() {
            cont.allocate(frame, amount_in_pages as u32)?;
        }
    }
    Ok(())
}

pub(crate) fn write_statsd<T>(
    caller: &mut Caller<T>,
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let offset = input.get(0).unwrap().i32().unwrap();
    let len = input.get(1).unwrap().i32().unwrap();
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mut buffer = vec![0u8; len as usize];

    memory.read(caller, offset as usize, &mut buffer)?;

    if let Ok(mut cont) = ctx.lock() {
        cont.write_statsd(&buffer)?;
    }
    Ok(())
}

const MODULE_NAME: &str = "dylibso_observe";

type EventChannel = (Sender<Event>, Receiver<Event>);

/// Link observability import functions required by instrumented wasm code
pub fn add_to_linker<T: 'static>(id: usize, linker: &mut Linker<T>) -> Result<EventChannel> {
    let (ctx, events_tx, events_rx) = InstrumentationContext::new(id);

    let t = FuncType::new([], []);

    let enter_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "instrument_enter",
        t.clone(),
        move |caller, params, results| instrument_enter(caller, params, results, enter_ctx.clone()),
    )?;

    let exit_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "instrument_exit",
        t,
        move |caller, params, results| instrument_exit(caller, params, results, exit_ctx.clone()),
    )?;

    let grow_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "instrument_memory_grow",
        FuncType::new([ValType::I32], []),
        move |caller, params, results| instrument_memory_grow(caller, params, results, grow_ctx.clone()),
    )?;

    let write_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "write_statsd",
        FuncType::new([ValType::I32, ValType::I32], []),
        move |mut caller, params, results| write_statsd(&mut caller, params, results, write_ctx.clone()),
    )?;

    Ok((events_tx, events_rx))
}
