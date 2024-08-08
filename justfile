_help:
  @just --list

_edit:
  @$EDITOR {{justfile()}}

local_instr instr_path="../wasm-instr/wasm-instr":
  for i in test/*.c.wasm; do o=${i%.wasm}; {{ instr_path }} $i > $o.instr.wasm; done
