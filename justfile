_help:
  @just --list

build_wit:
  wasm-tools component wit wit/observe.wit -w -o wit/observe.wasm

component_demo:
  #!/bin/bash
  set -eou pipefail
  (cd corpus/00-component-instr-reactor; cargo component build)
  (cd rust/; cargo run -F component-model --example otel-stdout-components ../corpus/00-component-instr-reactor/target/wasm32-wasi/debug/component-instr-reactor.wasm)
