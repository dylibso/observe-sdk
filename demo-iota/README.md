# Iota

Iota is a toy functions-as-a-service platform where all the functions are
represented as Wasm modules.

Iota maps the HTTP body to the Wasm function's standard input. And it maps the
Wasm function's standard output to the HTTP response.

This document describes the high-level make command interface to use and
configure an Iota service, as well as the low-level APIs implemented by each
Iota backend.

## Iota Proxy Service

In order to simplify operation of these Iota services, there is a reverse proxy
deployed that can reach all Iotas through a single entrypoint.

### Upload a module:

`POST https://iota.dylibso.com/upload/<host>/<name>`

Where `<host>` is any of `go`, `rust`, `node`, `deno`, and the `<name>` is the
deployed code to execute.

```sh
cd demo-iota
make iota-upload host=rust wasm=functions/count_vowels.rs.instr.wasm name=count_vowels
```

### Run a module:

`POST https://iota.dylibso.com/run/<host>/<adapter>/<name>`

Where `<host>` is any of `go`, `rust`, `node`, `deno`, `<adapter>` is any
supported APM (`datadog`, `honeycomb`), and the `<name>` is the name is the
deployed code to execute.

```sh
cd demo-iota 
echo "this is a test" | make iota-run host=rust adapter=datadog name=count_vowels
4
```

---

## API

There are only two endpoints:

### `POST /upload?name=<name>`

Post a Wasm module to this endpoint using any name you want to identify the code
to execute:

```bash
curl -F wasm=@count_vowels.wasm  "https://p01--iota-web--tyqfmnr79gjf.code.run/upload?name=count_vowels" -X POST
```

### `POST /run?name=<name>`

Post to this endpoint to run code by name. The HTTP body will get passed to the
function input. The function output will be returned in the HTTP response body

```bash
curl  "https://p01--iota-web--tyqfmnr79gjf.code.run/run?name=count_vowels" -X POST -d "Hello World"
# => 3
```

## Trying it out

We have some already instrumented functions in [functions](functions).

You can upload and run them using the Makefile which just calls the service
using curl:

```bash
# Upload the instrumented wasm file to iota and name it count_vowels
make upload name=count_vowels wasm=functions/count_vowels.rs.instr.wasm

# Pipe the string "hello world" to the count_vowels function we just uploaded
echo "hello world" | make run name="count_vowels"
# => 3
```

## Languages

We have 3 webservers that implement these two endpoints. Each is written in a
different language and utilizes the corresponding Observe SDK:

- [rust](./rust)
  - https://p01--iota-web-tyqfmnr79gjf.code.run/
- [go](./go)
  - https://p01--iota-web-go--tyqfmnr79gjf.code.run/
- js
  - TODO

## Rebuilding Example Functions

You can rebuild and re-instrument the example functions if you have an API key
for the instrumenting compiler:

```bash
make build WASM_INSTR_API_KEY=0a3257906cb81fdb66930b1020b8d0ff
```
