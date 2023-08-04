build:
	@cd test/rust && cargo build --target=wasm32-wasi && cd ../..
	@cp test/rust/target/wasm32-wasi/debug/rust_guest.wasm test/rust_guest.wasm
