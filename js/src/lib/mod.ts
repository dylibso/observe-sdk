export const now = (): NanosFromOrigin => {
  // performance.now is in millis with greater precision than Date.now()
  // https://developer.mozilla.org/en-US/docs/Web/API/Performance/now
  return (performance.now() + performance.timeOrigin) * 1000000;
};

export type Milliseconds = number;
export type NanosFromOrigin = number;
export type ObserveEvent = FunctionCall | MemoryGrow | CustomEvent;
export type MemoryGrowAmount = number;
export type FunctionId = number;
export type NamesMap = Map<FunctionId, string>;

export class CustomEvent {
  constructor(public readonly name: string, public readonly data: any) { }
}

export class MemoryGrow {
  start: Milliseconds;
  constructor(public readonly amount: MemoryGrowAmount) {
    this.start = now();
  }

  public getPages(): MemoryGrowAmount {
    return this.amount;
  }
}

export class FunctionCall {
  start: Milliseconds;
  end: Milliseconds;
  within: Array<ObserveEvent>;

  constructor(
    public readonly name: string,
    public readonly id: FunctionId,
  ) {
    this.start = now();
    this.end = now();;
    this.within = [];
  }

  public stop(time: number | undefined) {
    if (time && time >= this.start) {
      this.end = time;
      return;
    }
    this.end = now();
  }

  public hrDuration(): NanosFromOrigin {
    return this.end - this.start;
  }

  public startNano(): number {
    return 1e6 * (performance.timeOrigin + this.start);
  }

  public duration(): number {
    return Math.ceil(1e6 * this.hrDuration());
  }
}

export interface Formatter {
  format(wasm?: Uint8Array): Promise<Collector>;
}

export interface Collector {
  getImportObject(): WebAssembly.Imports;
  send(to: Adapter): void;
  addMetadata(name: string, value: any): void;
  stop(): void;
}

export interface AdapterConfig {
  emitTracesInterval: number;
}

export abstract class Adapter {
  traceIntervalId: number | undefined = undefined;
  config: AdapterConfig;

  abstract start(wasm?: Uint8Array): Promise<Collector>;

  abstract collect(events: Array<ObserveEvent>): void;

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
