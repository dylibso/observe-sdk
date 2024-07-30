export const now = (): Nanoseconds => {
  // performance.now is in millis with greater precision than Date.now()
  // https://developer.mozilla.org/en-US/docs/Web/API/Performance/now
  return (performance.now() + performance.timeOrigin) * 1000000;
};

export type WASM = Uint8Array | WebAssembly.Module;
export type Nanoseconds = number;
export type Microseconds = number;
export type ObserveEvent = FunctionCall | MemoryGrow | CustomEvent;
export type MemoryGrowAmount = number;
export type FunctionId = number;
export type NamesMap = Map<FunctionId, string>;

export class CustomEvent {
  constructor(public readonly name: string, public readonly data: any) { }
}

export class MemoryGrow {
  start: Nanoseconds;
  constructor(public readonly amount: MemoryGrowAmount) {
    this.start = now();
  }

  public getPages(): MemoryGrowAmount {
    return this.amount;
  }
}

export class FunctionCall {
  start: Nanoseconds;
  end: Nanoseconds;
  within: Array<ObserveEvent>;

  constructor(
    public readonly name: string,
    public readonly id?: FunctionId,
  ) {
    this.start = now();
    this.end = this.start;
    this.within = [];
  }

  public stop(time: number | undefined) {
    if (time && time >= this.start) {
      this.end = time;
      return;
    }
    this.end = now();
  }

  public duration(): Nanoseconds {
    return this.end - this.start;
  }
}

export interface Formatter {
  format(wasm?: Uint8Array): Promise<Collector>;
}

export interface Collector {
  getImportObject(): WebAssembly.Imports;
  send(to: Adapter): void;
  addMetadata(name: string, value: any): void;
  setMetadata(data: any): void;
  stop(): void;
}

export interface AdapterConfig {
  emitTracesInterval: number;
}

export abstract class Adapter {
  traceIntervalId: number | undefined | NodeJS.Timer = undefined;
  config: AdapterConfig;

  abstract start(wasm: WASM, opts?: Options): Promise<Collector>;

  abstract collect(events: Array<ObserveEvent>, metadata: any): void;

  abstract send?();

  restartTraceInterval() {
    if (this.traceIntervalId) {
      clearInterval(this.traceIntervalId);
      this.traceIntervalId = undefined;
    }

    this.startTraceInterval();
  }

  startTraceInterval() {
    // @ts-ignore - return value of setInterval is definitely a `number`
    this.traceIntervalId = setInterval(
      async () => await this.send(),
      this.config.emitTracesInterval,
    );
  }
}

export type TelemetryId = number;

export const newTelemetryId = (): TelemetryId => {
  return Math.ceil(Math.random() * Number.MAX_SAFE_INTEGER);
};

export const newSpanId = (): TelemetryId => {
  return newTelemetryId();
};

export const newTraceId = (): TelemetryId => {
  return newTelemetryId();
};
export interface SpanFilter {
  minDurationMicroseconds: Microseconds
}

export class Options {
  spanFilter: SpanFilter = {
    minDurationMicroseconds: 20
  }
}
