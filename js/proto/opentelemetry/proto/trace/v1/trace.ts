/* eslint-disable */
import * as _m0 from "protobufjs/minimal";
import { InstrumentationScope, KeyValue } from "../../common/v1/common";
import { Resource } from "../../resource/v1/resource";
import Long = require("long");

export const protobufPackage = "opentelemetry.proto.trace.v1";

/**
 * TracesData represents the traces data that can be stored in a persistent storage,
 * OR can be embedded by other protocols that transfer OTLP traces data but do
 * not implement the OTLP protocol.
 *
 * The main difference between this message and collector protocol is that
 * in this message there will not be any "control" or "metadata" specific to
 * OTLP protocol.
 *
 * When new fields are added into this message, the OTLP request MUST be updated
 * as well.
 */
export interface TracesData {
  /**
   * An array of ResourceSpans.
   * For data coming from a single resource this array will typically contain
   * one element. Intermediary nodes that receive data from multiple origins
   * typically batch the data before forwarding further and in that case this
   * array will contain multiple elements.
   */
  resourceSpans: ResourceSpans[];
}

/** A collection of ScopeSpans from a Resource. */
export interface ResourceSpans {
  /**
   * The resource for the spans in this message.
   * If this field is not set then no resource info is known.
   */
  resource:
  | Resource
  | undefined;
  /** A list of ScopeSpans that originate from a resource. */
  scopeSpans: ScopeSpans[];
  /**
   * This schema_url applies to the data in the "resource" field. It does not apply
   * to the data in the "scope_spans" field which have their own schema_url field.
   */
  schemaUrl: string;
}

/** A collection of Spans produced by an InstrumentationScope. */
export interface ScopeSpans {
  /**
   * The instrumentation scope information for the spans in this message.
   * Semantically when InstrumentationScope isn't set, it is equivalent with
   * an empty instrumentation scope name (unknown).
   */
  scope:
  | InstrumentationScope
  | undefined;
  /** A list of Spans that originate from an instrumentation scope. */
  spans: Span[];
  /** This schema_url applies to all spans and span events in the "spans" field. */
  schemaUrl: string;
}

/**
 * A Span represents a single operation performed by a single component of the system.
 *
 * The next available field id is 17.
 */
export interface Span {
  /**
   * A unique identifier for a trace. All spans from the same trace share
   * the same `trace_id`. The ID is a 16-byte array. An ID with all zeroes OR
   * of length other than 16 bytes is considered invalid (empty string in OTLP/JSON
   * is zero-length and thus is also invalid).
   *
   * This field is required.
   */
  traceId: Uint8Array;
  /**
   * A unique identifier for a span within a trace, assigned when the span
   * is created. The ID is an 8-byte array. An ID with all zeroes OR of length
   * other than 8 bytes is considered invalid (empty string in OTLP/JSON
   * is zero-length and thus is also invalid).
   *
   * This field is required.
   */
  spanId: Uint8Array;
  /**
   * trace_state conveys information about request position in multiple distributed tracing graphs.
   * It is a trace_state in w3c-trace-context format: https://www.w3.org/TR/trace-context/#tracestate-header
   * See also https://github.com/w3c/distributed-tracing for more details about this field.
   */
  traceState: string;
  /**
   * The `span_id` of this span's parent span. If this is a root span, then this
   * field must be empty. The ID is an 8-byte array.
   */
  parentSpanId: Uint8Array;
  /**
   * A description of the span's operation.
   *
   * For example, the name can be a qualified method name or a file name
   * and a line number where the operation is called. A best practice is to use
   * the same display name at the same call point in an application.
   * This makes it easier to correlate spans in different traces.
   *
   * This field is semantically required to be set to non-empty string.
   * Empty value is equivalent to an unknown span name.
   *
   * This field is required.
   */
  name: string;
  /**
   * Distinguishes between spans generated in a particular context. For example,
   * two spans with the same name may be distinguished using `CLIENT` (caller)
   * and `SERVER` (callee) to identify queueing latency associated with the span.
   */
  kind: Span_SpanKind;
  /**
   * start_time_unix_nano is the start time of the span. On the client side, this is the time
   * kept by the local machine where the span execution starts. On the server side, this
   * is the time when the server's application handler starts running.
   * Value is UNIX Epoch time in nanoseconds since 00:00:00 UTC on 1 January 1970.
   *
   * This field is semantically required and it is expected that end_time >= start_time.
   */
  startTimeUnixNano: number;
  /**
   * end_time_unix_nano is the end time of the span. On the client side, this is the time
   * kept by the local machine where the span execution ends. On the server side, this
   * is the time when the server application handler stops running.
   * Value is UNIX Epoch time in nanoseconds since 00:00:00 UTC on 1 January 1970.
   *
   * This field is semantically required and it is expected that end_time >= start_time.
   */
  endTimeUnixNano: number;
  /**
   * attributes is a collection of key/value pairs. Note, global attributes
   * like server name can be set using the resource API. Examples of attributes:
   *
   *     "/http/user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_14_2) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/71.0.3578.98 Safari/537.36"
   *     "/http/server_latency": 300
   *     "example.com/myattribute": true
   *     "example.com/score": 10.239
   *
   * The OpenTelemetry API specification further restricts the allowed value types:
   * https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/common/README.md#attribute
   * Attribute keys MUST be unique (it is not allowed to have more than one
   * attribute with the same key).
   */
  attributes: KeyValue[];
  /**
   * dropped_attributes_count is the number of attributes that were discarded. Attributes
   * can be discarded because their keys are too long or because there are too many
   * attributes. If this value is 0, then no attributes were dropped.
   */
  droppedAttributesCount: number;
  /** events is a collection of Event items. */
  events: Span_Event[];
  /**
   * dropped_events_count is the number of dropped events. If the value is 0, then no
   * events were dropped.
   */
  droppedEventsCount: number;
  /**
   * links is a collection of Links, which are references from this span to a span
   * in the same or different trace.
   */
  links: Span_Link[];
  /**
   * dropped_links_count is the number of dropped links after the maximum size was
   * enforced. If this value is 0, then no links were dropped.
   */
  droppedLinksCount: number;
  /**
   * An optional final status for this span. Semantically when Status isn't set, it means
   * span's status code is unset, i.e. assume STATUS_CODE_UNSET (code = 0).
   */
  status: Status | undefined;
}

