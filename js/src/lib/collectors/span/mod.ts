import * as wasm from "./modsurfer-demangle/modsurfer_demangle_bg.wasm";
import { __wbg_set_wasm } from "./modsurfer-demangle/modsurfer_demangle_bg.js";
import { demangle } from "./modsurfer-demangle/modsurfer_demangle.js";
import { parseNameSection } from "../../parser/mod.ts";

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
} from "../../mod.ts";

const initDemangle = () =>
  new Promise(async (resolve, _) => {
    // @ts-ignore - there is indeed a `default` function on `wasm`
    const bytes = await wasm.default();
    __wbg_set_wasm(bytes);
    resolve(true);
  });

export class SpanCollector implements Collector {
  names: NamesMap;
  stack: Array<FunctionCall>;
  events: ObserveEvent[];

  constructor(private adapter: Adapter) {
    this.stack = [];
    this.events = [];
    this.names = new Map<FunctionId, string>();
  }

  public async setNames(wasm: ArrayBuffer | WebAssembly.Module) {
    await initDemangle();

    let module = wasm;
    if (!(wasm instanceof WebAssembly.Module)) {
      module = new WebAssembly.Module(wasm);
    }

    const mangledNames = parseNameSection(WebAssembly.Module.customSections(module, "name")[0]);
    mangledNames.forEach((value, key) => {
      this.names.set(key, demangle(value))
    })
  }

  public send(to: Adapter): void {
    to.collect(this.events);
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
