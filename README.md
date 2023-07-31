<picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/assets/observe-wasm-light.png">
    <img alt="WebAssembly Observability - Observe SDK by Dylibso" width="75%" style="max-width: 600px" src=".github/assets/observe-wasm.png">
</picture>

[![CI](https://github.com/dylibso/observe-sdk/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/dylibso/observe-sdk/actions/workflows/ci.yml)

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

Each language includes some examples demonstrating use with different adapters.
You can view these examples here:

- [Rust](rust/examples)
- [Go](go/bin)
- [Js](js/packages)

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
