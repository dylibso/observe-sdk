import { Adapter, ObserveEvent, WASM } from "../../mod.ts";
import { SpanCollector } from "../../collectors/span/mod.ts";
import { traceFromEvents, Trace, TracesData } from "../../formatters/opentelemetry.ts";
import { AdapterConfig } from "../../mod.ts";

const defaultConfig: LightstepConfig = {
    apiKey: '',
    serviceName: 'default-service-name',
    emitTracesInterval: 1000,
    traceBatchMax: 100,
    host: 'https://ingest.lightstep.com',
}

export interface LightstepConfig extends AdapterConfig {
    apiKey: string;
    serviceName: string;
    traceBatchMax: number;
    host: string,
}

export class LightstepAdapter extends Adapter {
    config: LightstepConfig = defaultConfig;
    traces: Trace[] = [];

    constructor(config?: LightstepConfig) {
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
        this.traces.push(traceFromEvents(this.config.serviceName, events));
        if (this.traces.length > this.config.traceBatchMax) {
            this.send();
            this.restartTraceInterval();
        }
    }

    private tracesEndpoint() {
        const endpoint = new URL(this.config.host);
        endpoint.pathname = `/traces/otlp/v0.9`;
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
                        "content-type": "application/x-protobuf",
                        "lightstep-access-token": this.config.apiKey,
                    },
                    method: "POST",
                    body: bytes,
                    signal: controller.signal,
                });
                if (!resp.ok) {
                    const msg = await resp.text();
                    console.error(
                        "Request to lightstep failed with status:",
                        resp.status,
                        msg,
                    );
                }
            } catch (e) {
                console.error("Request to lightstep failed:", e);
            } finally {
                clearTimeout(id);
            }
        });
        this.traces = [];
    }
}
