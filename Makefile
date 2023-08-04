WASICC?=$(WASI_SDK_PATH)/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot
WASM_INSTR_API_KEY=?

.PHONY: test
test:
	cargo run --example=basic test/test.c.instr.wasm "Test"
	cargo run --example=many test/test.c.instr.wasm "Test"

C_MODULES := $(wildcard test/*.c)
RS_MODULES := $(wildcard test/*.rs)
instrument:
	@for file in $(C_MODULES); do \
		$(WASICC) -o $$file.wasm $$file; \
		curl -F wasm=@$$file.wasm https://compiler-preview.dylibso.com/instrument -X POST -H "Authorization: Bearer $(WASM_INSTR_API_KEY)" > $$file.instr.wasm; \
	done
	@for file in $(RS_MODULES); do \
		rustc $$file --target=wasm32-wasi -C opt-level=3 -C debuginfo=0 -o $$file.wasm; \
		curl -F wasm=@$$file.wasm https://compiler-preview.dylibso.com/instrument -X POST -H "Authorization: Bearer $(WASM_INSTR_API_KEY)" > $$file.instr.wasm; \
	done
	@cd test/rust && cargo build --target=wasm32-unknown-unknown && cd ../..
	@cp test/rust/target/wasm32-unknown-unknown/debug/rust_guest.wasm test/rustapp.wasm
