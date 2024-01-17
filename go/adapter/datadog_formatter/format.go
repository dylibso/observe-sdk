package datadog_formatter

import (
	"fmt"
	"strconv"
	"time"

	observe "github.com/dylibso/observe-sdk/go"
)

type DatadogFormatter []Trace

type Trace []*Span

type Span struct {
	TraceId  uint64            `json:"trace_id"`
	SpanId   uint64            `json:"span_id"`
	ParentId *uint64           `json:"parent_id,omitempty"`
	Name     string            `json:"name"`
	Start    uint64            `json:"start"`
	Duration uint64            `json:"duration"`
	Resource string            `json:"resource"`
	Error    uint8             `json:"error"`
	Meta     map[string]string `json:"meta"`
	Metrics  map[string]string `json:"metrics"`
	Service  string            `json:"service"`
	Type     *string           `json:"type,omitempty"`
}

func NewSpan(service string, traceId uint64, parentId *uint64, name string, start, end time.Time) *Span {
	id := observe.NewSpanId()
	span := Span{
		SpanId:   id.ToUint64(),
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

	existingAmount, err := strconv.Atoi(s.Meta["allocation"])
	if err == nil && existingAmount > 0 {
		s.Meta["allocation"] = fmt.Sprintf("%d", amount+uint32(existingAmount))
	} else {
		s.Meta["allocation"] = fmt.Sprintf("%d", amount)
	}
}

func (s *Span) AddTag(key, value string) {
	s.Meta[key] = value
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
