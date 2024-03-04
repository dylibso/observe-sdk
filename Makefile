WASICC?=$(WASI_SDK_PATH)/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot
WASM_INSTR_API_KEY?=
.NOTINTERMEDIATE:
.PHONY: test
test:
	cargo run --example=basic test/test.c.instr.wasm "Test"
	cargo run --example=many test/test.c.instr.wasm "Test"

C_MODULES := $(wildcard test/*.c)
RS_MODULES := $(wildcard test/*.rs)
ALL_MODULES := $(C_MODULES) $(RS_MODULES)
ALL_MODULES_INSTR_WASM := $(addsuffix .instr.wasm, $(ALL_MODULES))

test/reactor-%.c.wasm: test/reactor-%.c
	$(WASICC) -mexec-model=reactor -o $@ $^

test/%.c.wasm: test/%.c
	$(WASICC) -o $@ $^

test/%.rs.wasm: test/%.rs
	rustc $^ --target=wasm32-wasi -C opt-level=3 -C debuginfo=0 -o $@

test/%.instr.wasm: test/%.wasm
	curl -F wasm=@$^ https://compiler-preview.dylibso.com/instrument -X POST -H "Authorization: Bearer $(WASM_INSTR_API_KEY)" > $@

.PHONY: instrument
instrument: $(ALL_MODULES_INSTR_WASM)

.PHONY: clean
clean:
	rm -f test/*.wasm
