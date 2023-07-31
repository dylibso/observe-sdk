use anyhow::{anyhow, Context, Result};
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
    Metric, MetricFormat, Tags,
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
            // if func.index != func_index {
            //     return Err(anyhow!("missed a function exit"));
            // }
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

        Ok(())
    }

    fn metric(&mut self, format: MetricFormat, message: &[u8]) -> Result<()> {
        let message = from_utf8(message)?.to_string();

        let ev = Event::Metric(Metric {
            ts: SystemTime::now(),
            trace_id: None,
            format,
            message,
        });

        if let Some(mut f) = self.stack.pop() {
            f.within.push(ev.clone());
            self.stack.push(f);
        }

        Ok(())
    }

    fn span_tags(&mut self, tags: Vec<String>) -> Result<()> {
        let ev = Event::Tags(Tags {
            ts: SystemTime::now(),
            tags,
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
    let func_id = input
        .get(0)
        .context("Missing func_id arg")?
        .i32()
        .context("Could not convert func_id arg to i32")? as u32;
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
    let func_id = input
        .get(0)
        .context("Missing func_id arg")?
        .i32()
        .context("Could not convert func_id arg to i32")?;

    if let Ok(mut cont) = ctx.lock() {
        cont.exit(func_id as u32)?;
    }
    Ok(())
}

pub(crate) fn instrument_memory_grow(
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let amount_in_pages = input
        .get(0)
        .context("Missing amount_in_pages arg")?
        .i32()
        .context("Could not convert amount_in_pages arg to i32")?;

    if let Ok(mut cont) = ctx.lock() {
        cont.allocate(amount_in_pages as u32)?;
    }
    Ok(())
}

pub(crate) fn metric<T>(
    caller: &mut Caller<T>,
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let format = input
        .get(0)
        .context("Missing format arg")?
        .i32()
        .context("Could not convert format arg to i32")?;

    let format = match format {
        1 => MetricFormat::Statsd,
        _ => anyhow::bail!("Illegal metric format value"),
    };

    let ptr = input
        .get(1)
        .context("Missing ptr arg")?
        .i32()
        .context("Could not convert ptr arg to i32")?;

    let len = input
        .get(2)
        .context("Missing len arg")?
        .i32()
        .context("Could not convert len arg to i32")?;

    let memory = caller
        .get_export("memory")
        .context("Could not get memory from caller")?
        .into_memory()
        .context("Could not convert to into memory")?;

    let mut buffer = vec![0u8; len as usize];
    memory.read(caller, ptr as usize, &mut buffer)?;

    if let Ok(mut cont) = ctx.lock() {
        cont.metric(format, &buffer)?;
    }

    Ok(())
}

pub(crate) fn span_tags<T>(
    caller: &mut Caller<T>,
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let ptr = input
        .get(0)
        .context("Missing ptr arg")?
        .i32()
        .context("Could not convert ptr arg to i32")?;

    let len = input
        .get(1)
        .context("Missing len arg")?
        .i32()
        .context("Could not convert len arg to i32")?;

    let memory = caller
        .get_export("memory")
        .context("Could not get memory from caller")?
        .into_memory()
        .context("Could not convert to into memory")?;

    let mut buffer = vec![0u8; len as usize];
    memory.read(caller, ptr as usize, &mut buffer)?;

    let tags: Vec<String> = from_utf8(&buffer)?
        .split(|c| c == ',')
        .map(|e| e.to_string())
        .collect();

    if let Ok(mut cont) = ctx.lock() {
        cont.span_tags(tags)?;
    }

    Ok(())
}

pub(crate) fn log_write<T>(
    caller: &mut Caller<T>,
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let level = input
        .get(0)
        .context("Missing level arg")?
        .i32()
        .context("Could not convert ptr arg to i32")?;

    let ptr = input
        .get(1)
        .context("Missing ptr arg")?
        .i32()
        .context("Could not convert ptr arg to i32")?;

    let len = input
        .get(2)
        .context("Missing len arg")?
        .i32()
        .context("Could not convert len arg to i32")?;

    let memory = caller
        .get_export("memory")
        .context("Could not get memory from caller")?
        .into_memory()
        .context("Could not convert to into memory")?;

    let mut buffer = vec![0u8; len as usize];
    memory.read(caller, ptr as usize, &mut buffer)?;

    if let Ok(mut cont) = ctx.lock() {
        cont.log_write(level as u8, &buffer)?;
    }

    Ok(())
}

pub(crate) fn span_enter<T>(
    caller: &mut Caller<T>,
    input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    let ptr = input
        .get(0)
        .context("Missing ptr arg")?
        .i32()
        .context("Could not convert ptr arg to i32")?;

    let len = input
        .get(1)
        .context("Missing len arg")?
        .i32()
        .context("Could not convert len arg to i32")?;

    let memory = caller
        .get_export("memory")
        .context("Could not get memory from caller")?
        .into_memory()
        .context("Could not convert to into memory")?;

    let mut buffer = vec![0u8; len as usize];
    memory.read(caller, ptr as usize, &mut buffer)?;
    let name = from_utf8(&buffer)?;

    if let Ok(mut cont) = ctx.lock() {
        cont.enter(0u32, Some(name))?;
    }

    Ok(())
}

pub(crate) fn span_exit<T>(
    _caller: &mut Caller<T>,
    _input: &[Val],
    _output: &mut [Val],
    ctx: Arc<Mutex<InstrumentationContext>>,
) -> Result<()> {
    if let Ok(mut cont) = ctx.lock() {
        cont.exit(0u32)?;
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

    let span_enter_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "span_enter",
        t.clone(),
        move |mut caller, params, results| {
            span_enter(&mut caller, params, results, span_enter_ctx.clone())
        },
    )?;

    let span_tags_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "span_tags",
        t.clone(),
        move |mut caller, params, results| {
            span_tags(&mut caller, params, results, span_tags_ctx.clone())
        },
    )?;

    let t = FuncType::new([ValType::I32, ValType::I32, ValType::I32], []);

    let metric_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "metric",
        t.clone(),
        move |mut caller, params, results| metric(&mut caller, params, results, metric_ctx.clone()),
    )?;

    let log_ctx = ctx.clone();
    linker.func_new(MODULE_NAME, "log", t, move |mut caller, params, results| {
        log_write(&mut caller, params, results, log_ctx.clone())
    })?;

    let t = FuncType::new([], []);

    let span_exit_ctx = ctx.clone();
    linker.func_new(
        MODULE_NAME,
        "span_exit",
        t,
        move |mut caller, params, results| {
            span_exit(&mut caller, params, results, span_exit_ctx.clone())
        },
    )?;

    // if the wasm was automatically instrumented using Dylibso's compiler, there will be some
    // metadata added to enforce compatibility with the SDK. This metadata is stored as a module
    // global export, which by default can cause wasmtime to return an error during instantiation.
    linker.allow_unknown_exports(true);
    Ok((events_tx, events_rx))
}
