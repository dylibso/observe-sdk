import { HoneycombAdapter } from "@dylibso/observe-sdk-honeycomb";
import { File, OpenFile, WASI } from "@bjorn3/browser_wasi_shim";

const f = async () => {
  const config = {
    apiKey: 'YOUR_API_KEY_HERE',
    dataset: 'web',
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
  const resp = await fetch("test.c.instr.wasm");

  const bytes = await resp.arrayBuffer();
  const traceContext = await adapter.start(bytes, opts);

  let fds = [
    new OpenFile(
      new File(
        new TextEncoder("utf-8").encode(`count these vowels for me please`),
      ),
    ), // stdin
    new OpenFile(new File([])), // stdout
    new OpenFile(new File([])), // stderr
  ];
  let wasi = new WASI([], [], fds);
  const instance = await WebAssembly.instantiate(bytes, {
    "wasi_snapshot_preview1": wasi.wasiImport,
    ...traceContext.getImportObject(),
  });

  wasi.start(instance.instance);
  let utf8decoder = new TextDecoder();
  console.log(utf8decoder.decode(fds[1].file.data));
  traceContext.stop();
};

f().then(() => { });
