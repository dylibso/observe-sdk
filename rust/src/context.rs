use anyhow::{bail, Context, Result};
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
    adapter::Options, collector::CollectorHandle, wasm_instr::WasmInstrInfo, Allocation, Event,
    FunctionCall, Log, Metric, MetricFormat, Tags,
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
    options: Options,
}

impl InstrumentationContext {
    pub(crate) fn new(
        options: Options,
    ) -> (
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
                options,
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

    fn exit(&mut self, _func_index: u32) -> Result<()> {
        if let Some(mut func) = self.stack.pop() {
            // TODO this prevents us from using wasm-instr and manual instr
            // we should decide how to handle this and put this back in some way
            // if func_index != 0 && func.index != func_index {
            //     bail!("missed a function exit");
            // }
            func.end = SystemTime::now();

            // if the stack is empty, we are exiting the root function of the trace
            if self.stack.is_empty() {
                if let Err(e) = self.collector.try_send(Event::Func(func)) {
                    error!("error recording function exit: {}", e);
                };
            } else {
                // if the function duration is less than minimum duration, disregard
                let func_duration = func.end.duration_since(func.start)?.as_micros();
                let min_span_duration = self
                    .options
                    .span_filter
                    .min_duration_microseconds
                    .as_micros();
                if func_duration < min_span_duration {
                    // check for memory allocations and attribute them to the parent span
                    if let Some(mut f) = self.stack.pop() {
                        func.within.into_iter().for_each(|ev| match ev {
                            Event::Alloc(e) => {
                                f.within.push(Event::Alloc(e));
                            }
                            _ => {}
                        });
                        self.stack.push(f);
                    }
                    return Ok(());
                }

                // the function is within another function
                if let Some(mut f) = self.stack.pop() {
                    f.within.push(Event::Func(func.clone()));
                    self.stack.push(f);
                }
            }

            return Ok(());
        }
        bail!("empty stack in exit")
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
            _ => bail!("Could not map log level to an appropriate level"),
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
        .context("Could not cast func_id arg to i32")? as u32;
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
        .context("Could not cast func_id arg to i32")?;

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
        .context("Could not cast amount_in_pages arg to i32")?;

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
        .context("Could not cast format arg to i32")?;

    let format = match format {
        1 => MetricFormat::Statsd,
        _ => bail!("Illegal metric format value"),
    };

    let ptr = input
        .get(1)
        .context("Missing ptr arg")?
        .i64()
        .context("Could not cast ptr arg to i64")?;

    let len = input
        .get(2)
        .context("Missing len arg")?
        .i32()
        .context("Could not cast len arg to i32")?;

    let memory = caller
        .get_export("memory")
        .context("Could not get memory from caller")?
        .into_memory()
        .context("Could not cast to into memory")?;

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
        .i64()
        .context("Could not cast ptr arg to i64")?;

    let len = input
        .get(1)
        .context("Missing len arg")?
        .i32()
        .context("Could not cast len arg to i32")?;

    let memory = caller
        .get_export("memory")
        .context("Could not get memory from caller")?
        .into_memory()
        .context("Could not cast to into memory")?;

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
        .context("Could not cast level arg to i32")?;

    let ptr = input
        .get(1)
        .context("Missing ptr arg")?
        .i64()
        .context("Could not cast ptr arg to i64")?;

    let len = input
        .get(2)
        .context("Missing len arg")?
        .i32()
        .context("Could not cast len arg to i32")?;

    let memory = caller
        .get_export("memory")
        .context("Could not get memory from caller")?
        .into_memory()
        .context("Could not cast to into memory")?;

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
        .i64()
        .context("Could not cast ptr arg to i64")?;

    let len = input
        .get(1)
        .context("Missing len arg")?
        .i32()
        .context("Could not cast len arg to i32")?;

    let memory = caller
        .get_export("memory")
        .context("Could not get memory from caller")?
        .into_memory()
        .context("Could not cast to into memory")?;

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

type EventChannel = (Sender<Event>, Receiver<Event>);

/// Link observability import functions required by instrumented wasm code
pub fn add_to_linker<T: 'static>(
    linker: &mut Linker<T>,
    data: &[u8],
    options: Options,
) -> Result<EventChannel> {
    let (ctx, events_tx, events_rx) = InstrumentationContext::new(options);

    // load the static wasm-instr info
    let wasm_instr_info = WasmInstrInfo::new(data)?;

    // check that the version number is supported with this SDK
    if let Err(e) = wasm_instr_info.check_version() {
        error!("{}", e);
        return Err(e);
    }
    let t = FuncType::new([ValType::I32], []);

    let enter_ctx = ctx.clone();
    linker.func_new(
        "dylibso:observe/instrument",
        "enter",
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
        "dylibso:observe/instrument",
        "exit",
        t.clone(),
        move |_caller, params, results| instrument_exit(params, results, exit_ctx.clone()),
    )?;

    let grow_ctx = ctx.clone();
    linker.func_new(
        "dylibso:observe/instrument",
        "memory-grow",
        t,
        move |_caller, params, results| instrument_memory_grow(params, results, grow_ctx.clone()),
    )?;

    let t = FuncType::new([ValType::I64, ValType::I32], []);

    let span_enter_ctx = ctx.clone();
    linker.func_new(
        "dylibso:observe/api",
        "span-enter",
        t.clone(),
        move |mut caller, params, results| {
            span_enter(&mut caller, params, results, span_enter_ctx.clone())
        },
    )?;

    let span_tags_ctx = ctx.clone();
    linker.func_new(
        "dylibso:observe/api",
        "span-tags",
        t.clone(),
        move |mut caller, params, results| {
            span_tags(&mut caller, params, results, span_tags_ctx.clone())
        },
    )?;

    let t = FuncType::new([ValType::I32, ValType::I64, ValType::I32], []);

    let metric_ctx = ctx.clone();
    linker.func_new(
        "dylibso:observe/api",
        "metric",
        t.clone(),
        move |mut caller, params, results| metric(&mut caller, params, results, metric_ctx.clone()),
    )?;

    let log_ctx = ctx.clone();
    linker.func_new(
        "dylibso:observe/api",
        "log",
        t,
        move |mut caller, params, results| log_write(&mut caller, params, results, log_ctx.clone()),
    )?;

    let t = FuncType::new([], []);

    let span_exit_ctx = ctx.clone();
    linker.func_new(
        "dylibso:observe/api",
        "span-exit",
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

#[cfg(feature = "component-model")]
pub mod component {
    //! # Component Model Observability
    //!
    //! Available via `feature = "component-model"`.
    //!
    //! This module exposes host observability capabilities to the component model. By using this
    //! module, hosts can provide support for guest components which depend on the
    //! `dylibso:observe/api` and `dylibso:observe/instrument` WIT interfaces.
    //!
    //! ## Integrating
    //!
    //! Like Wasmtime's [`wasmtime_wasi::preview2`] module, Observability support involves three steps:
    //!
    //! 1. Adding an [`ObserveSdk`] member to your host's [`wasmtime::Store`] state struct.
    //! 2. Providing access to that member by implementing [`ObserveSdkView`] for your host's state
    //!    struct.
    //! 3. Adding the host bindings to the [`wasmtime::component::Linker`].
    //!
    //! ```rust
    //! use dylibso_observe_sdk::context::component::{ ObserveSdk, ObserveSdkView };
    //!
    //! struct MyState {
    //!   // Step 1: add a member to your state...
    //!   sdk: ObserveSdk,
    //! }
    //!
    //! // Step 2: implement the view trait for your state.
    //! impl ObserveSdkView for MyState {
    //!     fn sdk_mut(&mut self) -> &mut ObserveSdk {
    //!         &mut self.sdk
    //!     }
    //! }
    //! ```
    //!
    //! Once you've completed step 1 and 2, you can add host bindings like so:
    //!
    //! ```no_run
    //! # use dylibso_observe_sdk::context::component::{ ObserveSdk, ObserveSdkView };
    //! # struct MyState {
    //! #  sdk: ObserveSdk,
    //! # }
    //! # impl ObserveSdkView for MyState {
    //! #    fn sdk_mut(&mut self) -> &mut ObserveSdk {
    //! #        &mut self.sdk
    //! #    }
    //! # }
    //! # #[tokio::main]
    //! # async fn main() -> anyhow::Result<()> {
    //! use dylibso_observe_sdk::adapter::otelstdout::OtelStdoutAdapter;
    //!
    //! // (Setup: Read a Wasm module from stdin.)
    //! let args: Vec<_> = std::env::args().skip(1).collect();
    //! let wasm_data = std::fs::read(&args[0])?;
    //! let mut config = wasmtime::Config::new();
    //!
    //! config.async_support(true);
    //! config.wasm_component_model(true);
    //! let engine = wasmtime::Engine::new(&config)?;
    //! let component = wasmtime::component::Component::new(&engine, &wasm_data)?;
    //! let mut linker = wasmtime::component::Linker::new(&engine);
    //!
    //! // All adapters have component support, OtelStdoutAdapter just happens to be
    //! // easiest to use as an example:
    //! let adapter = OtelStdoutAdapter::create();
    //!
    //! // Use the adapter to create observe_sdk bindings which you can then pass to your state.
    //! let observe_sdk = adapter.build_observe_sdk(&wasm_data, Default::default())?;
    //!
    //! // Create your state and wrap it in a wasmtime::Store.
    //! let state = MyState {
    //!     sdk: observe_sdk
    //! };
    //! let mut store = wasmtime::Store::new(&engine, state);
    //!
    //! // ...Then add it to the linker.
    //! dylibso_observe_sdk::context::component::add_to_linker(&mut linker)?;
    //!
    //! # let (cmd, _) = wasmtime_wasi::preview2::command::Command::instantiate_async(&mut store, &component, &linker).await?;
    //! # let run = cmd.wasi_cli_run();
    //! // Once you're done with wasm, call `shutdown()`, which shuts down the collector associated
    //! // with the Observe SDK.
    //! let state = store.into_data();
    //! state.sdk.shutdown().await?;
    //! # Ok(())
    //! # }
    //! ```
    //!
    //! To see an example integrating both Wasi preview 2 and the Observe SDK, see
    //! `rust/examples/otel-stdout-components.rs` in the [observe sdk
    //! repo](https://github.com/dylibso/observe-sdk/).
    use crate::adapter::TraceContext;

    use super::*;
    use wasmtime::component::Linker;

    /// Provide access to [`ObserveSdk`] from a [`wasmtime::Store`]'s inner object.
    ///
    /// ```rust
    /// use dylibso_observe_sdk::context::component::{ ObserveSdk, ObserveSdkView };
    ///
    /// struct MyState {
    ///   sdk: ObserveSdk,
    /// }
    ///
    /// impl ObserveSdkView for MyState {
    ///     fn sdk_mut(&mut self) -> &mut ObserveSdk {
    ///         &mut self.sdk
    ///     }
    /// }
    /// ```
    pub trait ObserveSdkView {
        fn sdk_mut(&mut self) -> &mut ObserveSdk;
    }

    // Hide the bindgen-generated modules from rustdoc by using an "internal" module.
    mod internal {
        wasmtime::component::bindgen!({
            interfaces: r#"
                import dylibso:observe/api;
                import dylibso:observe/instrument;
            "#,
            path: "../wit",
            async: false
        });
    }

    use internal::dylibso::observe::api::{Host as ApiHost, LogLevel, MetricFormat};
    use internal::dylibso::observe::instrument::Host as InstrumentHost;

    /// A data structure backing ObserveSdk bindings: contains private information mapping
    /// function ids to names and contexts for collector channels.
    ///
    /// Instantiate by calling
    /// [`AdapterHandle::build_observe_sdk`](crate::adapter::AdapterHandle::build_observe_sdk)
    /// using any adapter type:
    ///
    /// ```no_run
    /// # use dylibso_observe_sdk::context::component::ObserveSdk;
    /// use dylibso_observe_sdk::adapter::zipkin::ZipkinAdapter;
    /// # fn main() -> anyhow::Result<()> {
    /// let args: Vec<_> = std::env::args().skip(1).collect();
    /// let wasm_data = std::fs::read(&args[0])?;
    /// let zipkin = ZipkinAdapter::create();
    ///
    /// // NB: the ": ObserveSdk" type annotation here isn't necessary, it's just to indicate the
    /// // return type.
    /// let observe_sdk: ObserveSdk = zipkin.build_observe_sdk(&wasm_data,
    /// Default::default()).unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub struct ObserveSdk {
        pub(crate) instr_context: Arc<Mutex<InstrumentationContext>>,
        pub(crate) wasm_instr_info: WasmInstrInfo,
        pub(crate) trace_context: TraceContext,
    }

    impl ObserveSdk {
        /// Shut down the trace collector. Once the collector is shut down this instance
        /// should no longer be used. `shutdown` may be called multiple times but will emit
        /// warnings on subsequent calls.
        pub async fn shutdown(&self) -> Result<()> {
            self.trace_context.shutdown().await;
            Ok(())
        }
    }

    impl TryInto<super::MetricFormat> for MetricFormat {
        type Error = anyhow::Error;

        fn try_into(self) -> std::result::Result<super::MetricFormat, Self::Error> {
            #[allow(unreachable_patterns)]
            match self {
                MetricFormat::Statsd => Ok(super::MetricFormat::Statsd),
                _ => bail!("Illegal metric format value"),
            }
        }
    }

    impl ApiHost for ObserveSdk {
        fn metric(&mut self, format: MetricFormat, name: Vec<u8>) -> wasmtime::Result<()> {
            if let Ok(mut cont) = self.instr_context.lock() {
                cont.metric(format.try_into()?, name.as_slice())?;
            }
            Ok(())
        }

        fn log(&mut self, level: LogLevel, msg: Vec<u8>) -> wasmtime::Result<()> {
            if let Ok(mut cont) = self.instr_context.lock() {
                cont.log_write(level as u8, msg.as_slice())?;
            }
            Ok(())
        }

        fn span_enter(&mut self, name: String) -> wasmtime::Result<()> {
            if let Ok(mut cont) = self.instr_context.lock() {
                cont.enter(0u32, Some(name.as_str()))?;
            }
            Ok(())
        }

        fn span_tags(&mut self, tags: String) -> wasmtime::Result<()> {
            let tags: Vec<String> = tags.split(',').map(|xs| xs.to_string()).collect();
            if let Ok(mut cont) = self.instr_context.lock() {
                cont.span_tags(tags)?;
            }
            Ok(())
        }

        fn span_exit(&mut self) -> wasmtime::Result<()> {
            if let Ok(mut cont) = self.instr_context.lock() {
                cont.exit(0u32)?;
            }
            Ok(())
        }
    }

    impl InstrumentHost for ObserveSdk {
        fn memory_grow(&mut self, amount_in_pages: u32) -> wasmtime::Result<()> {
            if let Ok(mut cont) = self.instr_context.lock() {
                cont.allocate(amount_in_pages)?;
            }
            Ok(())
        }

        fn enter(&mut self, func_id: u32) -> wasmtime::Result<()> {
            let printname = self.wasm_instr_info.function_names.get(&func_id);
            if let Ok(mut cont) = self.instr_context.lock() {
                cont.enter(func_id, printname.map(|x| x.as_str()))?;
            }
            Ok(())
        }

        fn exit(&mut self, func_id: u32) -> wasmtime::Result<()> {
            if let Ok(mut cont) = self.instr_context.lock() {
                cont.exit(func_id)?;
            }
            Ok(())
        }
    }

    /// Make ObserveSdk host bindings available to the component model
    /// [`wasmtime::component::Linker`]. Assumes that [`ObserveSdkView`] has been implemented to
    /// map from the [`wasmtime::Store`] to an instance of [`ObserveSdk`].
    pub fn add_to_linker<T>(linker: &mut Linker<T>) -> Result<()>
    where
        T: ObserveSdkView + 'static,
    {
        internal::dylibso::observe::api::add_to_linker(linker, |s| -> &mut ObserveSdk {
            s.sdk_mut()
        })?;

        internal::dylibso::observe::instrument::add_to_linker(linker, |s| -> &mut ObserveSdk {
            s.sdk_mut()
        })?;
        Ok(())
    }
}
