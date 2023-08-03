import {
  Nanoseconds,
  MemoryGrowAmount,
  newSpanId,
  newTraceId,
  TelemetryId,
} from "../../mod.ts";

export interface Span {
  trace_id: number;
  span_id: number;
  parent_id?: number;
  name: string;
  start: number;
  duration: number;
  resource: string;
  error: number;
  meta: Map<string, string>;
  metrics: Map<string, number>;
  service: string;
  type?: string;
}

export class Trace {
  spans: Span[];
  trace_id: TelemetryId;

  constructor() {
    this.spans = [];
    this.trace_id = newTraceId();
  }

  toJSON(): Span[] {
    return this.spans;
  }
}

export const addAllocation = (span: Span, amount: MemoryGrowAmount) => {
  span.meta.set("allocation", amount.toString());
};

export class DatadogFormatter {
  constructor(public traces: Trace[]) { }

  public addTrace(trace: Trace) {
    this.traces.push(trace);
  }

  private truncateName(name: string, max: number) {
    const snip = `[...]`;
    if (name.length > max) {
      return `${name.slice(0, max)}${snip}`;
    }

    return name;
  }

  public newSpan(
    serviceName: string,
    traceId: number,
    name: string,
    start: Nanoseconds,
    duration: Nanoseconds,
    parentId?: number,
  ): Span {
    return {
      trace_id: traceId,
      span_id: newSpanId(),
      name: name,
      meta: new Map<string, string>(),
      metrics: new Map<string, number>(),
      start,
      duration,
      resource: name,
      service: serviceName,
      error: 0,
      parent_id: parentId,
    };
  }
}