/**
 * SpanKind is the type of span. Can be used to specify additional relationships between spans
 * in addition to a parent/child relationship.
 */
export enum Span_SpanKind {
  /**
   * SPAN_KIND_UNSPECIFIED - Unspecified. Do NOT use as default.
   * Implementations MAY assume SpanKind to be INTERNAL when receiving UNSPECIFIED.
   */
  SPAN_KIND_UNSPECIFIED = 0,
  /**
   * SPAN_KIND_INTERNAL - Indicates that the span represents an internal operation within an application,
   * as opposed to an operation happening at the boundaries. Default value.
   */
  SPAN_KIND_INTERNAL = 1,
  /**
   * SPAN_KIND_SERVER - Indicates that the span covers server-side handling of an RPC or other
   * remote network request.
   */
  SPAN_KIND_SERVER = 2,
  /** SPAN_KIND_CLIENT - Indicates that the span describes a request to some remote service. */
  SPAN_KIND_CLIENT = 3,
  /**
   * SPAN_KIND_PRODUCER - Indicates that the span describes a producer sending a message to a broker.
   * Unlike CLIENT and SERVER, there is often no direct critical path latency relationship
   * between producer and consumer spans. A PRODUCER span ends when the message was accepted
   * by the broker while the logical processing of the message might span a much longer time.
   */
  SPAN_KIND_PRODUCER = 4,
  /**
   * SPAN_KIND_CONSUMER - Indicates that the span describes consumer receiving a message from a broker.
   * Like the PRODUCER kind, there is often no direct critical path latency relationship
   * between producer and consumer spans.
   */
  SPAN_KIND_CONSUMER = 5,
  UNRECOGNIZED = -1,
}

export function span_SpanKindFromJSON(object: any): Span_SpanKind {
  switch (object) {
    case 0:
    case "SPAN_KIND_UNSPECIFIED":
      return Span_SpanKind.SPAN_KIND_UNSPECIFIED;
    case 1:
    case "SPAN_KIND_INTERNAL":
      return Span_SpanKind.SPAN_KIND_INTERNAL;
    case 2:
    case "SPAN_KIND_SERVER":
      return Span_SpanKind.SPAN_KIND_SERVER;
    case 3:
    case "SPAN_KIND_CLIENT":
      return Span_SpanKind.SPAN_KIND_CLIENT;
    case 4:
    case "SPAN_KIND_PRODUCER":
      return Span_SpanKind.SPAN_KIND_PRODUCER;
    case 5:
    case "SPAN_KIND_CONSUMER":
      return Span_SpanKind.SPAN_KIND_CONSUMER;
    case -1:
    case "UNRECOGNIZED":
    default:
      return Span_SpanKind.UNRECOGNIZED;
  }
}

