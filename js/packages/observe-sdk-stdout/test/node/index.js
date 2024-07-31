const fs = require("fs");
const { WASI } = require("wasi");
const { StdOutAdapter } = require("@dylibso/observe-sdk-stdout");
//const { StdOutAdapter } = require("../../dist/cjs/index.js");
const { env, argv } = require('node:process')

const bytes = fs.readFileSync("../../../observe-api/test/cxx_guest_2.wasm");

// old (verbose) api
{
  const wasi = new WASI({
    version: 'preview1',
    args: argv.slice(1),
    env,
  });
  const adapter = new StdOutAdapter();
  adapter.start(bytes, {
    spanFilter: {
      minDurationMicroseconds: 0,
    }
  }).then((traceContext) => {
    const module = new WebAssembly.Module(bytes);

    WebAssembly.instantiate(module, {
      ...wasi.getImportObject(),
      ...traceContext.getImportObject(),
    }).then((instance) => {
      traceContext.initSpanEnter(instance.exports.memory.buffer);
      wasi.start(instance);
      traceContext.stop();
    });
  });
}

// new API using instantiateWasm
// SetTimeout as temporary hack as demangling multiple modules at the same time fails:
// TypeError: wasm.__wbindgen_add_to_stack_pointer is not a function
{
  const wasi = new WASI({
    version: 'preview1',
    args: argv.slice(1),
    env,
  });
  const adapter = new StdOutAdapter();
  setTimeout(() => {
    adapter.start(bytes, {
      spanFilter: {
        minDurationMicroseconds: 0,
      },
      instantiateWasm: async (module, traceContext) => {
        return WebAssembly.instantiate(module, {
          ...wasi.getImportObject(),
          ...traceContext.getImportObject(),
        });
      }
    }).then(({ collector: traceContext, instance }) => {
      wasi.start(instance);
      traceContext.stop()
    });
  }, 5000);
}
