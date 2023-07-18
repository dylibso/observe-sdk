package datadog_formatter

import (
	"fmt"
	"time"

	"github.com/dylibso/observe-sdk/go"
)

type DatadogFormatter []Trace

type Trace []Span

type Span struct {
	TraceId  observe.TelemetryId   `json:"trace_id"`
	SpanId   observe.TelemetryId   `json:"span_id"`
	ParentId *observe.TelemetryId  `json:"parent_id,omitempty"`
	Name     string                `json:"name"`
	Start    uint64                `json:"start"`
	Duration uint64                `json:"duration"`
	Resource string                `json:"resource"`
	Error    uint8                 `json:"error"`
	Meta     map[string]string     `json:"meta"`
	Metrics  map[string]string     `json:"metrics"`
	Service  string                `json:"service"`
	Type     *string               `json:"type,omitempty"`
}

func NewSpan(service string, traceId observe.TelemetryId, parentId *observe.TelemetryId, name string, start, end time.Time) *Span {
	id := observe.NewSpanId()
	span := Span{
		SpanId:   id,
		ParentId: parentId,
		TraceId:  traceId,
		Name:     name,
		Start:    uint64(start.UnixNano()),
		Duration: uint64(end.Sub(start).Nanoseconds()),
		Service:  service,
		Resource: name,
	}
	return &span
}

func (s *Span) AddAllocation(amount uint32) {
	if s.Meta == nil {
		s.Meta = make(map[string]string)
	}

	s.Meta["allocation"] = fmt.Sprintf("%d", amount)
}

func New() *DatadogFormatter {
	return &DatadogFormatter{}
}

func (d *DatadogFormatter) AddTrace(trace Trace) {
	*d = append(*d, trace)
}

type DatadogTraceType int

const (
	Web DatadogTraceType = iota
	Db
	Cache
	Custom
)

func (d DatadogTraceType) String() string {
	switch d {
	case Web:
		return "web"
	case Db:
		return "db"
	case Cache:
		return "cache"
	case Custom:
		return "custom"
	default:
		return "unknown-trace-type"
	}
}
