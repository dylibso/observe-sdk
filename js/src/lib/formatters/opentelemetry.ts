import { ObserveEvent, newTraceId, newSpanId, FunctionCall, MemoryGrow } from '../mod';
import { TracesData, ResourceSpans, Span, Span_SpanKind } from '../../../proto/opentelemetry/proto/trace/v1/trace'

export class Trace implements TracesData {
    public traceId: number;
    resourceSpans: ResourceSpans[] = [];

    constructor(traceId: number) {
        this.traceId = traceId;
    }
}

export function traceFromEvents(events: ObserveEvent[]): Trace {
    const trace = new Trace(newTraceId());
    const spans: Span[] = [];
    events.forEach((e) => {
        eventToSpans(trace, spans, e);
    })

    trace.resourceSpans = [{
        scopeSpans: [{
            spans: spans,
            scope: null,
            schemaUrl: '',
        }],
        resource: null,
        schemaUrl: '',
    }];
    return trace;
}

function eventToSpans(trace: Trace, spans: Span[], ev: ObserveEvent) {
    if (ev instanceof FunctionCall) {
        const span = newSpan(trace, ev.name || 'unknown-name', ev.startNano(), ev.end);
        spans.push(span);

        ev.within.forEach((e) => {
            eventToSpans(trace, spans, e);
        })
    }
    else if (ev instanceof MemoryGrow) {
        const span = newSpan(trace, 'allocation', ev.start, ev.start);
        span.attributes.push({
            key: 'amount',
            value: {
                intValue: ev.amount,
            }
        })
        spans.push(span);
    }
}

function newSpan(
    trace: Trace,
    name: string,
    start: number,
    end: number,
    parentSpanId?: number,
): Span {
    const spanId = newSpanId();
    const span: Span = {
        traceId: numberToUint8Array(trace.traceId),
        spanId: numberToUint8Array(spanId),
        name,
        kind: Span_SpanKind.SPAN_KIND_INTERNAL, // 
        parentSpanId: parentSpanId ? numberToUint8Array(parentSpanId) : null,
        startTimeUnixNano: start,
        endTimeUnixNano: end,
        attributes: [],
        droppedAttributesCount: 0,
        droppedEventsCount: 0,
        droppedLinksCount: 0,
        traceState: '', // https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/tracestate-handling.md
        events: [],
        links: [],
        status: null,
    };

    return span;
}


function numberToUint8Array(number) {
    // Ensure that the input is a finite number
    if (typeof number !== 'number' || !Number.isFinite(number)) {
        throw new Error('Invalid input. Please provide a finite number.');
    }

    // Convert the number to an 8-byte buffer (64-bit floating-point representation)
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setFloat64(0, number);

    // Create a Uint8Array with a view of the buffer
    const uint8Array = new Uint8Array(buffer);

    // Return the Uint8Array
    return uint8Array;
}
