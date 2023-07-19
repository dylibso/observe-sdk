package otel_stdout

import (
	"encoding/json"
	"fmt"
	"log"

	observe "github.com/dylibso/observe-sdk/go"
	"github.com/dylibso/observe-sdk/go/adapter/otel_formatter"
	otel "github.com/dylibso/observe-sdk/go/adapter/otel_formatter"
)

type OtelStdoutAdapter struct {
	observe.AdapterBase
}

func NewOtelStdoutAdapter() OtelStdoutAdapter {
	base := observe.NewAdapterBase()
	return OtelStdoutAdapter{
		AdapterBase: base,
	}
}

func (o *OtelStdoutAdapter) HandleTraceEvent(te observe.TraceEvent) {
	traceId := te.TelemetryId.ToHex16()

	var allSpans []otel_formatter.Span
	for _, e := range te.Events {
		switch event := e.(type) {
		case observe.CallEvent:
			spans := o.makeCallSpans(event, nil, traceId)
			if len(spans) > 0 {
				allSpans = append(allSpans, spans...)
			}
		case observe.MemoryGrowEvent:
			span := otel.NewSpan(traceId, nil, "allocation", event.Time, event.Time)
			span.AddAttribute("amount", event.MemoryGrowAmount())
			allSpans = append(allSpans, *span)
		case observe.CustomEvent:
			log.Println("Otel adapter does not respect custom events")
		}
	}

	if len(allSpans) <= 1 {
		log.Println("No spans built for datadog trace")
		return
	}

	output := otel.New()
	resourceSpan := otel.NewResourceSpan()
	resourceSpan.AddSpans(allSpans)
	output.AddResourceSpan(*resourceSpan)
	b, err := json.Marshal(output)
	if err != nil {
		log.Println("failed to encode CallEvent spans")
		return
	}

	fmt.Println(string(b))

}

func (o *OtelStdoutAdapter) Start() {
	o.AdapterBase.Start(o)
}

func (o *OtelStdoutAdapter) Stop() {
	o.AdapterBase.Stop()
}

func (o *OtelStdoutAdapter) makeCallSpans(event observe.CallEvent, parentId *string, traceId string) []otel.Span {
	name := event.FunctionName()
	span := otel.NewSpan(traceId, parentId, name, event.Time, event.Time.Add(event.Duration))
	span.AddAttribute("function_name", fmt.Sprintf("function-call-%s", name))

	spans := []otel.Span{*span}
	for _, ev := range event.Within() {
		if call, ok := ev.(observe.CallEvent); ok {
			spans = append(spans, o.makeCallSpans(call, &span.SpanId, traceId)...)
		}
	}

	return spans
}
