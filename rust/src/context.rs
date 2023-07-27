use anyhow::{anyhow, Result};
use log::{error, warn};
use modsurfer_demangle::demangle_function_name;
use std::{
    collections::HashMap,
    str::from_utf8,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use wasmtime::{Caller, FuncType, Linker, Val, ValType};

use crate::{
    collector::CollectorHandle, wasm_instr::WasmInstrInfo, Allocation, Event, FunctionCall, Log,
    Statsd,
};

/// The InstrumentationContext holds the implementations
/// of the Observe Wasm host functions. As these are triggered,
/// this module captures the function calls and collects them in
/// Events which get sent to the background Collector task.
///
///
/// ┌────────────────┐         ┌──────────────┐
/// │  InstrContext  │         │  Collector   │
/// │                │         │              │
/// │                │         │              │
/// ├────────────────┤ <Event> ├──────────────┤
/// │  collector_tx  ├────────►│ collector_rx │
/// └────────────────┘         └──────────────┘
#[derive(Clone)]
pub struct InstrumentationContext {
    collector: CollectorHandle,
    stack: Vec<FunctionCall>,
}

impl InstrumentationContext {
    fn new() -> (
        Arc<Mutex<InstrumentationContext>>,
        CollectorHandle,
        Receiver<Event>,
    ) {
        // TODO: decide how big the buffer for this channel should be
        // this channel will block the module if it fills
        let (events_tx, events_rx) = channel(128);
        (
            Arc::new(Mutex::new(InstrumentationContext {
                collector: events_tx.clone(),
                stack: Vec::new(),
            })),
            events_tx,
            events_rx,
        )
    }

    fn enter(&mut self, func_index: u32, func_name: Option<&str>) -> Result<()> {
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

    fn exit(&mut self, func_index: u32) -> Result<()> {
        if let Some(mut func) = self.stack.pop() {
            if func.index != func_index {
                return Err(anyhow!("missed a function exit"));
            }
            func.end = SystemTime::now();

            if let Some(mut f) = self.stack.pop() {
                f.within.push(Event::Func(func.clone()));
                self.stack.push(f);
            }

            // only push the end of the final call onto the channel
            // this will contain all the other calls within it
            if self.stack.is_empty() {
                if let Err(e) = self.collector.try_send(Event::Func(func)) {
                    error!("error recording function exit: {}", e);
                };
            }

            return Ok(());
        }
        Err(anyhow!("empty stack in exit"))
    }

    fn allocate(&mut self, amount: u32) -> Result<()> {
        let ev = Event::Alloc(Allocation {
            ts: SystemTime::now(),
            amount,
        });

        if let Some(mut f) = self.stack.pop() {
            f.within.push(ev.clone());
            self.stack.push(f);
        }

        if let Err(e) = self.collector.try_send(ev) {
            error!("error recording memory allocation: {}", e);
        }
        Ok(())
    }

    fn statsd(&mut self, message: &[u8]) -> Result<()> {
        let message = from_utf8(message)?.to_string();

        let ev = Event::Statsd(Statsd {
            ts: SystemTime::now(),
            trace_id: None,
            message,
        });

        if let Some(mut f) = self.stack.pop() {
            f.within.push(ev.clone());
            self.stack.push(f);
        }

        Ok(())
    }

    fn log_write(&mut self, level: u8, message: &[u8]) -> Result<()> {
        let level = match level {
            1 => log::Level::Error,
            2 => log::Level::Warn,
            3 => log::Level::Info,
            4 => log::Level::Debug,
            _ => anyhow::bail!("Could not map log level to an appropriate level"),
        };

        let message = from_utf8(message)?.to_string();

        let ev = Event::Log(Log {
            ts: SystemTime::now(),
            message,
            level,
        });
        println!("Got a log");

        if let Some(mut f) = self.stack.pop() {
            f.within.push(ev.clone());
            self.stack.push(f);
        }

        Ok(())
    }
}

pub(crate) fn instrument_enter(
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
    function_names: &HashMap<u32, String>,
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

pub(crate) fn statsd<T>(
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
        cont.statsd(&buffer)?;
    }
    Ok(())
}

pub(crate) fn log_write<T>(
    caller: &mut Caller<T>,
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let level = input.get(0).unwrap().i32().unwrap();
    let offset = input.get(1).unwrap().i32().unwrap();
    let len = input.get(2).unwrap().i32().unwrap();
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mut buffer = vec![0u8; len as usize];

    memory.read(caller, offset as usize, &mut buffer)?;

    if let Ok(mut cont) = ctx.lock() {
        cont.log_write(level as u8, &buffer)?;
    }
    Ok(())
}

const MODULE_NAME: &str = "dylibso_observe";

type EventChannel = (Sender<Event>, Receiver<Event>);

/// Link observability import functions required by instrumented wasm code
pub fn add_to_linker<T: 'static>(linker: &mut Linker<T>, data: &[u8]) -> Result<EventChannel> {
    let (ctx, events_tx, events_rx) = InstrumentationContext::new();

    // load the static wasm-instr info
    let wasm_instr_info = WasmInstrInfo::new(data)?;

    // check that the version number is supported with this SDK
    // TODO decide what to do about this error?
    if let Err(e) = wasm_instr_info.check_version() {
        warn!("{}", e);
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

    let grow_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "instrument_memory_grow",
        t,
        move |_caller, params, results| instrument_memory_grow(params, results, grow_ctx.clone()),
    )?;

    let t = FuncType::new([ValType::I32, ValType::I32], []);

    let statsd_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "statsd",
        t,
        move |mut caller, params, results| statsd(&mut caller, params, results, statsd_ctx.clone()),
    )?;

    let t = FuncType::new([ValType::I32, ValType::I32, ValType::I32], []);

    let log_ctx = ctx.clone();
    linker.func_new(MODULE_NAME, "log", t, move |mut caller, params, results| {
        log_write(&mut caller, params, results, log_ctx.clone())
    })?;

    // if the wasm was automatically instrumented using Dylibso's compiler, there will be some
    // metadata added to enforce compatibility with the SDK. This metadata is stored as a module
    // global export, which by default can cause wasmtime to return an error during instantiation.
    linker.allow_unknown_exports(true);
    Ok((events_tx, events_rx))
}
