const fs = require("fs");
const { WASI } = require("wasi");
const { env, argv } = require('node:process');
const { HoneycombAdapter } = require("@dylibso/observe-sdk-honeycomb");

const wasi = new WASI({
  version: "preview1",
  args: argv.slice(1),
  env,
});

const adapter = new HoneycombAdapter();

const bytes = fs.readFileSync("../../test-data/test.c.instr.wasm");
adapter.start(bytes).then((traceContext) => {
  const module = new WebAssembly.Module(bytes);

  WebAssembly.instantiate(module, {
    ...wasi.getImportObject(),
    ...traceContext.getImportObject(),
  }).then((instance) => {
    wasi.start(instance);
    adapter.setMetadata({
      http_status_code: 200,
      http_url: "https://example.com",
    });
    traceContext.stop();
  });
});
