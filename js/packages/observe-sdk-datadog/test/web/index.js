import { DatadogAdapter } from "@dylibso/observe-sdk-datadog";
import { File, OpenFile, WASI } from "@bjorn3/browser_wasi_shim";

const f = async () => {
  const adapter = new DatadogAdapter();
  const opts = {
    spanFilter: {
      minimumDurationMicroseconds: 100,
    }
  };
  const resp = await fetch("count_vowels.instr.wasm");

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
