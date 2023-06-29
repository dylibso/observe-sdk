use dylibso_observe_sdk::adapter::{datadog::{DatadogAdapter, DatadogConfigBuilder, DatadogMetadataBuilder}, new_trace_id};
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
    let module = wasmtime::Module::new(&engine, &data)?;

    let ddconfig = DatadogConfigBuilder::default().build()?;
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
    let mut trace_ctx = adapter.start(&mut linker, &data).await?;

    let instance = linker.instantiate(&mut store, &module)?;

    // get the function and run it, the events pop into the queue
    // as the function is running

    trace_ctx.set_trace_id(new_trace_id()).await;

    let f = instance
        .get_func(&mut store, function_name)
        .expect("function exists");


    f.call(&mut store, &[], &mut []).unwrap();

    // optionally set metadata
    // this can be set anytime before calling shutdown, but overwrites
    // the whole meta object each time
    let meta = DatadogMetadataBuilder::default()
        .http_url("https://planktonic.com/run?name=mymodule")
        .http_status_code(201u16)
        .http_method("POST")
        .build()?;
    trace_ctx.set_metadata(meta).await;

    // call shutdown to mark the end of the trace
    trace_ctx.shutdown().await;

    Ok(())
}
