use dylibso_observe_sdk::{
    adapter::otelstdout::OtelStdoutAdapter,
    context::component::{ObserveSdk, ObserveSdkView},
};

use wasmtime_wasi::preview2::{ResourceTable, WasiCtx, WasiView};

struct State {
    table: ResourceTable,
    wasi_ctx: WasiCtx,
    observe_sdk: ObserveSdk,
}

impl WasiView for State {
    fn table(&self) -> &ResourceTable {
        &self.table
    }

    fn table_mut(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&self) -> &WasiCtx {
        &self.wasi_ctx
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

impl ObserveSdkView for State {
    fn sdk_mut(&mut self) -> &mut ObserveSdk {
        &mut self.observe_sdk
    }
}

wasmtime::component::bindgen!({path: "../corpus/01-component-instr-component/wit", async: true});

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0])?;
    let mut config = wasmtime::Config::new();

    config.async_support(true);
    config.wasm_component_model(true);

    // Create instance
    let engine = wasmtime::Engine::new(&config)?;
    let component = wasmtime::component::Component::new(&engine, &data)?;

    let table = ResourceTable::new();

    // Setup WASI
    let wasi_ctx = wasmtime_wasi::preview2::WasiCtxBuilder::new()
        .inherit_stdio()
        .args(&args.clone())
        .build();

    let mut linker = wasmtime::component::Linker::new(&engine);

    let adapter = OtelStdoutAdapter::create();
    let observe_sdk = adapter.build_observe_sdk(&data, Default::default())?;

    let state = State {
        table,
        wasi_ctx,
        observe_sdk,
    };
    let mut store = wasmtime::Store::new(&engine, state);
    wasmtime_wasi::preview2::command::add_to_linker(&mut linker)?;

    dylibso_observe_sdk::context::component::add_to_linker(&mut linker)?;

    let (component_instr_component, _instance) =
        Example::instantiate_async(&mut store, &component, &linker).await?;

    match component_instr_component.call_hello_world(&mut store).await {
        Ok(result) => println!("hello_world: {:?}", result),
        _ => println!("encountered error"),
    };

    let state = store.into_data();
    state.observe_sdk.shutdown().await?;

    Ok(())
}
