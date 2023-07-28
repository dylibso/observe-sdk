<picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/assets/observe-wasm-light.svg">
    <img alt="WebAssembly Observability - Observe SDK by Dylibso" src=".github/assets/observe-wasm.svg">
</picture>

# WebAssembly Observability Toolkit

Observe provides observability SDKs for WebAssembly, enabling continuous
monitoring of WebAssembly code as it executes within a runtime.

This repository contains the Runtime SDKs and the Adapters necessary to have
live profiling & tracing, and over time will include a complete observability
stack for WebAssembly.

## SDKs and Official Adapters

The table below tracks the supported Runtime SDKs and Adapters for the host
application language that is running a WebAssembly module. The Runtime SDKs link
to a particular WebAssembly runtime, and the Adapter formats the raw telemetry
data to be emitted to a particular output/sink. If you need support for another
Adapter, please open an issue here or email
[support@dylibso.com](mailto:support@dylibso.com).

**Note:** Any supported Runtime SDK can be paired with any Adapter from the same
language.

| Language   | Runtime SDKs                             | Adapters                                                                                                                                  |
| ---------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Rust       | [Wasmtime](/rust)                        | [Datadog](/rust/src/adapter/datadog.rs), [OpenTelemetry (STDOUT)](/rust/src/adapter/otelstdout.rs), [Zipkin](/rust/src/adapter/zipkin.rs) |
| Go         | [Wazero](/go)                            | [Datadog](/go/adapter/datadog/), [OpenTelemetry (STDOUT)](/go/adapter/otel_stdout/)                                                       |
| JavaScript | [Native](/js) (Browser, Node, Deno, Bun) | [Datadog](/js/packages/observe-sdk-datadog)                                                                                               |

_More languages, SDKs, and adapters are coming soon! Reach out to help us
prioritize these additional components
([support@dylibso.com](mailto:support@dylibso.com))._

## Overview

There are two components to this process:

1. [Instrumenting the Wasm code](#instrumenting-wasm-modules)
2. [Including a runtime/host SDK](#including-a-runtime-sdk)

## Instrumenting Wasm Modules

This package expects the wasm code to be instrumented using our instrumenting
compiler. The only way to instrument your wasm right now is through the
instrumentation service. The easiest way to do this is to send up your wasm with
curl and get an instrumented wasm module back:

```
curl -F wasm=@code.wasm https://compiler-preview.dylibso.com/instrument -X POST -H 'Authorization: Bearer <your-api-key>' > code.instr.wasm
```

:key: **You can get an API key by contacting
[support@dylibso.com](mailto:support@dylibso.com).**

> **Note**: The Instrumentation Service
> (https://compiler-preview.dylibso.com/instrument) only re-compiles a .wasm
> binary and returns the updated code. We do not log or store any information
> about your submitted code. The compilation also adds no telemetry or other
> information besides the strictly-necessary auto-instrumentation to the .wasm
> instructions. If you would prefer to run this service yourself, please contact
> [support@dylibso.com](mailto:support@dylibso.com) to discuss the available
> options.

## Including a runtime SDK

This example covers the integration of the Rust SDK for Wasmtime. First install
the cargo dependency for the SDK:

```toml
[dependencies]
dylibso-observe-sdk = { git = "https://github.com/dylibso/observe-sdk.git" }
```

> **Note**: A runnable example can be found
> [here](rust/examples/otel-stdout.rs).

```rust
use dylibso_observe_sdk::adapter::otelstdout::OtelStdoutAdapter;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0])?;
    let function_name = "_start";
    let config = wasmtime::Config::new();

    // Create instance
    let engine = wasmtime::Engine::new(&config)?;
    let module = wasmtime::Module::new(&engine, &data)?;

    let adapter = OtelStdoutAdapter::create();

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

    // get the function and run it, the events pop into the queue
    // as the function is running

    let f = instance
        .get_func(&mut store, function_name)
        .expect("function exists");

    f.call(&mut store, &[], &mut []).unwrap();

    trace_ctx.shutdown().await;

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

These are already checked in, but you can compile and instrument them with.
Please check in any changes in the `test/` directory.

```
make instrument WASM_INSTR_API_KEY=<your-api-key>
```

### Running Zipkin

One of the test adapters will output to Zipkin, defaulting to one running on
localhost.

    docker run -d -p 9411:9411 openzipkin/zipkin
