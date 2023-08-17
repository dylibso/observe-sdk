const fs = require("fs");
const { WASI } = require("wasi");
const { env, argv } = require('node:process');
const { LightstepAdapter } = require("@dylibso/observe-sdk-lightstep");
require('dotenv').config();

const wasi = new WASI({
  version: "preview1",
  args: argv.slice(1),
  env,
});

const config = {
  apiKey: process.env.LIGHTSTEP_API_KEY,
  serviceName: 'node',
  emitTracesInterval: 1000,
  traceBatchMax: 100,
  host: 'https://ingest.lightstep.com',
}
const adapter = new LightstepAdapter(config);
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
    // adapter.setMetadata({
    //   http_status_code: 200,
    //   http_url: "https://example.com",
    // });
    traceContext.stop();
  });
});