export function span_SpanKindToJSON(object: Span_SpanKind): string {
  switch (object) {
    case Span_SpanKind.SPAN_KIND_UNSPECIFIED:
      return "SPAN_KIND_UNSPECIFIED";
    case Span_SpanKind.SPAN_KIND_INTERNAL:
      return "SPAN_KIND_INTERNAL";
    case Span_SpanKind.SPAN_KIND_SERVER:
      return "SPAN_KIND_SERVER";
    case Span_SpanKind.SPAN_KIND_CLIENT:
      return "SPAN_KIND_CLIENT";
    case Span_SpanKind.SPAN_KIND_PRODUCER:
      return "SPAN_KIND_PRODUCER";
    case Span_SpanKind.SPAN_KIND_CONSUMER:
      return "SPAN_KIND_CONSUMER";
    case Span_SpanKind.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

/**
 * Event is a time-stamped annotation of the span, consisting of user-supplied
 * text description and key-value pairs.
 */
export interface Span_Event {
  /** time_unix_nano is the time the event occurred. */
  timeUnixNano: number;
  /**
   * name of the event.
   * This field is semantically required to be set to non-empty string.
   */
  name: string;
  /**
   * attributes is a collection of attribute key/value pairs on the event.
   * Attribute keys MUST be unique (it is not allowed to have more than one
   * attribute with the same key).
   */
  attributes: KeyValue[];
  /**
   * dropped_attributes_count is the number of dropped attributes. If the value is 0,
   * then no attributes were dropped.
   */
  droppedAttributesCount: number;
}

/**
 * A pointer from the current span to another span in the same trace or in a
 * different trace. For example, this can be used in batching operations,
 * where a single batch handler processes multiple requests from different
 * traces or when the handler receives a request from a different project.
 */
export interface Span_Link {
  /**
   * A unique identifier of a trace that this linked span is part of. The ID is a
   * 16-byte array.
   */
  traceId: Uint8Array;
  /** A unique identifier for the linked span. The ID is an 8-byte array. */
  spanId: Uint8Array;
  /** The trace_state associated with the link. */
  traceState: string;
  /**
   * attributes is a collection of attribute key/value pairs on the link.
   * Attribute keys MUST be unique (it is not allowed to have more than one
   * attribute with the same key).
   */
  attributes: KeyValue[];
  /**
   * dropped_attributes_count is the number of dropped attributes. If the value is 0,
   * then no attributes were dropped.
   */
  droppedAttributesCount: number;
}

/**
 * The Status type defines a logical error model that is suitable for different
 * programming environments, including REST APIs and RPC APIs.
 */
export interface Status {
  /** A developer-facing human readable error message. */
  message: string;
  /** The status code. */
  code: Status_StatusCode;
}

/**
 * For the semantics of status codes see
 * https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/api.md#set-status
 */
export enum Status_StatusCode {
  /** STATUS_CODE_UNSET - The default status. */
  STATUS_CODE_UNSET = 0,
  /**
   * STATUS_CODE_OK - The Span has been validated by an Application developer or Operator to
   * have completed successfully.
   */
  STATUS_CODE_OK = 1,
  /** STATUS_CODE_ERROR - The Span contains an error. */
  STATUS_CODE_ERROR = 2,
  UNRECOGNIZED = -1,
}

export function status_StatusCodeFromJSON(object: any): Status_StatusCode {
  switch (object) {
    case 0:
    case "STATUS_CODE_UNSET":
      return Status_StatusCode.STATUS_CODE_UNSET;
    case 1:
    case "STATUS_CODE_OK":
      return Status_StatusCode.STATUS_CODE_OK;
    case 2:
    case "STATUS_CODE_ERROR":
      return Status_StatusCode.STATUS_CODE_ERROR;
    case -1:
    case "UNRECOGNIZED":
    default:
      return Status_StatusCode.UNRECOGNIZED;
  }
}

export function status_StatusCodeToJSON(object: Status_StatusCode): string {
  switch (object) {
    case Status_StatusCode.STATUS_CODE_UNSET:
      return "STATUS_CODE_UNSET";
    case Status_StatusCode.STATUS_CODE_OK:
      return "STATUS_CODE_OK";
    case Status_StatusCode.STATUS_CODE_ERROR:
      return "STATUS_CODE_ERROR";
    case Status_StatusCode.UNRECOGNIZED:
    default:
      return "UNRECOGNIZED";
  }
}

function createBaseTracesData(): TracesData {
  return { resourceSpans: [] };
}

export const TracesData = {
  encode(message: TracesData, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.resourceSpans) {
      ResourceSpans.encode(v!, writer.uint32(10).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): TracesData {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseTracesData();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.resourceSpans.push(ResourceSpans.decode(reader, reader.uint32()));
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): TracesData {
    return {
      resourceSpans: Array.isArray(object?.resourceSpans)
        ? object.resourceSpans.map((e: any) => ResourceSpans.fromJSON(e))
        : [],
    };
  },

  toJSON(message: TracesData): unknown {
    const obj: any = {};
    if (message.resourceSpans?.length) {
      obj.resourceSpans = message.resourceSpans.map((e) => ResourceSpans.toJSON(e));
    }
    return obj;
  },

  create<I extends Exact<DeepPartial<TracesData>, I>>(base?: I): TracesData {
    return TracesData.fromPartial(base ?? ({} as any));
  },
  fromPartial<I extends Exact<DeepPartial<TracesData>, I>>(object: I): TracesData {
    const message = createBaseTracesData();
    message.resourceSpans = object.resourceSpans?.map((e) => ResourceSpans.fromPartial(e)) || [];
    return message;
  },
};

function createBaseResourceSpans(): ResourceSpans {
  return { resource: undefined, scopeSpans: [], schemaUrl: "" };
}

export const ResourceSpans = {
  encode(message: ResourceSpans, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.resource !== undefined) {
      Resource.encode(message.resource, writer.uint32(10).fork()).ldelim();
    }
    for (const v of message.scopeSpans) {
      ScopeSpans.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    if (message.schemaUrl !== "") {
      writer.uint32(26).string(message.schemaUrl);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ResourceSpans {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseResourceSpans();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.resource = Resource.decode(reader, reader.uint32());
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.scopeSpans.push(ScopeSpans.decode(reader, reader.uint32()));
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.schemaUrl = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ResourceSpans {
    return {
      resource: isSet(object.resource) ? Resource.fromJSON(object.resource) : undefined,
      scopeSpans: Array.isArray(object?.scopeSpans) ? object.scopeSpans.map((e: any) => ScopeSpans.fromJSON(e)) : [],
      schemaUrl: isSet(object.schemaUrl) ? String(object.schemaUrl) : "",
    };
  },

  toJSON(message: ResourceSpans): unknown {
    const obj: any = {};
    if (message.resource !== undefined) {
      obj.resource = Resource.toJSON(message.resource);
    }
    if (message.scopeSpans?.length) {
      obj.scopeSpans = message.scopeSpans.map((e) => ScopeSpans.toJSON(e));
    }
    if (message.schemaUrl !== "") {
      obj.schemaUrl = message.schemaUrl;
    }
    return obj;
  },

  create<I extends Exact<DeepPartial<ResourceSpans>, I>>(base?: I): ResourceSpans {
    return ResourceSpans.fromPartial(base ?? ({} as any));
  },
  fromPartial<I extends Exact<DeepPartial<ResourceSpans>, I>>(object: I): ResourceSpans {
    const message = createBaseResourceSpans();
    message.resource = (object.resource !== undefined && object.resource !== null)
      ? Resource.fromPartial(object.resource)
      : undefined;
    message.scopeSpans = object.scopeSpans?.map((e) => ScopeSpans.fromPartial(e)) || [];
    message.schemaUrl = object.schemaUrl ?? "";
    return message;
  },
};

function createBaseScopeSpans(): ScopeSpans {
  return { scope: undefined, spans: [], schemaUrl: "" };
}

export const ScopeSpans = {
  encode(message: ScopeSpans, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.scope !== undefined) {
      InstrumentationScope.encode(message.scope, writer.uint32(10).fork()).ldelim();
    }
    for (const v of message.spans) {
      Span.encode(v!, writer.uint32(18).fork()).ldelim();
    }
    if (message.schemaUrl !== "") {
      writer.uint32(26).string(message.schemaUrl);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): ScopeSpans {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseScopeSpans();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.scope = InstrumentationScope.decode(reader, reader.uint32());
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.spans.push(Span.decode(reader, reader.uint32()));
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.schemaUrl = reader.string();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): ScopeSpans {
    return {
      scope: isSet(object.scope) ? InstrumentationScope.fromJSON(object.scope) : undefined,
      spans: Array.isArray(object?.spans) ? object.spans.map((e: any) => Span.fromJSON(e)) : [],
      schemaUrl: isSet(object.schemaUrl) ? String(object.schemaUrl) : "",
    };
  },

  toJSON(message: ScopeSpans): unknown {
    const obj: any = {};
    if (message.scope !== undefined) {
      obj.scope = InstrumentationScope.toJSON(message.scope);
    }
    if (message.spans?.length) {
      obj.spans = message.spans.map((e) => Span.toJSON(e));
    }
    if (message.schemaUrl !== "") {
      obj.schemaUrl = message.schemaUrl;
    }
    return obj;
  },

  create<I extends Exact<DeepPartial<ScopeSpans>, I>>(base?: I): ScopeSpans {
    return ScopeSpans.fromPartial(base ?? ({} as any));
  },
  fromPartial<I extends Exact<DeepPartial<ScopeSpans>, I>>(object: I): ScopeSpans {
    const message = createBaseScopeSpans();
    message.scope = (object.scope !== undefined && object.scope !== null)
      ? InstrumentationScope.fromPartial(object.scope)
      : undefined;
    message.spans = object.spans?.map((e) => Span.fromPartial(e)) || [];
    message.schemaUrl = object.schemaUrl ?? "";
    return message;
  },
};

function createBaseSpan(): Span {
  return {
    traceId: new Uint8Array(0),
    spanId: new Uint8Array(0),
    traceState: "",
    parentSpanId: new Uint8Array(0),
    name: "",
    kind: 0,
    startTimeUnixNano: 0,
    endTimeUnixNano: 0,
    attributes: [],
    droppedAttributesCount: 0,
    events: [],
    droppedEventsCount: 0,
    links: [],
    droppedLinksCount: 0,
    status: undefined,
  };
}

export const Span = {
  encode(message: Span, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.traceId.length !== 0) {
      writer.uint32(10).bytes(message.traceId);
    }
    if (message.spanId.length !== 0) {
      writer.uint32(18).bytes(message.spanId);
    }
    if (message.traceState !== "") {
      writer.uint32(26).string(message.traceState);
    }
    if (message.parentSpanId.length !== 0) {
      writer.uint32(34).bytes(message.parentSpanId);
    }
    if (message.name !== "") {
      writer.uint32(42).string(message.name);
    }
    if (message.kind !== 0) {
      writer.uint32(48).int32(message.kind);
    }
    if (message.startTimeUnixNano !== 0) {
      writer.uint32(57).fixed64(message.startTimeUnixNano);
    }
    if (message.endTimeUnixNano !== 0) {
      writer.uint32(65).fixed64(message.endTimeUnixNano);
    }
    for (const v of message.attributes) {
      KeyValue.encode(v!, writer.uint32(74).fork()).ldelim();
    }
    if (message.droppedAttributesCount !== 0) {
      writer.uint32(80).uint32(message.droppedAttributesCount);
    }
    for (const v of message.events) {
      Span_Event.encode(v!, writer.uint32(90).fork()).ldelim();
    }
    if (message.droppedEventsCount !== 0) {
      writer.uint32(96).uint32(message.droppedEventsCount);
    }
    for (const v of message.links) {
      Span_Link.encode(v!, writer.uint32(106).fork()).ldelim();
    }
    if (message.droppedLinksCount !== 0) {
      writer.uint32(112).uint32(message.droppedLinksCount);
    }
    if (message.status !== undefined) {
      Status.encode(message.status, writer.uint32(122).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Span {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseSpan();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.traceId = reader.bytes();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.spanId = reader.bytes();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.traceState = reader.string();
          continue;
        case 4:
          if (tag !== 34) {
            break;
          }

          message.parentSpanId = reader.bytes();
          continue;
        case 5:
          if (tag !== 42) {
            break;
          }

          message.name = reader.string();
          continue;
        case 6:
          if (tag !== 48) {
            break;
          }

          message.kind = reader.int32() as any;
          continue;
        case 7:
          if (tag !== 57) {
            break;
          }

          message.startTimeUnixNano = longToNumber(reader.fixed64() as Long);
          continue;
        case 8:
          if (tag !== 65) {
            break;
          }

          message.endTimeUnixNano = longToNumber(reader.fixed64() as Long);
          continue;
        case 9:
          if (tag !== 74) {
            break;
          }

          message.attributes.push(KeyValue.decode(reader, reader.uint32()));
          continue;
        case 10:
          if (tag !== 80) {
            break;
          }

          message.droppedAttributesCount = reader.uint32();
          continue;
        case 11:
          if (tag !== 90) {
            break;
          }

          message.events.push(Span_Event.decode(reader, reader.uint32()));
          continue;
        case 12:
          if (tag !== 96) {
            break;
          }

          message.droppedEventsCount = reader.uint32();
          continue;
        case 13:
          if (tag !== 106) {
            break;
          }

          message.links.push(Span_Link.decode(reader, reader.uint32()));
          continue;
        case 14:
          if (tag !== 112) {
            break;
          }

          message.droppedLinksCount = reader.uint32();
          continue;
        case 15:
          if (tag !== 122) {
            break;
          }

          message.status = Status.decode(reader, reader.uint32());
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): Span {
    return {
      traceId: isSet(object.traceId) ? bytesFromBase64(object.traceId) : new Uint8Array(0),
      spanId: isSet(object.spanId) ? bytesFromBase64(object.spanId) : new Uint8Array(0),
      traceState: isSet(object.traceState) ? String(object.traceState) : "",
      parentSpanId: isSet(object.parentSpanId) ? bytesFromBase64(object.parentSpanId) : new Uint8Array(0),
      name: isSet(object.name) ? String(object.name) : "",
      kind: isSet(object.kind) ? span_SpanKindFromJSON(object.kind) : 0,
      startTimeUnixNano: isSet(object.startTimeUnixNano) ? Number(object.startTimeUnixNano) : 0,
      endTimeUnixNano: isSet(object.endTimeUnixNano) ? Number(object.endTimeUnixNano) : 0,
      attributes: Array.isArray(object?.attributes) ? object.attributes.map((e: any) => KeyValue.fromJSON(e)) : [],
      droppedAttributesCount: isSet(object.droppedAttributesCount) ? Number(object.droppedAttributesCount) : 0,
      events: Array.isArray(object?.events) ? object.events.map((e: any) => Span_Event.fromJSON(e)) : [],
      droppedEventsCount: isSet(object.droppedEventsCount) ? Number(object.droppedEventsCount) : 0,
      links: Array.isArray(object?.links) ? object.links.map((e: any) => Span_Link.fromJSON(e)) : [],
      droppedLinksCount: isSet(object.droppedLinksCount) ? Number(object.droppedLinksCount) : 0,
      status: isSet(object.status) ? Status.fromJSON(object.status) : undefined,
    };
  },

  toJSON(message: Span): unknown {
    const obj: any = {};
    if (message.traceId.length !== 0) {
      obj.traceId = base64FromBytes(message.traceId);
    }
    if (message.spanId.length !== 0) {
      obj.spanId = base64FromBytes(message.spanId);
    }
    if (message.traceState !== "") {
      obj.traceState = message.traceState;
    }
    if (message.parentSpanId.length !== 0) {
      obj.parentSpanId = base64FromBytes(message.parentSpanId);
    }
    if (message.name !== "") {
      obj.name = message.name;
    }
    if (message.kind !== 0) {
      obj.kind = span_SpanKindToJSON(message.kind);
    }
    if (message.startTimeUnixNano !== 0) {
      obj.startTimeUnixNano = Math.round(message.startTimeUnixNano);
    }
    if (message.endTimeUnixNano !== 0) {
      obj.endTimeUnixNano = Math.round(message.endTimeUnixNano);
    }
    if (message.attributes?.length) {
      obj.attributes = message.attributes.map((e) => KeyValue.toJSON(e));
    }
    if (message.droppedAttributesCount !== 0) {
      obj.droppedAttributesCount = Math.round(message.droppedAttributesCount);
    }
    if (message.events?.length) {
      obj.events = message.events.map((e) => Span_Event.toJSON(e));
    }
    if (message.droppedEventsCount !== 0) {
      obj.droppedEventsCount = Math.round(message.droppedEventsCount);
    }
    if (message.links?.length) {
      obj.links = message.links.map((e) => Span_Link.toJSON(e));
    }
    if (message.droppedLinksCount !== 0) {
      obj.droppedLinksCount = Math.round(message.droppedLinksCount);
    }
    if (message.status !== undefined) {
      obj.status = Status.toJSON(message.status);
    }
    return obj;
  },

  create<I extends Exact<DeepPartial<Span>, I>>(base?: I): Span {
    return Span.fromPartial(base ?? ({} as any));
  },
  fromPartial<I extends Exact<DeepPartial<Span>, I>>(object: I): Span {
    const message = createBaseSpan();
    message.traceId = object.traceId ?? new Uint8Array(0);
    message.spanId = object.spanId ?? new Uint8Array(0);
    message.traceState = object.traceState ?? "";
    message.parentSpanId = object.parentSpanId ?? new Uint8Array(0);
    message.name = object.name ?? "";
    message.kind = object.kind ?? 0;
    message.startTimeUnixNano = object.startTimeUnixNano ?? 0;
    message.endTimeUnixNano = object.endTimeUnixNano ?? 0;
    message.attributes = object.attributes?.map((e) => KeyValue.fromPartial(e)) || [];
    message.droppedAttributesCount = object.droppedAttributesCount ?? 0;
    message.events = object.events?.map((e) => Span_Event.fromPartial(e)) || [];
    message.droppedEventsCount = object.droppedEventsCount ?? 0;
    message.links = object.links?.map((e) => Span_Link.fromPartial(e)) || [];
    message.droppedLinksCount = object.droppedLinksCount ?? 0;
    message.status = (object.status !== undefined && object.status !== null)
      ? Status.fromPartial(object.status)
      : undefined;
    return message;
  },
};

function createBaseSpan_Event(): Span_Event {
  return { timeUnixNano: 0, name: "", attributes: [], droppedAttributesCount: 0 };
}

export const Span_Event = {
  encode(message: Span_Event, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.timeUnixNano !== 0) {
      writer.uint32(9).fixed64(message.timeUnixNano);
    }
    if (message.name !== "") {
      writer.uint32(18).string(message.name);
    }
    for (const v of message.attributes) {
      KeyValue.encode(v!, writer.uint32(26).fork()).ldelim();
    }
    if (message.droppedAttributesCount !== 0) {
      writer.uint32(32).uint32(message.droppedAttributesCount);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Span_Event {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseSpan_Event();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 9) {
            break;
          }

          message.timeUnixNano = longToNumber(reader.fixed64() as Long);
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.name = reader.string();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.attributes.push(KeyValue.decode(reader, reader.uint32()));
          continue;
        case 4:
          if (tag !== 32) {
            break;
          }

          message.droppedAttributesCount = reader.uint32();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): Span_Event {
    return {
      timeUnixNano: isSet(object.timeUnixNano) ? Number(object.timeUnixNano) : 0,
      name: isSet(object.name) ? String(object.name) : "",
      attributes: Array.isArray(object?.attributes) ? object.attributes.map((e: any) => KeyValue.fromJSON(e)) : [],
      droppedAttributesCount: isSet(object.droppedAttributesCount) ? Number(object.droppedAttributesCount) : 0,
    };
  },

  toJSON(message: Span_Event): unknown {
    const obj: any = {};
    if (message.timeUnixNano !== 0) {
      obj.timeUnixNano = Math.round(message.timeUnixNano);
    }
    if (message.name !== "") {
      obj.name = message.name;
    }
    if (message.attributes?.length) {
      obj.attributes = message.attributes.map((e) => KeyValue.toJSON(e));
    }
    if (message.droppedAttributesCount !== 0) {
      obj.droppedAttributesCount = Math.round(message.droppedAttributesCount);
    }
    return obj;
  },

  create<I extends Exact<DeepPartial<Span_Event>, I>>(base?: I): Span_Event {
    return Span_Event.fromPartial(base ?? ({} as any));
  },
  fromPartial<I extends Exact<DeepPartial<Span_Event>, I>>(object: I): Span_Event {
    const message = createBaseSpan_Event();
    message.timeUnixNano = object.timeUnixNano ?? 0;
    message.name = object.name ?? "";
    message.attributes = object.attributes?.map((e) => KeyValue.fromPartial(e)) || [];
    message.droppedAttributesCount = object.droppedAttributesCount ?? 0;
    return message;
  },
};

function createBaseSpan_Link(): Span_Link {
  return {
    traceId: new Uint8Array(0),
    spanId: new Uint8Array(0),
    traceState: "",
    attributes: [],
    droppedAttributesCount: 0,
  };
}

export const Span_Link = {
  encode(message: Span_Link, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.traceId.length !== 0) {
      writer.uint32(10).bytes(message.traceId);
    }
    if (message.spanId.length !== 0) {
      writer.uint32(18).bytes(message.spanId);
    }
    if (message.traceState !== "") {
      writer.uint32(26).string(message.traceState);
    }
    for (const v of message.attributes) {
      KeyValue.encode(v!, writer.uint32(34).fork()).ldelim();
    }
    if (message.droppedAttributesCount !== 0) {
      writer.uint32(40).uint32(message.droppedAttributesCount);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Span_Link {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseSpan_Link();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          if (tag !== 10) {
            break;
          }

          message.traceId = reader.bytes();
          continue;
        case 2:
          if (tag !== 18) {
            break;
          }

          message.spanId = reader.bytes();
          continue;
        case 3:
          if (tag !== 26) {
            break;
          }

          message.traceState = reader.string();
          continue;
        case 4:
          if (tag !== 34) {
            break;
          }

          message.attributes.push(KeyValue.decode(reader, reader.uint32()));
          continue;
        case 5:
          if (tag !== 40) {
            break;
          }

          message.droppedAttributesCount = reader.uint32();
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): Span_Link {
    return {
      traceId: isSet(object.traceId) ? bytesFromBase64(object.traceId) : new Uint8Array(0),
      spanId: isSet(object.spanId) ? bytesFromBase64(object.spanId) : new Uint8Array(0),
      traceState: isSet(object.traceState) ? String(object.traceState) : "",
      attributes: Array.isArray(object?.attributes) ? object.attributes.map((e: any) => KeyValue.fromJSON(e)) : [],
      droppedAttributesCount: isSet(object.droppedAttributesCount) ? Number(object.droppedAttributesCount) : 0,
    };
  },

  toJSON(message: Span_Link): unknown {
    const obj: any = {};
    if (message.traceId.length !== 0) {
      obj.traceId = base64FromBytes(message.traceId);
    }
    if (message.spanId.length !== 0) {
      obj.spanId = base64FromBytes(message.spanId);
    }
    if (message.traceState !== "") {
      obj.traceState = message.traceState;
    }
    if (message.attributes?.length) {
      obj.attributes = message.attributes.map((e) => KeyValue.toJSON(e));
    }
    if (message.droppedAttributesCount !== 0) {
      obj.droppedAttributesCount = Math.round(message.droppedAttributesCount);
    }
    return obj;
  },

  create<I extends Exact<DeepPartial<Span_Link>, I>>(base?: I): Span_Link {
    return Span_Link.fromPartial(base ?? ({} as any));
  },
  fromPartial<I extends Exact<DeepPartial<Span_Link>, I>>(object: I): Span_Link {
    const message = createBaseSpan_Link();
    message.traceId = object.traceId ?? new Uint8Array(0);
    message.spanId = object.spanId ?? new Uint8Array(0);
    message.traceState = object.traceState ?? "";
    message.attributes = object.attributes?.map((e) => KeyValue.fromPartial(e)) || [];
    message.droppedAttributesCount = object.droppedAttributesCount ?? 0;
    return message;
  },
};

function createBaseStatus(): Status {
  return { message: "", code: 0 };
}

export const Status = {
  encode(message: Status, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.message !== "") {
      writer.uint32(18).string(message.message);
    }
    if (message.code !== 0) {
      writer.uint32(24).int32(message.code);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Status {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseStatus();
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 2:
          if (tag !== 18) {
            break;
          }

          message.message = reader.string();
          continue;
        case 3:
          if (tag !== 24) {
            break;
          }

          message.code = reader.int32() as any;
          continue;
      }
      if ((tag & 7) === 4 || tag === 0) {
        break;
      }
      reader.skipType(tag & 7);
    }
    return message;
  },

  fromJSON(object: any): Status {
    return {
      message: isSet(object.message) ? String(object.message) : "",
      code: isSet(object.code) ? status_StatusCodeFromJSON(object.code) : 0,
    };
  },

  toJSON(message: Status): unknown {
    const obj: any = {};
    if (message.message !== "") {
      obj.message = message.message;
    }
    if (message.code !== 0) {
      obj.code = status_StatusCodeToJSON(message.code);
    }
    return obj;
  },

  create<I extends Exact<DeepPartial<Status>, I>>(base?: I): Status {
    return Status.fromPartial(base ?? ({} as any));
  },
  fromPartial<I extends Exact<DeepPartial<Status>, I>>(object: I): Status {
    const message = createBaseStatus();
    message.message = object.message ?? "";
    message.code = object.code ?? 0;
    return message;
  },
};

declare const self: any | undefined;
declare const window: any | undefined;
declare const global: any | undefined;
const tsProtoGlobalThis: any = (() => {
  if (typeof globalThis !== "undefined") {
    return globalThis;
  }
  if (typeof self !== "undefined") {
    return self;
  }
  if (typeof window !== "undefined") {
    return window;
  }
  if (typeof global !== "undefined") {
    return global;
  }
  throw "Unable to locate global object";
})();

function bytesFromBase64(b64: string): Uint8Array {
  if (tsProtoGlobalThis.Buffer) {
    return Uint8Array.from(tsProtoGlobalThis.Buffer.from(b64, "base64"));
  } else {
    const bin = tsProtoGlobalThis.atob(b64);
    const arr = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; ++i) {
      arr[i] = bin.charCodeAt(i);
    }
    return arr;
  }
}

function base64FromBytes(arr: Uint8Array): string {
  if (tsProtoGlobalThis.Buffer) {
    return tsProtoGlobalThis.Buffer.from(arr).toString("base64");
  } else {
    const bin: string[] = [];
    arr.forEach((byte) => {
      bin.push(String.fromCharCode(byte));
    });
    return tsProtoGlobalThis.btoa(bin.join(""));
  }
}

type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined;

export type DeepPartial<T> = T extends Builtin ? T
  : T extends Array<infer U> ? Array<DeepPartial<U>> : T extends ReadonlyArray<infer U> ? ReadonlyArray<DeepPartial<U>>
  : T extends {} ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>;

type KeysOfUnion<T> = T extends T ? keyof T : never;
export type Exact<P, I extends P> = P extends Builtin ? P
  : P & { [K in keyof P]: Exact<P[K], I[K]> } & { [K in Exclude<keyof I, KeysOfUnion<P>>]: never };

function longToNumber(long: Long): number {
  if (long.gt(Number.MAX_SAFE_INTEGER)) {
    throw new tsProtoGlobalThis.Error("Value is larger than Number.MAX_SAFE_INTEGER");
  }
  return long.toNumber();
}

if (_m0.util.Long !== Long) {
  _m0.util.Long = Long as any;
  _m0.configure();
}

function isSet(value: any): boolean {
  return value !== null && value !== undefined;
}
