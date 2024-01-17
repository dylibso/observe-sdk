package otel_stdout

import (
	"context"
	"encoding/json"
	"fmt"
	"log"

	observe "github.com/dylibso/observe-sdk/go"
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
				spans := o.MakeOtelCallSpans(event, nil, traceId)
				if len(spans) > 0 {
					allSpans = append(allSpans, spans...)
				}
			case observe.MemoryGrowEvent:
				log.Println("MemoryGrowEvent should be attached to a span")
			case observe.CustomEvent:
				log.Println("Otel adapter does not respect custom events")
			case observe.MetricEvent:
				log.Printf("metric: %s\n", event.Message)
			case observe.LogEvent:
				log.Println(event.Message)
			}
		}

		if len(allSpans) == 0 {
			return nil
		}

		t := observe.NewOtelTrace(traceId, "golang", allSpans)
		b, err := json.Marshal(t.TracesData)
		if err != nil {
			log.Println("failed to encode CallEvent spans")
			return nil
		}

		fmt.Println(string(b))
	}

	return nil
}

func (o *OtelStdoutAdapter) Start(ctx context.Context) {
	o.AdapterBase.Start(ctx, o)
}
