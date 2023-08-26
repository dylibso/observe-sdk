<p align="center">Checkout the official overview and documentation here:<br/>
<b><a href="https://dev.dylibso.com/docs/observe/overview">https://dev.dylibso.com/docs/observe/overview</a></b>
</p>

---

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

| Language   | Runtime SDKs                             | Adapters                                                                                                                                                                                                                            |
| ---------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust       | [Wasmtime](/rust)                        | [Datadog](/rust/src/adapter/datadog.rs), [Honeycomb](/rust/src/adapter/honeycomb.rs), [Lightstep](/rust/src/adapter/lightstep.rs), [OpenTelemetry (stdout)](/rust/src/adapter/otelstdout.rs), [Zipkin](/rust/src/adapter/zipkin.rs) |
| Go         | [Wazero](/go)                            | [Datadog](/go/adapter/datadog/), [Honeycomb](/go/adapter/honeycomb/), [Lightstep](/go/adapter/lightstep/), [OpenTelemetry (stdout)](/go/adapter/otel_stdout/)                                                                       |
| JavaScript | [Native](/js) (Browser, Node, Deno, Bun) | [Datadog](/js/packages/observe-sdk-datadog), [Honeycomb](/js/packages/observe-sdk-honeycomb), [Lightstep](/js/packages/observe-sdk-lightstep)                                                                                       |

_More languages, SDKs, and adapters are coming soon! Reach out to help us
prioritize these additional components
([support@dylibso.com](mailto:support@dylibso.com))._

## Overview

There are two components to this process:

1. [Including a runtime/host SDK](#including-a-runtime-sdk)
2. [Instrumenting the Wasm code](#instrumenting-wasm-modules)

## Including a runtime SDK and an Adapter

First you should choose a Host SDK corresponding to your host application's
language and Wasm runtime. The Host SDK captures raw observability events from
the running Wasm module and sends them to an adapter. You must choose an adapter
based on where you want your data to go. At the moment, we support a few systems
out of the box. In the future we will support a lot more and will have more
community driven options. If you don't see support for your favorite
observability tools feel free to reach out to us at
([support@dylibso.com](mailto:support@dylibso.com)).

Each language includes some examples demonstrating use with different adapters.
You can view these examples here:

- [Rust](rust/examples)
- [Go](go/bin)
- [Js](js/packages)

## Instrumenting Wasm Modules

There are two ways to instrument the Wasm modules: automatically and manually.

### Automatically instrument your Wasm

The easiest way to instrument your code right now is to use our instrumenting
compiler. This is a tool that can look at your Wasm and recompile it with
instrumentation built in. The compiler is available as a service. You can
generate a key to use
[this service for free here](https://compiler-preview.dylibso.com/).

To use the key:

```
curl --fail -F wasm=@code.wasm https://compiler-preview.dylibso.com/instrument -X POST -H 'Authorization: Bearer <your-api-key>' > code.instr.wasm
```

> **Note**: The Instrumentation Service
> (https://compiler-preview.dylibso.com/instrument) only re-compiles a .wasm
> binary and returns the updated code. We do not log or store any information
> about your submitted code. The compilation also adds no telemetry or other
> information besides the strictly-necessary auto-instrumentation to the .wasm
> instructions. If you would prefer to run this service yourself, please contact
> [support@dylibso.com](mailto:support@dylibso.com) to discuss the available
> options.

### Manually instrument your Wasm

The Host SDKs expose a series of host functions that make up our _Observe API_.
You can code directly against this if you wish. Because we are still changing
and experimenting with this API, we have not built much tooling or support for
this yet. See [the Observe API README](observe-api/) to learn more about the API
and the language bindings we provide.

Expect to see some documentation and alpha tools by September 2023. We will be
building out a lot of the language specific layers, but we hope the community
can help by building tools on top of it and integrating with existing libraries
like OpenTelemetry.

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
