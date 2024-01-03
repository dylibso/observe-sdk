import { HoneycombAdapter } from "@dylibso/observe-sdk-honeycomb";
import { File, OpenFile, WASI } from "@bjorn3/browser_wasi_shim";
import code from './code.wasm';

export interface Env {
	HONEYCOMB_API_KEY: string;
}

export default {
	async fetch(req: Request, env: Env, _ctx: ExecutionContext): Promise<Response> {
		// setup some custom configuration for the adapter
		const config = {
		  // loaded as a Secret defined in the Worker, 
		  // see: https://developers.cloudflare.com/workers/wrangler/configuration/#environmental-variables
		  apiKey: env.HONEYCOMB_API_KEY, 
		  dataset: "cloudflare-worker",
		  emitTracesInterval: 100,
		  traceBatchMax: 100,
		  host: "https://api.honeycomb.io",
		};

		// create a new instance of the adapter with the config, this should be shared across requests
		const adapter = new HoneycombAdapter(config);

		// setup some files for stdin, stdout, and stderr
		let fds = [
		  new OpenFile(
		    new File(
		      new TextEncoder().encode(await req.text()),
		    ),
		  ), // stdin
		  new OpenFile(new File([])), // stdout
		  new OpenFile(new File([])), // stderr
		];

		// instantiate the wasm module
		let wasi = new WASI([], [], fds);

		// start the adapter with the wasm module bytes and options
		const traceContext = await adapter.start(code, {});

		// create a new instance of the wasm module using a new trace context to record observability data
		const instance = await WebAssembly.instantiate(code, {
		  "wasi_snapshot_preview1": wasi.wasiImport,
		  ...traceContext.getImportObject(),
		});

		// execute the module
		wasi.start(instance);
		let dec = new TextDecoder();
		const output = dec.decode(fds[1].file.data);

		traceContext.stop();
		await adapter.send();

		return new Response(output)
	},
};
