use dylibso_observe_sdk::{
    adapter::{datadog::{DatadogAdapter, DatadogConfig}, datadog_formatter::new_trace_id, Collector},
    add_to_linker,
};
use tokio::task;

/// You need the datadog agent running on localhost for this example to work
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0])?;
    let function_name = "_start";
    let config = wasmtime::Config::new();

    // Create instance
    let engine = wasmtime::Engine::new(&config)?;
    let module = wasmtime::Module::new(&engine, data)?;

    let ddconfig = DatadogConfig::new();
    let adapter = DatadogAdapter::new(ddconfig);

    // Setup WASI
    let wasi_ctx = wasmtime_wasi::WasiCtxBuilder::new()
        .inherit_env()?
        .inherit_stdio()
        .args(&args.clone())?
        .build();

    let mut store = wasmtime::Store::new(&engine, wasi_ctx);
    let mut linker = wasmtime::Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |wasi| wasi)?;

    // Provide the observability functions to the `Linker` to be made available
    // to the instrumented guest code. These are safe to add and are a no-op
    // if guest code is uninstrumented.
    let id = adapter.lock().await.new_collector();
    let events = add_to_linker(id, &mut linker)?;

    let collector = Collector::new(adapter.clone(), id, events).await?;

    let instance = linker.instantiate(&mut store, &module)?;

    // get the function and run it, the events pop into the queue
    // as the function is running

    let f = instance
        .get_func(&mut store, function_name)
        .expect("function exists");

    DatadogAdapter::set_trace_id(&collector, new_trace_id()).await;
    f.call(&mut store, &[], &mut []).unwrap();

    task::yield_now().await;
    collector.shutdown().await;

    Ok(())
}
