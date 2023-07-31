export const now = (): hrMillisecondsFromOrigin => {
  return performance.now();
};

export type Milliseconds = number;
export type hrMillisecondsFromOrigin = number;
export type ObserveEvent = FunctionCall | MemoryGrow | CustomEvent;
export type MemoryGrowAmount = number;
export type FunctionId = number;
export type NamesMap = Map<FunctionId, string>;

export class CustomEvent {
  constructor(public readonly name: string, public readonly data: any) {}
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
    this.end = 0;
    this.within = [];
  }

  public stop(time: number | undefined) {
    if (time && time >= this.start) {
      this.end = time;
      return;
    }
    this.end = now();
  }

  public hrDuration(): hrMillisecondsFromOrigin {
    return this.end - this.start;
  }

  public startNano(): number {
    return 1e6 * (performance.timeOrigin + this.start);
  }

  public duration(): number {
    return Math.ceil(1e6 * this.hrDuration());
  }
}

export interface Collector {
  getImportObject(): WebAssembly.Imports;
  send(to: Adapter): void;
  addMetadata(name: string, value: any): void;
  stop(): void;
}

export interface Adapter {
  start(wasm?: Uint8Array): Promise<Collector>;
  collect(events: Array<ObserveEvent>): void;
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
