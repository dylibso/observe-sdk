import { ObserveEvent, newTraceId, newSpanId, FunctionCall, MemoryGrow } from '../mod';
import { TracesData, ResourceSpans, Span, Span_SpanKind } from '../../../proto/opentelemetry/proto/trace/v1/trace'

export {
    TracesData
}
export class Trace implements TracesData {
    public traceId: number;
    resourceSpans: ResourceSpans[] = [];

    constructor(traceId: number) {
        this.traceId = traceId;
    }
}

export function traceFromEvents(serviceName: string, events: ObserveEvent[]): Trace {
    const trace = new Trace(newTraceId());
    const spans: Span[] = [];
    events.forEach((e) => {
        eventToSpans(trace, spans, e);
    })

    trace.resourceSpans = [{
        scopeSpans: [{
            spans: spans,
            scope: undefined,
            schemaUrl: '',
        }],
        resource: {
            attributes: [{
                key: 'service.name',
                value: {
                    stringValue: serviceName
                }
            }],
            droppedAttributesCount: 0,
        },
        schemaUrl: '',
    }];
    return trace;
}

/**
 * 
 * @param trace - all spans created will be tied to this trace
 * @param spans - the list of spans associated with this trace, this is mutated
 * @param ev - the ObserveEvent to convert into spans
 */
function eventToSpans(trace: Trace, spans: Span[], ev: ObserveEvent, parentId?: Uint8Array) {
    if (ev instanceof FunctionCall) {
        const span = newSpan(trace, ev.name || 'unknown-name', ev.start, ev.end, parentId);
        spans.push(span);

        ev.within.forEach((e) => {
            eventToSpans(trace, spans, e, span.spanId);
        })
    }
    else if (ev instanceof MemoryGrow) {
        const span = newSpan(trace, 'allocation', ev.start, ev.start, parentId);
        span.attributes.push({
            key: 'amount',
            value: {
                intValue: ev.amount,
            }
        })
        spans.push(span);
    }
}

/**
 * 
 * @param trace - The trace this span is associated with
 * @param name - Name of the span, either the function's name or 'allocation'
 * @param start - When the span started
 * @param end - When the span ended
 * @param parentSpanId - SpanID of the parent (if any)
 * @returns 
 */
function newSpan(
    trace: Trace,
    name: string,
    start: number,
    end: number,
    parentSpanId?: Uint8Array,
): Span {
    const spanId = newSpanId();
    const span: Span = {
        traceId: numberToUint8Array(trace.traceId),
        spanId: numberToUint8Array(spanId),
        name,
        kind: Span_SpanKind.SPAN_KIND_INTERNAL, // 
        parentSpanId: parentSpanId || new Uint8Array(),
        startTimeUnixNano: start,
        endTimeUnixNano: end,
        attributes: [],
        droppedAttributesCount: 0,
        droppedEventsCount: 0,
        droppedLinksCount: 0,
        traceState: '', // https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/tracestate-handling.md
        events: [],
        links: [],
        status: {
            code: null,
            message: '',
        },
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
