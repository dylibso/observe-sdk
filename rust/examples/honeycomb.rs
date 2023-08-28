use dylibso_observe_sdk::adapter::honeycomb::{
    AdapterMetadata, Attribute, HoneycombAdapter, HoneycombConfig, Options, SpanFilter, Value,
};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0])?;
    let function_name = "_start";
    let config = wasmtime::Config::new();

    // Create instance
    let engine = wasmtime::Engine::new(&config)?;
    let module = wasmtime::Module::new(&engine, &data)?;

    let config = HoneycombConfig {
        api_key: String::from(std::env::var("HONEYCOMB_API_KEY")?),
        host: String::from("https://api.honeycomb.io"),
        dataset: String::from("rust"),
    };
    let adapter = HoneycombAdapter::create(config);

    // Setup WASI
    let wasi_ctx = wasmtime_wasi::WasiCtxBuilder::new()
        .inherit_env()?
        .inherit_stdio()
        .args(&args.clone())?
        .build();

    let mut store = wasmtime::Store::new(&engine, wasi_ctx);
    let mut linker = wasmtime::Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |wasi| wasi)?;

    let options = Options {
        span_filter: SpanFilter {
            min_duration_microseconds: std::time::Duration::from_micros(0),
        },
    };
    // Provide the observability functions to the `Linker` to be made available
    // to the instrumented guest code. These are safe to add and are a no-op
    // if guest code is uninstrumented.
    let trace_ctx = adapter.start(&mut linker, &data, options)?;

    let instance = linker.instantiate(&mut store, &module)?;

    // get the function and run it, the events pop into the queue
    // as the function is running

    let f = instance
        .get_func(&mut store, function_name)
        .expect("function exists");

    f.call(&mut store, &[], &mut []).unwrap();

    let meta: Vec<Attribute> = vec![
        Attribute {
            key: "http.url".into(),
            value: Value {
                string_value: Some("https://example.com/things/123".into()),
                int_value: None,
            },
        },
        Attribute {
            key: "http.client_ip".into(),
            value: Value {
                string_value: Some("23.123.15.145".into()),
                int_value: None,
            },
        },
        Attribute {
            key: "http.status_code".into(),
            value: Value {
                string_value: None,
                int_value: Some(200),
            },
        },
    ];

    trace_ctx
        .set_metadata(AdapterMetadata::OpenTelemetry(meta))
        .await;
    trace_ctx.shutdown().await;

    Ok(())
}
