use dylibso_observe_sdk::adapter::stdout::StdoutAdapter;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );
    let args: Vec<_> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0])?;
    let function_name = "hello";
    let config = wasmtime::Config::new();

    // Create instance
    let engine = wasmtime::Engine::new(&config)?;
    let module = wasmtime::Module::new(&engine, &data)?;

    let adapter = StdoutAdapter::create();

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
    let trace_ctx = adapter.start(&mut linker, &data, Default::default())?;

    let instance = linker.instantiate(&mut store, &module)?;

    // call _initialize"
    {
        let f = instance
            .get_func(&mut store, "_initialize")
            .expect("function exists");
        f.call(&mut store, &[], &mut []).unwrap();
    }

    // call hello
    let f = instance
        .get_func(&mut store, function_name)
        .expect("function exists 2");
    f.call(&mut store, &[], &mut []).unwrap();
    trace_ctx.shutdown().await;

    Ok(())
}
