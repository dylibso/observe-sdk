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
  Metric,
  MetricFormat,
  NamesMap,
  now,
  ObserveEvent,
  Options,
  SpanTags,
  WASM
} from "../../mod.ts";

// wasm is loaded from base64 encoded string
// @ts-ignore - The esbuild wasm plugin provides a `default` function to initialize the wasm
if (typeof wasm.default === "function") {
  // @ts-ignore - The esbuild wasm plugin provides a `default` function to initialize the wasm
  wasm.default().then((bytes) => __wbg_set_wasm(bytes));
} else {
  // cloudflare workers - wasm imported directly
  // @ts-ignore
  WebAssembly.instantiate(wasm.default).then((instance) => __wbg_set_wasm(instance.exports));
}

export class SpanCollector implements Collector {
  meta?: any;
  names: NamesMap;
  stack: Array<FunctionCall>;
  events: ObserveEvent[];
  memoryBuffer?: Uint8Array;
  textDecoder: TextDecoder;

  constructor(private adapter: Adapter, private opts: Options = new Options()) {
    this.stack = [];
    this.events = [];
    this.names = new Map<FunctionId, string>();
    this.textDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: false });
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

    let warnOnDylibsoObserve = true;
    for (const iName of WebAssembly.Module.imports(module)) {
      if (iName.module === 'dylibso_observe') {
        if (warnOnDylibsoObserve) {
          warnOnDylibsoObserve = false;
          console.warn("Module uses deprecated namespace \"dylibso_observe\"!\n" +
            "Please consider reinstrumenting with newer wasm-instr!");
        }
        const apiNames = new Set(["span_enter", "span_tags", "metric", "log", "span_exit"]);
        if (apiNames.has(iName.name)) {
          throw new Error("js sdk does not yet support Observe API");
        }
        //} else if (iName.module === 'dylibso:observe/api') {
        //  throw new Error("js sdk does not yet support Observe API");
        //}
      }
    }
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

  public initSpanEnter(memoryBuffer: ArrayBuffer | SharedArrayBuffer): void {
    this.memoryBuffer = new Uint8Array(memoryBuffer);
  };

  spanEnter = (namePtr: number, nameLength: number) => {
    if (!this.memoryBuffer) {
      throw new Error("Call initSpanEnter first!");
    }
    const name = this.textDecoder.decode(
      this.memoryBuffer.subarray(namePtr, namePtr + nameLength)
    );
    const func = new FunctionCall(
      name
    );
    this.stack.push(func);
  };

  instrumentExit = (_funcId: FunctionId) => {
    this.exitImpl();
  };

  spanExit = () => {
    this.exitImpl();
  }

  exitImpl = () => {
    const end = now();
    const fn = this.stack.pop();
    if (!fn) {
      console.error("no function on stack");
      return;
    }
    fn.stop(end);

    // if the stack length is 0, we are exiting the root function of the trace
    if (this.stack.length === 0) {
      this.events.push(fn);
      return;
    }

    // if the function duration is less than minimum duration, disregard
    const funcDuration = fn.duration() * 1e-3;
    const minSpanDuration = this.opts.spanFilter?.minDurationMicroseconds ?? 0;
    if (funcDuration < minSpanDuration) {
      // check for memory allocations and attribute them to the parent span before filtering
      const f = this.stack.pop();
      if (f) {
        fn.within.forEach((ev) => {
          if (ev instanceof MemoryGrow) {
            f.within.push(ev);
          }
        });
        this.stack.push(f);
      }
      return;
    }

    // the function is within another function
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

  spanMetric = (format: MetricFormat, messagePtr: number, messageLength: number) => {
    if (!this.memoryBuffer) {
      throw new Error("Call initSpanEnter first!");
    }
    const message = this.textDecoder.decode(
      this.memoryBuffer.subarray(messagePtr, messagePtr + messageLength)
    );
    const ev = new Metric(format, message);
    const fn = this.stack.pop();
    if (!fn) {
      this.events.push(ev);
      return;
    }

    fn.within.push(ev);
    this.stack.push(fn);
  };

  spanTags = (messagePtr: number, messageLength: number) => {
    if (!this.memoryBuffer) {
      throw new Error("Call initSpanEnter first!");
    }
    const message = this.textDecoder.decode(
      this.memoryBuffer.subarray(messagePtr, messagePtr + messageLength)
    );
    const tags = message.split(',');
    const ev = new SpanTags(tags);
    const fn = this.stack.pop();
    if (!fn) {
      this.events.push(ev);
      return;
    }

    fn.within.push(ev);
    this.stack.push(fn);
  };

  /*
  enum DO_LOG_LEVEL {
        DO_LL_ERROR = 1,
        DO_LL_WARN = 2,
        DO_LL_INFO = 3,
        DO_LL_DEBUG = 4,
        DO_LL_TRACE = 5
      };
  */

  public getImportObject(): WebAssembly.Imports {
    return {
      "dylibso:observe/instrument": {
        "enter": this.instrumentEnter,
        "exit": this.instrumentExit,
        "memory-grow": this.instrumentMemoryGrow,
      },
      "dylibso:observe/api": {
        "span-enter": this.spanEnter,
        "span-exit": this.spanExit,
        "metric": this.spanMetric,
        "span-tags": this.spanTags,
        "log": () => { },
      },
      // old (deprecated apis)
      "dylibso_observe": {
        "instrument_enter": this.instrumentEnter,
        "instrument_exit": this.instrumentExit,
        "instrument_memory_grow": this.instrumentMemoryGrow,
        "span_enter": this.spanEnter,
        "span_exit": this.spanExit,
        "metric": this.spanMetric,
        "span_tags": this.spanTags
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
