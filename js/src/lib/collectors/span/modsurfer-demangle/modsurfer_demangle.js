import * as wasm from "./modsurfer_demangle_bg.wasm";
import { __wbg_set_wasm, demangle as rawDemangle } from "./modsurfer_demangle_bg.js";

let ModsurferDemangle;
function OnModsurferDemangleInitialized() {
    ModsurferDemangle = null;
}

// wasm is loaded from base64 encoded string
// @ts-ignore - The esbuild wasm plugin provides a `default` function to initialize the wasm
if (typeof wasm.default === "function") {
    // @ts-ignore - The esbuild wasm plugin provides a `default` function to initialize the wasm
    ModsurferDemangle = wasm.default().then((bytes) => __wbg_set_wasm(bytes)).then(OnModsurferDemangleInitialized);
} else {
    // cloudflare workers - wasm imported directly
    // @ts-ignore
    ModsurferDemangle = WebAssembly.instantiate(wasm.default).then((instance) => __wbg_set_wasm(instance.exports)).then(OnModsurferDemangleInitialized);
}

export async function demangle(name) {
    if (ModsurferDemangle) {
        await ModsurferDemangle;
    }
    return rawDemangle(name);
}
