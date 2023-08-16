const fs = require("fs");
const { WASI } = require("wasi");
const { StdOutAdapter } = require("@dylibso/observe-sdk-stdout");
const { env, argv } = require('node:process')

const wasi = new WASI({
  version: 'preview1',
  args: argv.slice(1),
  env,
});

const adapter = new StdOutAdapter();
const bytes = fs.readFileSync("../../test-data/test.c.instr.wasm");
adapter.start(bytes).then((traceContext) => {
  const module = new WebAssembly.Module(bytes);

  WebAssembly.instantiate(module, {
    ...wasi.getImportObject(),
    ...traceContext.getImportObject(),
  }).then((instance) => {
    wasi.start(instance);
    traceContext.stop();
  });
});
