WASICC?=$(WASI_SDK_PATH)/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot
WASICXX?=$(WASI_SDK_PATH)/bin/clang++ --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot

build:
		@cd test/rust && cargo build --target=wasm32-wasi && cd ../..
		@cp test/rust/target/wasm32-wasi/debug/rust_guest.wasm test/rust_guest.wasm
		$(WASICC) -o test/c_guest.wasm test/c/main.c c/observe_api.c
		$(WASICXX) -o test/cxx_guest.wasm -x c++ test/c/main.c c/observe_api.c