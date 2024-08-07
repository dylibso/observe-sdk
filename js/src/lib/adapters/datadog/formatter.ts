import {
  Log,
  MemoryGrowAmount,
  Metric,
  MetricFormat,
  Nanoseconds,
  newSpanId,
  newTraceId,
  TelemetryId,
} from "../../mod.ts";
import { DatadogMetadata } from "./mod.ts";

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
  meta?: DatadogMetadata;
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

const allocationKey = 'allocation';

export const addAllocation = (span: Span, amount: MemoryGrowAmount) => {
  let sumAmount = amount;
  let existingAllocation = span.meta[allocationKey];
  if (existingAllocation) {
    try {
      sumAmount = parseInt(existingAllocation) + amount;
    } catch (e) {
      console.error(e);
    }
  }
  span.meta[allocationKey] = sumAmount.toString();
};

export const addMetric = (span: Span, metric: Metric) => {
  if (metric.format !== MetricFormat.StatsdFormat) {
    console.error('cannot add non-statsd metric');
    return;
  }
  const [key, value] = metric.message.split(/:(.*)/);
};

export const addTags = (span: Span, tags: string[]) => {
};

export const addLog = (span: Span, log: Log) => {
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
