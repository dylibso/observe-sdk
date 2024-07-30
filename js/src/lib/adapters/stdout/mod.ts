import { Adapter, FunctionCall, MemoryGrow, Metric, MetricFormat, Log, LogLevel, ObserveEvent, Options, WASM, SpanTags } from "../../mod.ts";
import { SpanCollector } from "../../collectors/span/mod.ts";

export class StdOutAdapter extends Adapter {
  public async start(wasm: WASM, opts?: Options): Promise<SpanCollector> {
    const collector = new SpanCollector(this, opts);
    await collector.setNames(wasm);
    return collector;
  }

  public collect(events: ObserveEvent[]): void {
    events.forEach((ev) => printEvents(ev, 0));
  }

  async send() { }
}

function printEvents(event: ObserveEvent, indentation: number) {
  if (event instanceof FunctionCall) {
    console.log(
      `${"  ".repeat(indentation)
      } Call to ${event.name} took ${event.duration()}ns`,
    );
    event.within.forEach((f) => {
      printEvents(f, indentation + 1);
    });
  }
  if (event instanceof MemoryGrow) {
    console.log(
      `${"  ".repeat(indentation - 1)
      } Allocation grew memory by ${event.getPages()} pages`,
    );
  }
  if (event instanceof Metric) {
    console.log(
      `${"  ".repeat(indentation - 1)
      } metric(${MetricFormat[event.format]}): ${event.message}`,
    );
  }
  if (event instanceof SpanTags) {
    console.log(
      `${"  ".repeat(indentation - 1)
      } tags: ${event.tags.join(', ')}`,
    );
  }
  if (event instanceof Log) {
    console.log(
      `${"  ".repeat(indentation - 1)
      } log(${LogLevel[event.level]}): ${event.message}`,
    );
  }
}
