import { Adapter, ObserveEvent, WASM } from "../../mod.ts";
import { SpanCollector } from "../../collectors/span/mod.ts";
import { traceFromEvents, Trace, TracesData } from "../../formatters/opentelemetry.ts";
import { AdapterConfig } from "../../../lib/mod";

const defaultConfig: HoneycombConfig = {
    apiKey: '',
    dataset: 'default-dataset',
    emitTracesInterval: 1000,
    traceBatchMax: 100,
    host: 'https://api.honeycomb.io',
}

export interface HoneycombConfig extends AdapterConfig {
    apiKey: string;
    dataset: string;
    traceBatchMax: number;
    host: string,
}

export class HoneycombAdapter extends Adapter {
    config: HoneycombConfig = defaultConfig;
    traces: Trace[] = [];

    constructor(config?: HoneycombConfig) {
        super();
        if (config) {
            this.config = config;
        }
    }

    public async start(wasm: WASM): Promise<SpanCollector> {
        super.startTraceInterval();
        const collector = new SpanCollector(this);
        await collector.setNames(wasm);
        return collector;
    }

    public collect(events: ObserveEvent[]): void {
        this.traces.push(traceFromEvents(this.config.dataset, events));
        if (this.traces.length > this.config.traceBatchMax) {
            this.send();
            this.restartTraceInterval();
        }
    }

    private tracesEndpoint() {
        const endpoint = new URL(this.config.host);
        endpoint.pathname = `/v1/traces`;
        return endpoint;
    }

    async send() {
        this.traces.forEach(async (trace) => {
            const controller = new AbortController();
            const id = setTimeout(() => controller.abort(), 1000);
            const bytes = TracesData.encode(trace).finish();
            try {
                const resp = await fetch(this.tracesEndpoint(), {
                    headers: {
                        "content-type": "application/protobuf",
                        "x-honeycomb-team": this.config.apiKey,
                    },
                    method: "POST",
                    body: bytes,
                    signal: controller.signal,
                });
                if (!resp.ok) {
                    const msg = await resp.json();
                    console.error(
                        "Request to honeycomb failed with status:",
                        resp.status,
                        msg
                    );
                }
            } catch (e) {
                console.error("Request to honeycomb failed:", e);
            } finally {
                clearTimeout(id);
            }
        });
        this.traces = [];
    }
}
