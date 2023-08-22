const express = require('express')
const os = require('os')
const multer = require('multer')
const fs = require('fs');
const { WASI } = require('wasi');
const { env, argv } = require('node:process');
const { DatadogAdapter } = require('../../../js/packages/observe-sdk-datadog');

const storage = multer.diskStorage(
    {
        destination: os.tmpdir(),
        filename: (req, _, cb) => {
            cb(null, `${req.query['name']}.wasm`);
        }
    }
)
const upload = multer({ storage })
const app = express()

const config = {
    agentHost: new URL("http://ddagent:8126"),
    serviceName: "iota-node",
    defaultTags: new Map(),
    traceType: "node",
    emitTracesInterval: 1000, // milliseconds
    traceBatchMax: 100
}
const adapter = new DatadogAdapter(config)

app.get('/', (_, res) => {
    res.send('Hi')
})

app.post('/upload', upload.single('wasm'), (req, res) => {
    try {
        const _ = fs.readFileSync(`${os.tmpdir()}/${req.query['name']}.wasm`)
        console.log(`Successfully uploaded ${req.query['name']}.wasm`)
        res.status(200)
    } catch (e) {
        console.error(e)
        res.sendStatus(500)
    }
})

app.post('/run', async (req, res) => {
    try {
        const stdoutPath = `${os.tmpdir}/stdout_${Math.ceil(Math.random() * 10000)}.txt`
        const stdout = fs.openSync(stdoutPath, 'w')
        console.log(stdout)
        const wasi = new WASI({
            version: 'preview1',
            args: argv.slice(1),
            stdout: stdout,
            env,
        })
        const bytes = fs.readFileSync(`${os.tmpdir()}/${req.query['name']}.wasm`)

        const traceContext = await adapter.start(bytes)
        const module = new WebAssembly.Module(bytes)
        const instance = await WebAssembly.instantiate(module, {
            ...wasi.getImportObject(),
            ...traceContext.getImportObject(),
        })
        wasi.start(instance)
        traceContext.setMetadata({
            http_status_code: '200',
            http_url: `${req.protocol}://${req.headers['host']}${req.originalUrl}`,
        })
        traceContext.stop()
        res.status(200)
        res.send(fs.readFileSync(stdoutPath))
        fs.unlinkSync(stdoutPath)
    } catch (e) {
        console.error(e)
        res.sendStatus(500)
    }
})

const port = 3000
app.listen(port, () => {
    console.log(`Listening on port ${port}`)
})