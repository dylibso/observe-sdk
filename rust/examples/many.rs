use dylibso_observe_sdk::{adapter::otelstdout::OtelStdoutAdapter, new_trace_id};
use rand::{seq::SliceRandom, thread_rng};

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

        let mut tasks = vec![];
        for (trace_ctx, instance, mut store) in instances {
            trace_ctx.set_trace_id(new_trace_id()).await;
            // get the function and run it, the events pop into the queue
            // as the function is running
            let t = tokio::spawn(async move {
                let f = instance
                    .get_func(&mut store, function_name)
                    .expect("function exists");

                f.call(&mut store, &[], &mut []).unwrap();

                trace_ctx.shutdown().await;
            });
            tasks.push(t);
        }

        // we need to actually await the tasks to make sure they are done
        for t in tasks {
            t.await?;
        }
    }

    Ok(())
}

