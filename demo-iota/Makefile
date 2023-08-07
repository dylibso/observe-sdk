WASM_INSTR_API_KEY=?
STDIN_FILE := /tmp/makefile_tmp
RS_MODULES := $(wildcard functions/*.rs)

name=""
wasm=""
upload:
	@curl -F wasm=@$(wasm) "https://p01--iota-web--tyqfmnr79gjf.code.run/upload?name=$(name)" -X POST

run:
	@cat > $(STDIN_FILE)
	@curl "https://p01--iota-web--tyqfmnr79gjf.code.run/run?name=$(name)" -X POST -d "$$(cat $(STDIN_FILE))"
	@rm $(STDIN_FILE)

build:
	@for file in $(RS_MODULES); do \
		rustc $$file --target=wasm32-wasi -C opt-level=3 -C debuginfo=0 -o $$file.wasm; \
		curl -F wasm=@$$file.wasm https://compiler-preview.dylibso.com/instrument -X POST -H "Authorization: Bearer $(WASM_INSTR_API_KEY)" > $$file.instr.wasm; \
	done

iota-upload:
	@curl -F wasm=@$(wasm) "https://iota.dylibso.com/upload/$(host)/$(name)" -X POST

iota-run: 
	@cat > $(STDIN_FILE)
	@curl "https://iota.dylibso.com/run/$(host)/$(adapter)/$(name)" -X POST -d "$$(cat $(STDIN_FILE))"
	@rm $(STDIN_FILE)
