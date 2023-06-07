// use std::sync::mpsc::{channel, Receiver, Sender};
use log::error;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use anyhow::{anyhow, Result};
use modsurfer_demangle::demangle_function_name;
use wasmtime::{Caller, FrameInfo, FuncType, Linker, Val, ValType, WasmBacktrace};

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

    pub fn send_event(&self, ev: Event) -> Result<()> {
        if let Err(e) = self.events_tx.try_send(ev.clone()) {
            match ev {
                Event::Func(id, _) => error!("error recording function {}: {}", id, e),
                Event::Alloc(_, _) => error!("error recording memory allocation: {}", e),
                Event::Metadata(_, _) => error!("error recording metadata event: {}", e),
                Event::Shutdown(_) => error!("error recording shutdown event: {}", e),
            }
        }
        Ok(())
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
            fc.name = Some(name.to_string());
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
                self.send_event(Event::Func(self.id, func))?;
            }

            return Ok(());
        }
        Err(anyhow!("empty stack"))
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

        self.send_event(ev)?;
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

const MODULE_NAME: &str = "dylibso_observe";

type EventChannels = (Receiver<Event>, Sender<Event>);

/// Link observability import functions required by instrumented wasm code
pub fn add_to_linker<T: 'static>(id: usize, linker: &mut Linker<T>) -> Result<EventChannels> {
    let (ctx, events_rx, events_tx) = InstrumentationContext::new(id);

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

    linker.func_new(
        MODULE_NAME,
        "instrument_memory_grow",
        FuncType::new([ValType::I32], []),
        move |caller, params, results| instrument_memory_grow(caller, params, results, ctx.clone()),
    )?;

    Ok((events_rx, events_tx))
}
