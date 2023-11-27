## This guide is for developers of the Observe SDK. If you are just using the SDK you may ignore this document.

# SDK API Versioning

To avoid confusion with users of the Instrumentation Service and the Observe API, we must be careful with API changes. Users may have modules instrumented with an older version of the Instrumentation Service or built with an older version of the API. When possible we should support the older api to avoid breaking those modules. When too painful to maintain support we should at least detect the issue pre-linking and recommend a path forward.

## General How-To
1. If a change is non-breaking, such as an extension of the API maintaining the same signature, just change the function in all the sdks and call it a day.
2. Otherwise, implement the new version of the function with a new name in all the sdks. Generally, the new name would be `old-name-vX` where `X` is the version number. The first `vX` should be `v2`.
3. If possible keep the old function. It is preferred to modify it to use the new implementation when viable rather than maintaining duplicate implementations. **Pre-linking should warn that the old function is deprecated.**
4. If not possible or unwieldy, pre-linking should error out when a module contains a removed function.

## Language specific notes
### Rust
Pre-linking checks are done in `rust/src/wasm_instr.rs` in `WasmInstrInfo::new`
### Go
Pre-linking checks are done in `go/wasm.go` in `parseNames`
### JS
Pre-linking checks are done in `js/src/lib/collectors/span/mod.ts` in `SpanCollector::setNames`

## Observe API SDKs
TBD whether to change the signatures of the prototypes to match the new versions or to add new prototypes for the new versions.

## Wasm-Instr / Instrumentation Service
Bump `WASM_INSTR_VERSION_MAJOR` and `WASM_INSTR_VERSION_MINOR` (and equivalents in Go and JS) when the observe sdk should warn when a module older than those is loaded (Such as when older versions use deprecated or removed functions). This is purely advisory information, the other SDK API Versioning checks handle whether the observe sdk will error out.

