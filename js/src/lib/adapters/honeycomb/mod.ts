import { Adapter, ObserveEvent } from "../../mod.ts";
import { SpanCollector } from "../../collectors/span/mod.ts";
import { traceFromEvents, Trace } from "../../formatters/opentelemetry.ts";

export interface HoneycombConfig {
    apiKey: string;
    dataset: string;
    emitTracesInterval: number;
    traceBatchMax: number;
    host: string,
}

const defaultConfig = {
    apiKey: '',
    dataset: 'default-dataset',
    emitTracesInterval: 1000,
    traceBatchMax: 100,
    host: '',
}

export class HoneycombAdapter implements Adapter {
    config: HoneycombConfig = defaultConfig;
    traceIntervalId: number | undefined = undefined;
    traces: Trace[] = [];

    constructor(config?: HoneycombConfig) {
        if (config) {
            this.config = config;
        }
    }

    public async start(wasm: Uint8Array): Promise<SpanCollector> {
        const collector = new SpanCollector(this);
        await collector.setNames(wasm);
        return collector;
    }

    public collect(events: ObserveEvent[]): void {
        this.traces.push(traceFromEvents(events));
        if (this.traces.length > this.config.traceBatchMax) {
            this.send();
        }
    }

    private tracesEndpoint() {
        const endpoint = new URL(this.config.host);
        endpoint.pathname = `1/batch/${this.config.dataset}`;
        return endpoint;
    }

    private async send() {
        if (this.traces.length > 0) {
            const controller = new AbortController();
            const id = setTimeout(() => controller.abort(), 1000);
            try {
                const resp = await fetch(this.tracesEndpoint(), {
                    headers: {
                        "content-type": "application/json",
                    },
                    method: "PUT",
                    body: JSON.stringify(this.traces),
                    signal: controller.signal,
                });
                if (!resp.ok) {
                    console.error(
                        "Request to honeycomb failed with status:",
                        resp.status,
                    );
                } else {
                    this.traces = [];
                }
            } catch (e) {
                console.error("Request to honeycomb failed:", e);
            } finally {
                clearTimeout(id);
            }
        }
    }
}
