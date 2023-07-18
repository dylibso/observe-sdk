use core::iter::Extend;
use dylibso_observe_sdk::adapter::otelstdout::OtelStdoutAdapter;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0])?;
    let function_name = "_start";

    // Create module
    let mut store = wasmer::Store::default();
    let module = wasmer::Module::new(&store, &data)?;

    // Create Dylibso Observe adapter and get imports for wasmer host
    let adapter = OtelStdoutAdapter::create();
    let (mut imports, trace_ctx) = adapter.start(&mut store, &data)?;

    // Setup WASI (may not be a requirement for your implementation - only this example)
    let mut wasi_env = wasmer_wasi::WasiEnv::builder("wasmer-example").finalize(&mut store)?;

    // include the wasi imports in the import object
    imports.extend(&wasi_env.import_object(&mut store, &module)?);

    // add the import object to the wasmer host runtime (this now includes the Observe host SDK functions)
    let instance = wasmer::Instance::new(&mut store, &module, &imports)?;

    // initialize wasi for wasmer
    wasi_env.initialize(&mut store, instance.clone())?;

    // get the function and run it, the events pop into the queue
    // as the function is running
    let start = instance.exports.get_function("_start")?;
    start.call(&mut store, &[])?;

    wasi_env.cleanup(&mut store, None);

    trace_ctx.shutdown().await;

    Ok(())
}
