package datadog

import (
	"bytes"
	"encoding/json"
	"log"
	"net/http"
	"net/url"
	"strconv"
	"time"

	observe "github.com/dylibso/observe-sdk-wazero"
	"github.com/dylibso/observe-sdk-wazero/adapter/datadog_formatter"
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
	TraceId uint64
	Spans   []datadog_formatter.Span
	Config  *DatadogConfig
}

func NewDatadogAdapter(config *DatadogConfig, wasm []byte) (DatadogAdapter, error) {
	if config == nil {
		config = DefaultDatadogConfig()
	}

	return DatadogAdapter{
		AdapterBase: observe.NewAdapterBase(),
		TraceId:     uint64(observe.NewTraceId()),
		Config:      config,
	}, nil
}

func (d *DatadogAdapter) Event(e observe.Event) {
	switch event := e.(type) {
	case observe.CallEvent:
		spans := d.makeCallSpans(event, nil)
		if len(spans) > 0 {
			d.Spans = append(d.Spans, spans...)
		}

	case observe.MemoryGrowEvent:
		if len(d.Spans) > 0 {
			d.Spans[len(d.Spans)-1].AddAllocation(event.MemoryGrowAmount())
		}
	case observe.CustomEvent:
		if value, ok := event.Metadata["trace_id"]; ok {
			traceId, err := strconv.ParseUint(value.(string), 10, 64)
			if err != nil {
				log.Println("failed to parse traceId from event metadata")
				return
			}

			d.TraceId = traceId
		}
	}
}

func (d *DatadogAdapter) Wait(collector *observe.Collector, timeout time.Duration) {
	d.AdapterBase.Wait(collector, timeout, nil)
}

func (d *DatadogAdapter) Start(collector *observe.Collector) {
	go func() {
		for {
			select {
			case event := <-collector.Events:
				d.Event(event)
			case <-d.StopChan(collector):
				return
			}
		}
	}()
}

func (d *DatadogAdapter) Stop(collector *observe.Collector) {
	d.AdapterBase.Stop(collector)

	if len(d.Spans) == 0 {
		return
	}

	go func() {
		output := datadog_formatter.New()
		// TODO: for the moment, these are hard-coded, but will transition to a programmer-
		// controlled API to customer these values.
		d.Spans[0].Resource = "request"
		tt := d.Config.TraceType.String()
		d.Spans[0].Type = &tt
		output.AddTrace(d.Spans)

		b, err := json.Marshal(output)
		if err != nil {
			log.Println("failed to encode trace data to json")
			return
		}

		data := bytes.NewBuffer(b)

		host, err := url.JoinPath(d.Config.AgentHost, "v0.3", "traces")
		if err != nil {
			log.Println("failed to create datadog agent endpoint url")
			return
		}

		resp, err := http.Post(host, "application/json", data)
		if err != nil {
			log.Println("failed to send trace request to datadog")
			return
		}

		if resp.StatusCode != http.StatusOK {
			log.Println("unexpected status code from datadog agent:", resp.StatusCode)
		}
	}()
}

func (d *DatadogAdapter) makeCallSpans(event observe.CallEvent, parentId *uint64) []datadog_formatter.Span {
	name := event.FunctionName()
	span := datadog_formatter.NewSpan(d.Config.ServiceName, d.TraceId, parentId, name, event.Time, event.Time.Add(event.Duration))

	spans := []datadog_formatter.Span{*span}
	for _, ev := range event.Within() {
		if call, ok := ev.(observe.CallEvent); ok {
			spans = append(spans, d.makeCallSpans(call, &span.SpanId)...)
		}
	}

	return spans
}

func NewTraceId() uint64 {
	return uint64(observe.NewTraceId())
}

func (d *DatadogAdapter) SetTraceId(traceId uint64) {
	d.TraceId = traceId
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
