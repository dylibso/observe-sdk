import {
  Adapter,
  AdapterConfig,
  Collector,
  CustomEvent,
  FunctionCall,
  MemoryGrow,
  ObserveEvent,
  Options,
  TelemetryId,
  WASM,
} from "../../mod.ts";
import { SpanCollector } from "../../collectors/span/mod.ts";
import { addAllocation, DatadogFormatter, Trace } from "./formatter.ts";
export enum DatadogTraceType {
  Web = "web",
  Db = "db",
  Cache = "cache",
  Custom = "custom",
}

export enum DatadogSpanKind {
  Server = "server",
  Client = "client",
  Producer = "producer",
  Consumer = "consumer",
  Internal = "internal",
}

export enum DatadogLanguage {
  Cpp = "cpp",
  Dotnet = "dotnet",
  Go = "go",
  Jvm = "jvm",
  Javascript = "javascript",
  Php = "php",
  Ruby = "ruby",
  Python = "python",
}

export interface DatadogConfig extends AdapterConfig {
  agentHost: URL;
  serviceName: string;
  defaultTags: Map<string, string>;
  traceType: DatadogTraceType;
  emitTracesInterval: number;
  traceBatchMax: number;
}

export const DefaultDatadogConfig: DatadogConfig = {
  agentHost: new URL("http://localhost:8126"),
  serviceName: "my-wasm-service",
  defaultTags: new Map<string, string>(),
  traceType: DatadogTraceType.Web,
  emitTracesInterval: 1000,
  traceBatchMax: 100,
};

export interface DatadogMetadata {
  resource_name?: string;
  http_status_code?: number;
  http_url?: string;
  http_method?: string;
  http_client_ip?: string;
  http_request_content_length?: number;
  http_request_content_length_uncompressed?: number;
  http_response_content_length?: number;
  http_response_content_length_uncompressed?: number;
  span_kind?: DatadogSpanKind;
  language?: DatadogLanguage;
  component?: string;
}

export class DatadogTraceContext implements Collector {
  constructor(
    private collector: SpanCollector,
  ) { }
  setMetadata(data: DatadogMetadata): void {
    this.collector.meta = data;
  }
  getImportObject(): WebAssembly.Imports {
    return this.collector.getImportObject();
  }
  send(to: Adapter): void {
    this.collector.send(to);
  }
  addMetadata(name: string, value: string): void {
    this.collector.addMetadata(name, value);
  }
  stop(): void {
    this.collector.stop();
  }

  public setTraceId(traceId: TelemetryId) {
    this.collector.addMetadata("trace_id", traceId.toString());
  }
}

export class DatadogAdapter extends Adapter {
  formatter: DatadogFormatter;
  config: DatadogConfig;
  traceIntervalId: number | undefined | NodeJS.Timer;

  constructor(config?: DatadogConfig) {
    super();
    this.config = DefaultDatadogConfig;
    if (config) {
      this.config = config;
    }
    this.formatter = new DatadogFormatter([]);
  }

  public async start(
    wasm: WASM,
    opts?: Options,
  ): Promise<DatadogTraceContext> {
    const spanCollector = new SpanCollector(this, opts);
    await spanCollector.setNames(wasm);

    this.startTraceInterval();

    return new DatadogTraceContext(spanCollector);
  }

  public collect(events: ObserveEvent[], metadata: any): void {
    const trace = new Trace();
    trace.meta = metadata;
    const traceId = events.find((event) => {
      if (event instanceof CustomEvent) {
        if (event.name === "trace_id") {
          return true;
        }
      }

      return false;
    }) as CustomEvent | undefined;
    if (traceId) {
      trace.trace_id = traceId.data as number;
    }

    events.forEach((event) => {
      if (event instanceof FunctionCall) {
        this.addSpanToTrace(trace, event);
      }
    });
    this.formatter.addTrace(trace);

    if (this.formatter.traces.length >= this.config.traceBatchMax) {
      this.send();
      this.restartTraceInterval();
    }
  }

  private addSpanToTrace(
    trace: Trace,
    fn: FunctionCall,
    parentId?: number,
  ) {
    const span = this.formatter.newSpan(
      this.config.serviceName,
      trace.trace_id,
      fn.name,
      fn.start,
      fn.duration(),
      parentId,
    );
    if (parentId) {
      span.parent_id = parentId;
    }
    trace.spans.push(span);

    fn.within.forEach((event: ObserveEvent) => {
      if (event instanceof FunctionCall) {
        this.addSpanToTrace(trace, event, span.span_id);
      }
      if (event instanceof MemoryGrow) {
        addAllocation(span, event.amount);
      }
    });
  }

  private tracesEndpoint(): URL {
    const endpoint = new URL(this.config.agentHost);
    endpoint.pathname = "v0.3/traces";
    return endpoint;
  }

  async send() {
    if (this.formatter.traces.length > 0) {
      for (const trace of this.formatter.traces) {
        const span = trace.spans[0];
        if (span && trace.meta) {
          if (trace.meta.resource_name) {
            span.meta["resource"] = trace.meta.resource_name;
          }
          if (trace.meta.http_status_code) {
            span.meta["http.status_code"] = trace.meta.http_status_code;
          }
          if (trace.meta.http_url) {
            span.meta["http.url"] = trace.meta.http_url;
          }
          if (trace.meta.http_method) {
            span.meta["http.method"] = trace.meta.http_method;
          }
          if (trace.meta.http_client_ip) {
            span.meta["http.client_ip"] = trace.meta.http_client_ip;
          }
          if (trace.meta.http_request_content_length) {
            span.meta["http.request.content_length"] =
              trace.meta.http_request_content_length;
          }
          if (trace.meta.http_request_content_length_uncompressed) {
            span.meta["http.request.content_length_uncompressed"] =
              trace.meta.http_request_content_length_uncompressed;
          }
          if (trace.meta.http_response_content_length) {
            span.meta["http.response.content_length"] =
              trace.meta.http_response_content_length;
          }
          if (trace.meta.http_response_content_length_uncompressed) {
            span.meta["http.response.content_length_uncompressed"] =
              trace.meta.http_response_content_length_uncompressed;
          }
          if (trace.meta.span_kind) {
            span.meta["span.kind"] = trace.meta.span_kind;
          }
        }
      }

      const controller = new AbortController();
      const id = setTimeout(() => controller.abort(), 1000);
      try {
        const resp = await fetch(this.tracesEndpoint(), {
          headers: {
            "content-type": "application/json",
          },
          method: "PUT",
          body: JSON.stringify(this.formatter.traces),
          signal: controller.signal,
        });
        if (!resp.ok) {
          console.error(
            "Request to datadog agent failed with status:",
            resp.status,
            resp.statusText
          );
        } else {
          this.formatter.traces = [];
        }
      } catch (e) {
        console.error("Request to datadog agent failed:", e);
      } finally {
        clearTimeout(id);
      }
    }
  }
}
