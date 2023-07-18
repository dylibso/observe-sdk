package datadog

import (
	"bytes"
	"encoding/json"
	"log"
	"net/http"
	"net/url"

	"github.com/dylibso/observe-sdk/go"
	"github.com/dylibso/observe-sdk/go/adapter/datadog_formatter"
)

type DatadogConfig struct {
	AgentHost   string                             `json:"agent_host"`
	ServiceName string                             `json:"service_name"`
	DefaultTags map[string]string                  `json:"default_tags"`
	TraceType   datadog_formatter.DatadogTraceType `json:"trace_type"`
}

func DefaultDatadogConfig() *DatadogConfig {
	return &DatadogConfig{
		AgentHost:   "http://localhost:8126",
		ServiceName: "my-wasm-service",
		DefaultTags: nil,
		TraceType:   datadog_formatter.Web,
	}
}

type DatadogAdapter struct {
	observe.AdapterBase
	Config *DatadogConfig
}

func NewDatadogAdapter(config *DatadogConfig) (DatadogAdapter, error) {
	if config == nil {
		config = DefaultDatadogConfig()
	}

	return DatadogAdapter{
		AdapterBase: observe.NewAdapterBase(),
		Config:      config,
	}, nil
}

func (d *DatadogAdapter) HandleTraceEvent(te observe.TraceEvent) {
	if te.TelemetryId == nil {
		log.Println("Datadog adapter needs a trace id")
		return
	}

	var allSpans []datadog_formatter.Span
	for _, e := range te.Events {
		switch event := e.(type) {
		case observe.CallEvent:
			spans := d.makeCallSpans(event, nil, *te.TelemetryId)
			if len(spans) > 0 {
				allSpans = append(allSpans, spans...)
			}
		case observe.MemoryGrowEvent:
			if len(allSpans) > 0 {
				allSpans[len(allSpans)-1].AddAllocation(event.MemoryGrowAmount())
			}
		case observe.CustomEvent:
			log.Println("Datadog adapter does not respect custom events")
		}
	}

	if len(allSpans) <= 1 {
		log.Println("No spans built for datadog trace")
		return
	}

	go func() {
		output := datadog_formatter.New()
		// TODO: for the moment, these are hard-coded, but will transition to a programmer-
		// controlled API to customer these values.
		allSpans[0].Resource = "request"
		tt := d.Config.TraceType.String()
		allSpans[0].Type = &tt
		output.AddTrace(allSpans)

		b, err := json.Marshal(output)
		if err != nil {
			log.Println("failed to encode trace data to json", err)
			return
		}

		data := bytes.NewBuffer(b)

		host, err := url.JoinPath(d.Config.AgentHost, "v0.3", "traces")
		if err != nil {
			log.Println("failed to create datadog agent endpoint url:", err)
			return
		}

		resp, err := http.Post(host, "application/json", data)
		if err != nil {
			log.Println("failed to send trace request to datadog:", err)
			return
		}

		if resp.StatusCode != http.StatusOK {
			log.Println("unexpected status code from datadog agent:", resp.StatusCode)
		}
	}()
}

func (d *DatadogAdapter) Start() {
	d.AdapterBase.Start(d)
}

func (d *DatadogAdapter) Stop() {
	d.AdapterBase.Stop()
}

func (d *DatadogAdapter) makeCallSpans(event observe.CallEvent, parentId *observe.TelemetryId, traceId observe.TelemetryId) []datadog_formatter.Span {
	name := event.FunctionName()
	span := datadog_formatter.NewSpan(d.Config.ServiceName, traceId, parentId, name, event.Time, event.Time.Add(event.Duration))

	spans := []datadog_formatter.Span{*span}
	for _, ev := range event.Within() {
		if call, ok := ev.(observe.CallEvent); ok {
			spans = append(spans, d.makeCallSpans(call, &span.SpanId, traceId)...)
		}
	}

	return spans
}

type DatadogSpanKind int

const (
	Server DatadogSpanKind = iota
	Client
	Producer
	Consumer
	Internal
)

func (d DatadogSpanKind) String() string {
	switch d {
	case Server:
		return "server"
	case Client:
		return "client"
	case Producer:
		return "producer"
	case Consumer:
		return "consumer"
	case Internal:
		return "internal"
	default:
		return "unknown-span-kind"
	}
}
