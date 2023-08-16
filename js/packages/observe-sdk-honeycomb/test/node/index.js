const fs = require("fs");
const { WASI } = require("wasi");
const { env, argv } = require('node:process');
const { HoneycombAdapter } = require("@dylibso/observe-sdk-honeycomb");
require('dotenv').config();

const wasi = new WASI({
  version: "preview1",
  args: argv.slice(1),
  env,
});

const config = {
  apiKey: process.env.HONEYCOMB_API_KEY,
  dataset: 'node',
  emitTracesInterval: 1000,
  traceBatchMax: 100,
  host: 'https://api.honeycomb.io',
}
const adapter = new HoneycombAdapter(config);
const opts = {
  spanFilter: {
    minimumDurationMicroseconds: 100,
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
    // adapter.setMetadata({
    //   http_status_code: 200,
    //   http_url: "https://example.com",
    // });
    traceContext.stop();
  });
});
