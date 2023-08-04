package otel_stdout

import (
	"encoding/json"
	"fmt"
	"log"

	observe "github.com/dylibso/observe-sdk/go"
	otel "github.com/dylibso/observe-sdk/go/adapter/otel_formatter"
	trace "go.opentelemetry.io/proto/otlp/trace/v1"
)

type OtelStdoutAdapter struct {
	*observe.AdapterBase
}

func NewOtelStdoutAdapter() *OtelStdoutAdapter {
	base := observe.NewAdapterBase(1, 0)
	adapter := &OtelStdoutAdapter{
		AdapterBase: &base,
	}

	adapter.AdapterBase.SetFlusher(adapter)

	return adapter
}

func (o *OtelStdoutAdapter) HandleTraceEvent(te observe.TraceEvent) {
	o.AdapterBase.HandleTraceEvent(te)
}

func (o *OtelStdoutAdapter) Flush(evts []observe.TraceEvent) error {
	for _, te := range evts {
		traceId := te.TelemetryId.ToHex16()

		var allSpans []*trace.Span
		for _, e := range te.Events {
			switch event := e.(type) {
			case observe.CallEvent:
				spans := o.makeCallSpans(event, nil, traceId)
				if len(spans) > 0 {
					allSpans = append(allSpans, spans...)
				}
			case observe.MemoryGrowEvent:
				log.Println("MemoryGrowEvent should be attached to a span")
			case observe.CustomEvent:
				log.Println("Otel adapter does not respect custom events")
			}
		}

		if len(allSpans) == 0 {
			log.Println("No spans built for datadog trace")
			return nil
		}

		t := otel.NewTrace(traceId, "golang", allSpans)
		b, err := json.Marshal(t.TracesData)
		if err != nil {
			log.Println("failed to encode CallEvent spans")
			return nil
		}

		fmt.Println(string(b))
	}

	return nil
}

func (o *OtelStdoutAdapter) makeCallSpans(event observe.CallEvent, parentId []byte, traceId string) []*trace.Span {
	name := event.FunctionName()
	span := otel.NewSpan(traceId, parentId, name, event.Time, event.Time.Add(event.Duration))
	span.Attributes = append(span.Attributes, otel.NewKeyValueString("function_name", fmt.Sprintf("function-call-%s", name)))

	spans := []*trace.Span{span}
	for _, ev := range event.Within() {
		if call, ok := ev.(observe.CallEvent); ok {
			spans = append(spans, o.makeCallSpans(call, span.SpanId, traceId)...)
		}
		if alloc, ok := ev.(observe.MemoryGrowEvent); ok {
			last := spans[len(spans)-1]
			last.Attributes = append(last.Attributes, otel.NewKeyValueInt64("allocation", int64(alloc.MemoryGrowAmount())))
		}
	}

	return spans
}

func (o *OtelStdoutAdapter) Start() {
	o.AdapterBase.Start(o)
}
