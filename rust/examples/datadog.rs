use dylibso_observe_sdk::adapter::{
    datadog::{DatadogAdapter, DatadogConfig, DatadogMetadata},
    AdapterMetadata,
};

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

    let ddconfig = DatadogConfig::default();
    let adapter = DatadogAdapter::create(ddconfig);

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
    let trace_ctx = adapter.start(&mut linker, &data)?;

    let instance = linker.instantiate(&mut store, &module)?;

    let f = instance
        .get_func(&mut store, function_name)
        .expect("function exists");

    f.call(&mut store, &[], &mut []).unwrap();

    let meta = DatadogMetadata {
        http_url: Some("https://example.com/things/123".into()),
        http_method: Some("GET".into()),
        http_status_code: Some(200u16),
        http_client_ip: Some("23.123.15.145".into()),
        http_request_content_length: Some(128974u64),
        http_response_content_length: Some(239823874u64),
        ..Default::default()
    };

    trace_ctx.set_metadata(AdapterMetadata::Datadog(meta)).await;
    trace_ctx.shutdown().await;

    Ok(())
}
