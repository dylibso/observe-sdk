import * as wasm from "./modsurfer-demangle/modsurfer_demangle_bg.wasm";
import { __wbg_set_wasm } from "./modsurfer-demangle/modsurfer_demangle_bg.js";
import { demangle } from "./modsurfer-demangle/modsurfer_demangle.js";
import { parseNameSection } from "../../parser/mod.ts";
import { Microseconds } from "../../mod.ts";

import {
  Adapter,
  Collector,
  CustomEvent,
  FunctionCall,
  FunctionId,
  MemoryGrow,
  MemoryGrowAmount,
  NamesMap,
  now,
  ObserveEvent,
  Options,
  WASM
} from "../../mod.ts";

// @ts-ignore - The esbuild wasm plugin provides a `default` function to initialize the wasm
wasm.default().then((bytes) => __wbg_set_wasm(bytes));
export class SpanCollector implements Collector {
  meta?: any;
  names: NamesMap;
  stack: Array<FunctionCall>;
  events: ObserveEvent[];

  constructor(private adapter: Adapter, private opts: Options = new Options()) {
    this.stack = [];
    this.events = [];
    this.names = new Map<FunctionId, string>();
  }

  setMetadata(data: any): void {
    this.meta = data;
  }

  public async setNames(wasm: WASM) {
    let module = wasm;
    if (!(wasm instanceof WebAssembly.Module)) {
      module = await WebAssembly.compile(wasm)
    }

    const mangledNames = parseNameSection(
      WebAssembly.Module.customSections(module, "name")[0],
    );
    mangledNames.forEach((value, key) => {
      this.names.set(key, demangle(value));
    });
  }

  public send(to: Adapter): void {
    to.collect(this.events, this.meta);
  }

  instrumentEnter = (funcId: FunctionId) => {
    const func = new FunctionCall(
      this.names.get(funcId)!,
      funcId,
    );
    this.stack.push(func);
  };

  instrumentExit = (_funcId: FunctionId) => {
    const end = now();
    const fn = this.stack.pop();
    if (!fn) {
      console.error("no function on stack");
      return;
    }
    fn.stop(end);
    if (fn.duration() * 1e-3 < this.opts.spanFilter.minimumDurationMicroseconds) {
      return;
    }

    if (this.stack.length === 0) {
      this.events.push(fn);
      return;
    }

    const f = this.stack.pop()!;
    f.within.push(fn);
    this.stack.push(f);
  };

  instrumentMemoryGrow = (amount: MemoryGrowAmount) => {
    const ev = new MemoryGrow(amount);
    const fn = this.stack.pop();
    if (!fn) {
      this.events.push(ev);
      return;
    }

    fn.within.push(ev);
    this.stack.push(fn);
  };

  public getImportObject(): WebAssembly.Imports {
    return {
      "dylibso_observe": {
        "instrument_enter": this.instrumentEnter,
        "instrument_exit": this.instrumentExit,
        "instrument_memory_grow": this.instrumentMemoryGrow,
      },
    };
  }

  public addMetadata(name: string, value: string): void {
    this.events.push(new CustomEvent(name, value));
  }

  public stop() {
    this.send(this.adapter);
  }
}
