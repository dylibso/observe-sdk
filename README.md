# Observe

Observe is an observability SDK for WebAssembly. At the moment we support `wasmtime` hosts and we output opentelemetry
data to stdout. We plan to support more adapters and wasm runtimes in the near future. If there is a configuration you are interested in
reach out to [support@dylibso.com](mailto:support@dylibso.com).

## Overview

There are two components to this process:

1. [Instrumenting the Wasm code](#instrumenting-wasm-module)
2. [Including a runtime/host SDK](#including-a-runtime-sdk)


## Instrumenting Wasm Module

This package expects the wasm code to be instrumented using our instrumenting compiler. The only way to instrument your wasm right now is through the instrumentation service. The easiest way to do this is to send up your wasm with curl and get an instrumented wasm module back:

```
curl -F wasm=@code.wasm https://compiler-preview.dylibso.com/instrument -X POST -H 'Authorization: Bearer <your-api-key>' > code.instr.wasm
```

:key: **You can get an API key by contacting [support@dylibso.com](mailto:support@dylibso.com).**

> **Note**: The Instrumentation Service (https://compiler-preview.dylibso.com/instrument) only re-compiles a .wasm binary and returns the updated code. We do not log or store any information about your submitted code. The compilation also adds no telemetry or other information besides the strictly-necessary auto-instrumentation to the .wasm instructions. If you would prefer to run this service yourself, please contact [support@dylibso.com](mailto:support@dylibso.com) to discuss the available options.


## Including a runtime SDK

First install the cargo dependency for the SDK:

```toml
[dependencies]
dylibso-observe-sdk = { git = "https://github.com/dylibso/observe-sdk.git" }
```

> **Note**: A runnable example can be found [here](examples/basic.rs).

```rust
use dylibso_observe_sdk::{
    adapter::{otelstdout::OtelStdoutAdapter, Collector},
    add_to_linker,
};
use tokio::task;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0])?;
    let function_name = "_start";
    let config = wasmtime::Config::new();

    // Create instance
    let engine = wasmtime::Engine::new(&config)?;
    let module = wasmtime::Module::new(&engine, data)?;

    // let adapter = StdoutAdapter::new();
    let adapter = OtelStdoutAdapter::new();

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
    let events = add_to_linker(id, &mut linker)?;
    let collector = Collector::new(adapter, id, events).await?;
    let instance = linker.instantiate(&mut store, &module)?;

    // get the function and run it, the events pop into the queue
    // as the function is running

    let f = instance
        .get_func(&mut store, function_name)
        .expect("function exists");

    OtelStdoutAdapter::start_trace(
        String::from("your-trace-identifier"),
        function_name.to_string(),
        || {
            f.call(&mut store, &[], &mut []).unwrap();
        },
    );

    task::yield_now().await;
    collector.shutdown().await;

    Ok(())
}
```

## Development

### Building

To build the current wasmtime-based SDK, run:

```
$ cargo build
```

### Testing

```
$ make test
```

### Compile the Test Modules

These are already checked in, but you can compile and instrument them with. Please check in any changes in the `test/` directory.

```
make instrument WASM_INSTR_API_KEY=<your-api-key>
```

### Running Zipkin

One of the test adapters will output to Zipkin, defaulting to one running on localhost.

    docker run -d -p 9411:9411 openzipkin/zipkin
