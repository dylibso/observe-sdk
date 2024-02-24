_help:
  @just --list

_edit:
  @$EDITOR {{justfile()}}

build_wit:
  wasm-tools component wit wit/observe.wit -w -o wit/observe.wasm

component_demo:
  #!/bin/bash
  set -eou pipefail
  (cd corpus/00-component-instr-command; cargo component build)
  (cd rust/; cargo run -F component-model --example otel-stdout-components ../corpus/00-component-instr-command/target/wasm32-wasi/debug/component-instr-command.wasm)

component_demo_2:
  #!/bin/bash
  set -eou pipefail
  (cd corpus/01-component-instr-component; cargo component build)
  (cd corpus/02-component-instr-combined;

  cargo component build
  wasm-tools compose -c config.yml target/wasm32-wasi/debug/combined.wasm -o target/wasm32-wasi/debug/final.wasm
  )
  (cd rust/; cargo run -F component-model --example otel-stdout-components ../corpus/02-component-instr-combined/target/wasm32-wasi/debug/final.wasm)

component_demo_3:
  #!/bin/bash
  set -eou pipefail
  (cd corpus/01-component-instr-component; cargo component build)
  (mkdir -p corpus/01-component-instr-component/wit/deps/observe)
  (cp wit/observe.wit corpus/01-component-instr-component/wit/deps/observe/)
  (cd rust/; cargo run -F component-model --example reactor-hello-world-otel-stdout-components ../corpus/01-component-instr-component/target/wasm32-wasi/debug/component_instr_component.wasm)

local_instr instr_path="../wasm-instr/wasm-instr":
  for i in test/*.c.wasm; do o=${i%.wasm}; {{ instr_path }} $i > $o.instr.wasm; done
