use dylibso_observe_sdk::{
    adapter::{stdout::StdoutAdapter, Collector},
    add_to_linker,
    instrument_enter,
    instrument_exit
};
use dylibso_observe_sdk::InstrumentationContext;
use tokio::task;
use std::collections::HashMap;
use wasmtime::{Val};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0])?;
    let function_name = "_start";
    let config = wasmtime::Config::new();

    // Create instance
    let engine = wasmtime::Engine::new(&config)?;
    let module = wasmtime::Module::new(&engine, &data)?;

    let adapter = StdoutAdapter::new();

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
    let (ctx, events_rx, events_tx) = InstrumentationContext::new(id);
    let ctx0 = ctx.clone();
    let ctx00 = ctx.clone();;
    let events = add_to_linker(id, &mut linker, &data, ctx, events_rx, events_tx)?;

    let collector = Collector::new(adapter, id, events).await?;

    let instance = linker.instantiate(&mut store, &module)?;

    // get the function and run it, the events pop into the queue
    // as the function is running

    let f = instance
        .get_func(&mut store, function_name)
        .expect("function exists");
    let mut hm = HashMap::<u32, String>::new();
    hm.insert(10, String::from("_start"));
    instrument_enter(&[Val::from(10)], &mut[], ctx0, &hm).unwrap();
    f.call(&mut store, &[], &mut []).unwrap();
    instrument_exit(&[Val::from(10)], &mut[], ctx00).unwrap();
    task::yield_now().await;
    collector.shutdown().await;

    Ok(())
}
