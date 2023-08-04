package otel_formatter

import (
	"time"

	common "go.opentelemetry.io/proto/otlp/common/v1"
	resource "go.opentelemetry.io/proto/otlp/resource/v1"
	trace "go.opentelemetry.io/proto/otlp/trace/v1"

	observe "github.com/dylibso/observe-sdk/go"
)

type Trace struct {
	TraceId    string
	TracesData *trace.TracesData
}

func NewTrace(traceId string, serviceName string, spans []*trace.Span) *Trace {
	return &Trace{
		TraceId: traceId,
		TracesData: &trace.TracesData{
			ResourceSpans: []*trace.ResourceSpans{
				{
					Resource: &resource.Resource{
						Attributes: []*common.KeyValue{
							NewKeyValueString("service.name", serviceName),
						},
					},
					ScopeSpans: []*trace.ScopeSpans{
						{
							Spans: spans,
						},
					},
				},
			},
		},
	}
}

func NewSpan(traceId string, parentId []byte, name string, start, end time.Time) *trace.Span {
	if parentId == nil {
		parentId = []byte{}
	}
	return &trace.Span{
		TraceId:           []byte(traceId),
		SpanId:            []byte(observe.NewSpanId().ToHex8()),
		ParentSpanId:      parentId,
		Name:              name,
		Kind:              1,
		StartTimeUnixNano: uint64(start.UnixNano()),
		EndTimeUnixNano:   uint64(end.UnixNano()),
		// uses empty defaults for remaining fields...
	}
}

func NewKeyValueString(key string, value string) *common.KeyValue {
	strVal := &common.AnyValue_StringValue{
		StringValue: value,
	}
	return &common.KeyValue{
		Key: key,
		Value: &common.AnyValue{
			Value: strVal,
		},
	}
}

func NewKeyValueInt64(key string, value int64) *common.KeyValue {
	strVal := &common.AnyValue_IntValue{
		IntValue: value,
	}
	return &common.KeyValue{
		Key: key,
		Value: &common.AnyValue{
			Value: strVal,
		},
	}
}
