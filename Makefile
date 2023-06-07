WASICC?=$(WASI_SDK_PATH)/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot
WASM_INSTR_API_KEY=?

.PHONY: test
test:
	cargo run --example=basic test/test.c.instr.wasm "Test"
	cargo run --example=many test/test.c.instr.wasm "Test"

MODULES := $(wildcard test/*.c)
instrument:
	@for file in $(MODULES); do \
		$(WASICC) -o $$file.wasm $$file; \
		curl -F wasm=@$$file.wasm https://compiler-preview.dylibso.com/instrument -X POST -H "Authorization: Bearer $(WASM_INSTR_API_KEY)" > $$file.instr.wasm; \
	done
