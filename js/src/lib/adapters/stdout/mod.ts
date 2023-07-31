import { Adapter, FunctionCall, MemoryGrow, ObserveEvent } from "../../mod.ts";
import { SpanCollector } from "../../collectors/span/mod.ts";

export class StdOutAdapter implements Adapter {
  public async start(wasm: Uint8Array): Promise<SpanCollector> {
    const collector = new SpanCollector(this);
    await collector.setNames(wasm);
    return collector;
  }

  public collect(events: ObserveEvent[]): void {
    events.forEach((ev) => printEvents(ev, 0));
  }
}

function printEvents(event: ObserveEvent, indentation: number) {
  if (event instanceof FunctionCall) {
    console.log(
      `${
        "  ".repeat(indentation)
      } Call to ${event.name} took ${event.hrDuration()}ms`,
    );
    event.within.forEach((f) => {
      printEvents(f, indentation + 1);
    });
  }
  if (event instanceof MemoryGrow) {
    console.log(
      `${
        "  ".repeat(indentation - 1)
      } Allocation grew memory by ${event.getPages()} pages`,
    );
  }
}
