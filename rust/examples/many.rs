use dylibso_observe_sdk::adapter::otelstdout::OtelStdoutAdapter;
use rand::{seq::SliceRandom, thread_rng};
use tokio::task;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0])?;
    let function_name = "_start";
    let config = wasmtime::Config::new();

    // Create instance
    let engine = wasmtime::Engine::new(&config)?;
    let module = wasmtime::Module::new(&engine, &data)?;

    // create a thread-safe adapter container, which is used to create trace contexts,
    // one-per-instance of a wasm module.
    let adapter = OtelStdoutAdapter::create();

    for _ in 0..5 {
        let mut instances = Vec::new();
        for _ in 0..5 {
            // Setup WASI
            let wasi_ctx = wasmtime_wasi::WasiCtxBuilder::new()
                .inherit_env()?
                .inherit_stdio()
                .args(&args.clone())?
                .build();

            let mut store = wasmtime::Store::new(&engine, wasi_ctx);
            let mut linker = wasmtime::Linker::new(&engine);
            wasmtime_wasi::add_to_linker(&mut linker, |wasi| wasi)?;

            // Provide the observability functions to the `Linker` to be made
            // available to the instrumented guest code. These are safe to add
            // and are a no-op if guest code is uninstrumented.
            let trace_ctx = adapter.start(&mut linker, &data)?;

            let instance = linker.instantiate(&mut store, &module)?;
            instances.push((trace_ctx, instance, store));
        }

        instances.shuffle(&mut thread_rng());

        for (trace_ctx, instance, mut store) in instances {
            // get the function and run it, the events pop into the queue
            // as the function is running
            tokio::spawn(async move {
                let f = instance
                    .get_func(&mut store, function_name)
                    .expect("function exists");

                f.call(&mut store, &[], &mut []).unwrap();

                if let Err(e) = trace_ctx.shutdown().await {
                    eprintln!("{}", e);
                }
            });
        }
    }

    Ok(())
}

