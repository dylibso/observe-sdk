package otel_formatter

import (
	"time"

	"github.com/dylibso/observe-sdk-wazero"
)

type OtelFormatter struct {
	ResourceSpans []ResourceSpan `json:"resourceSpans"`
}

func New() *OtelFormatter {
	return &OtelFormatter{}
}

func (o *OtelFormatter) AddResourceSpan(span ResourceSpan) {
	o.ResourceSpans = append(o.ResourceSpans, span)
}

type ResourceSpan struct {
	Resource   Resource    `json:"resource"`
	ScopeSpans []ScopeSpan `json:"scopeSpans"`
}

func NewResourceSpan() *ResourceSpan {
	return &ResourceSpan{}
}

func (r *ResourceSpan) AddAttribute(key string, value any) *ResourceSpan {
	r.Resource.Attributes = append(r.Resource.Attributes, Attribute{Key: key, Value: value})
	return r
}

func (r *ResourceSpan) AddSpans(spans []Span) {
	r.ScopeSpans = append(r.ScopeSpans, ScopeSpan{
		Scope: Scope{
			Name: "event",
		},
		Spans: spans,
	})
}

type Resource struct {
	Attributes []Attribute `json:"attributes"`
}

type ScopeSpan struct {
	Scope Scope  `json:"scope"`
	Spans []Span `json:"spans"`
}

type Attribute struct {
	Key   string `json:"key"`
	Value any    `json:"value"`
}

type Scope struct {
	Name string `json:"name"`
}

type Span struct {
	TraceId                string      `json:"traceId"`
	SpanId                 string      `json:"spanId"`
	ParentSpanId           string      `json:"parentSpanId"`
	Name                   string      `json:"name"`
	Kind                   int64       `json:"kind"`
	StartTimeNano          int64       `json:"startTimeUnixNano"`
	EndTimeNano            int64       `json:"endTimeUnixNano"`
	Attributes             []Attribute `json:"attributes"`
	DroppedAttributesCount int64       `json:"droppedAttributesCount"`
	DroppedEventsCount     int64       `json:"droppedEventsCount"`
	DroppedLinksCount      int64       `json:"droppedLinksCount"`
	Status                 Status      `json:"status"`
}

type Status struct{}

func NewSpan(traceId string, parentId *string, name string, start, end time.Time) *Span {
	if parentId == nil {
		var empty string
		parentId = &empty
	}
	return &Span{
		TraceId:       traceId,
		SpanId:        dylibso_observe.NewSpanId().ToHex8(),
		ParentSpanId:  *parentId,
		Name:          name,
		Kind:          1,
		StartTimeNano: start.UnixNano(),
		EndTimeNano:   end.UnixNano(),
		// uses empty defaults for remaining fields...
	}
}

func (s *Span) AddAttribute(key string, value any) {
	s.Attributes = append(s.Attributes, Attribute{Key: key, Value: value})
}
