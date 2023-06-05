# Observe

Observe is a Rust observability SDK for WebAssembly. At the moment we support wasmtime hosts and we output opentelemetry
data to stdout. We plan to support more adapters and wasm runtimes in the future. If there is a configuration you are interested in
reach out to support@dylibso.com.

## Overview

There are two components to this process:

1. [Instrumenting the Wasm code](#instrumenting-wasm-module)
2. [Instrumenting the Host code](#instrumenting-host-wasmtime)


## Instrumenting Wasm Module

This package expects the wasm code to be instrumented using our instrumenting compiler. The only way to instrument your wasm right now is through the instrumentation service. The easiest way to do this is to send up your wasm with curl and get an instrumented wasm module back:

```
curl -L -F wasm=@code.wasm https://compiler-preview.dylibso.com/instrument -X POST -H 'Authorization: Bearer <your-api-key>' > code.instr.wasm
```

You can get an API key by contacting support@dylibso.com.

> **Note**: The Instrumentation Service (https://compiler-preview.dylibso.com/instrument) only re-compiles a .wasm binary and returns the updated code. We do not log or store any information about your submitted code. The compilation also adds no telemetry or other information besides the strictly-necessary auto-instrumentation to the .wasm instructions. If you would prefer to run this service yourself, please contact support@dylibso.com to discuss the available options.


## Instrumenting Host Wasmtime

First install the cargo dependency for the SDK:

```toml
[dependencies]
dylibso-observe-sdk = { git = "https://github.com/dylibso/observe-sdk.git" }
```

> **Note**: A runnable example can be found [here](examples/basic.rs).

```rust
use dylibso_observe_sdk::{
    adapter::{stdout::StdoutAdapter, Collector},
    add_to_linker,
};
use tokio::task;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    // ...

    // create our adapter
    let adapter = StdoutAdapter::new();

    // Our magic
    let id = adapter.lock().await.new_collector();
    let events = add_to_linker(id, &mut linker)?;
    let collector = Collector::new(adapter, id, events).await?;

    // instantiate our instance as usual
    let instance = linker.instantiate(&mut store, &module)?;

    let f = instance
        .get_func(&mut store, function_name)
        .expect("function exists");
    f.call(&mut store, &[], &mut []).unwrap();

    // collect the events and shut it down
    task::yield_now().await;
    collector.shutdown().await;

    Ok(())
}
```

## Development

### Building

To build the Wasmtime-based SDK, run:

```
$ cargo build
```

### Testing

```
$ make test
```

### Compile the Test Modules

These are already checked in, but you can compile and instrument them with. Please check in any changes in the test/ directory.

```
make instrument WASM_INSTR_API_KEY=<your-api-key>
```
