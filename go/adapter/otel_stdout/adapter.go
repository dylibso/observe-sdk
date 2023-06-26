package otel_stdout

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	observe "github.com/dylibso/observe-sdk/go"
	otel "github.com/dylibso/observe-sdk/go/adapter/otel_formatter"
)

type OtelStdoutAdapter struct {
	observe.AdapterBase
	TraceId string
}

func NewOtelStdoutAdapter() OtelStdoutAdapter {
	base := observe.NewAdapterBase()
	return OtelStdoutAdapter{
		AdapterBase: base,
		TraceId:     observe.NewTraceId().ToHex16(),
	}
}

func (o *OtelStdoutAdapter) Event(e observe.Event) {
	switch event := e.(type) {
	case observe.CallEvent:
		spans := o.makeCallSpans(event, nil)
		if len(spans) > 0 {
			output := otel.New()
			resourceSpan := otel.NewResourceSpan()
			resourceSpan.AddSpans(spans)
			output.AddResourceSpan(*resourceSpan)
			b, err := json.Marshal(output)
			if err != nil {
				log.Println("failed to encode CallEvent spans")
			}

			fmt.Println(string(b))
		}

	case observe.MemoryGrowEvent:
		output := otel.New()
		span := otel.NewSpan(o.TraceId, nil, "allocation", event.Time, event.Time)
		span.AddAttribute("amount", event.MemoryGrowAmount())
		resourceSpan := otel.NewResourceSpan()
		resourceSpan.AddSpans([]otel.Span{*span})
		output.AddResourceSpan(*resourceSpan)
		b, err := json.Marshal(output)
		if err != nil {
			log.Println("failed to encode MemoryGrowEvent spans")
		}

		fmt.Println(string(b))

	case observe.CustomEvent:
		if value, ok := event.Metadata["trace_id"]; ok {
			o.TraceId = value.(string)
		}
	}
}

func (o *OtelStdoutAdapter) Wait(collector *observe.Collector, timeout time.Duration) {
	o.AdapterBase.Wait(collector, timeout, func() {})
}

func (o *OtelStdoutAdapter) Start(collector *observe.Collector, wasm []byte) error {
	if err := o.AdapterBase.Start(collector, wasm); err != nil {
		return err
	}

	stop := o.StopChan(collector)

	go func() {
		for {
			select {
			case event := <-collector.Events:
				o.Event(event)
			case <-stop:
				return
			}
		}
	}()
	return nil
}

func (o *OtelStdoutAdapter) makeCallSpans(event observe.CallEvent, parentId *string) []otel.Span {
	name := event.FunctionName()
	span := otel.NewSpan(o.TraceId, parentId, name, event.Time, event.Time.Add(event.Duration))
	span.AddAttribute("function_name", fmt.Sprintf("function-call-%s", name))

	spans := []otel.Span{*span}
	for _, ev := range event.Within() {
		if call, ok := ev.(observe.CallEvent); ok {
			spans = append(spans, o.makeCallSpans(call, &span.SpanId)...)
		}
	}

	return spans
}
