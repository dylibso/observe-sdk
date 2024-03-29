const fs = require("fs");
const { WASI } = require("wasi");
const { env, argv } = require("node:process");
const { DatadogAdapter } = require("@dylibso/observe-sdk-datadog");

const wasi = new WASI({
  version: "preview1",
  args: argv.slice(1),
  env,
});

const adapter = new DatadogAdapter();
const opts = {
  spanFilter: {
    minDurationMicroseconds: 100,
  }
};

const bytes = fs.readFileSync("../../test-data/test.c.instr.wasm");

adapter.start(bytes, opts).then((traceContext) => {
  const module = new WebAssembly.Module(bytes);

  WebAssembly.instantiate(module, {
    ...wasi.getImportObject(),
    ...traceContext.getImportObject(),
  }).then((instance) => {
    wasi.start(instance);
    traceContext.setMetadata({
      http_status_code: 200,
      http_url: "https://example.com",
    });
    traceContext.stop();
  });
});
