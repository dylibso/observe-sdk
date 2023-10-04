use dylibso_observe_sdk::{adapter::otelstdout::OtelStdoutAdapter, context::component::ObserveSdkBindings};

use wasmtime::component::Val;
use wasmtime_wasi::preview2::{WasiView, WasiCtx, Table};

struct State {
    table: Table,
    wasi_ctx: WasiCtx,
    observe_sdk_bindings: ObserveSdkBindings,
}

impl WasiView for State {
    fn table(&self) -> &Table {
        &self.table
    }

    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }

    fn ctx(&self) -> &WasiCtx {
        &self.wasi_ctx
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

impl AsMut<ObserveSdkBindings> for State {
    fn as_mut(&mut self) -> &mut ObserveSdkBindings {
        &mut self.observe_sdk_bindings
    }
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0])?;
    let function_name = "hello-world";
    let mut config = wasmtime::Config::new();

    config.async_support(true);
    config.wasm_component_model(true);

    // Create instance
    let engine = wasmtime::Engine::new(&config)?;
    let component = wasmtime::component::Component::new(&engine, &data)?;

    let mut table = Table::new();

    // Setup WASI
    let wasi_ctx = wasmtime_wasi::preview2::WasiCtxBuilder::new()
        .inherit_stdio()
        .args(&args.clone())
        .build(&mut table)?;

    let mut linker = wasmtime::component::Linker::new(&engine);

    let adapter = OtelStdoutAdapter::create();
    // Provide the observability functions to the `Linker` to be made available
    // to the instrumented guest code. These are safe to add and are a no-op
    // if guest code is uninstrumented.
    let (observe_sdk_bindings, trace_ctx) = adapter.create_bindings(&data, Default::default())?;

    let state = State {
        table,
        wasi_ctx,
        observe_sdk_bindings
    };
    let mut store = wasmtime::Store::new(&engine, state);

    wasmtime_wasi::preview2::command::add_to_linker(&mut linker)?;
    dylibso_observe_sdk::context::component::add_to_linker(&mut linker)?;

    let instance = linker.instantiate_async(&mut store, &component).await?;

    // get the function and run it, the events pop into the queue
    // as the function is running

    let mut vals = [Val::U32(0)];
    let f = instance
        .get_func(&mut store, function_name)
        .expect("function exists");

    f.call_async(&mut store, &[], &mut vals).await.unwrap();

    dbg!(vals);
    trace_ctx.shutdown().await;

    Ok(())
}
