const esbuild = require('esbuild');
const path = require('path');
const fs = require('fs');
const { program } = require('commander');

let wasmPlugin = {
    name: 'wasm',
    setup(build) {
        // Resolve ".wasm" files to a path with a namespace
        build.onResolve({ filter: /\.wasm$/ }, args => {
            // If this is the import inside the stub module, import the
            // binary itself. Put the path in the "wasm-binary" namespace
            // to tell our binary load callback to load the binary file.
            if (args.namespace === 'wasm-stub') {
                return {
                    path: args.path,
                    namespace: 'wasm-binary',
                }
            }

            // Otherwise, generate the JavaScript stub module for this
            // ".wasm" file. Put it in the "wasm-stub" namespace to tell
            // our stub load callback to fill it with JavaScript.
            //
            // Resolve relative paths to absolute paths here since this
            // resolve callback is given "resolveDir", the directory to
            // resolve imports against.
            if (args.resolveDir === '') {
                return // Ignore unresolvable paths
            }
            return {
                path: path.isAbsolute(args.path) ? args.path : path.join(args.resolveDir, args.path),
                namespace: 'wasm-stub',
            }
        })

        // Virtual modules in the "wasm-stub" namespace are filled with
        // the JavaScript code for compiling the WebAssembly binary. The
        // binary itself is imported from a second virtual module.
        build.onLoad({ filter: /.*/, namespace: 'wasm-stub' }, async (args) => ({
            contents: `import wasm from ${JSON.stringify(args.path)}
          export default (imports) =>
            WebAssembly.instantiate(wasm, imports).then(
              result => result.instance.exports)`,
        }))

        // Virtual modules in the "wasm-binary" namespace contain the
        // actual bytes of the WebAssembly file. This uses esbuild's
        // built-in "binary" loader instead of manually embedding the
        // binary data inside JavaScript code ourselves.
        build.onLoad({ filter: /.*/, namespace: 'wasm-binary' }, async (args) => ({
            contents: await fs.promises.readFile(args.path),
            loader: 'binary',
        }))
    },
}

// parse cli input
program
    .option('-e, --entrypoint <string>')
    .option('-b, --bundle')
    .option('-o, --outfile <string>')
    .option('-p, --platform <string>')
    .option('-f, --format <string>')
    .option('-g, --generateTypes')
    .option('-w, --workers')
program.parse();

const options = program.opts();

// conditional logic for Cloudflare Workers
const plugins = options.workers ? [] : [
    wasmPlugin,
]

// conditional logic for Cloudflare Workers
const loader = options.workers ? {
    '.wasm': 'copy',
} : {}

esbuild.build({
    entryPoints: [options.entrypoint],
    bundle: options.bundle,
    // minify: true,
    allowOverwrite: true,
    outfile: options.outfile,
    platform: options.platform,
    format: options.format,
    loader,
    plugins,
})