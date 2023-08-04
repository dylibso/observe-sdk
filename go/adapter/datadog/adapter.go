package datadog

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"net/url"
	"time"

	observe "github.com/dylibso/observe-sdk/go"
	"github.com/dylibso/observe-sdk/go/adapter/datadog_formatter"
)

type DatadogConfig struct {
	AgentHost   string                             `json:"agent_host"`
	ServiceName string                             `json:"service_name"`
	DefaultTags map[string]string                  `json:"default_tags"`
	TraceType   datadog_formatter.DatadogTraceType `json:"trace_type"`
	FlushPeriod time.Duration
	BatchSize   int
}

func DefaultDatadogConfig() *DatadogConfig {
	return &DatadogConfig{
		AgentHost:   "http://localhost:8126",
		ServiceName: "my-wasm-service",
		DefaultTags: nil,
		TraceType:   datadog_formatter.Web,
		FlushPeriod: 2 * time.Second,
		BatchSize:   100,
	}
}

type DatadogAdapter struct {
	*observe.AdapterBase
	Config *DatadogConfig
}

func NewDatadogAdapter(config *DatadogConfig) (*DatadogAdapter, error) {
	if config == nil {
		config = DefaultDatadogConfig()
	}

	base := observe.NewAdapterBase(config.BatchSize, config.FlushPeriod)
	adapter := &DatadogAdapter{
		AdapterBase: &base,
		Config:      config,
	}

	adapter.AdapterBase.SetFlusher(adapter)

	return adapter, nil
}

func (d *DatadogAdapter) Start() {
	d.AdapterBase.Start(d)
}

func (d *DatadogAdapter) Stop(wait bool) {
	d.AdapterBase.Stop(wait)
}

func (d *DatadogAdapter) HandleTraceEvent(e observe.TraceEvent) {
	d.AdapterBase.HandleTraceEvent(e)
}

func (d *DatadogAdapter) Flush(evts []observe.TraceEvent) error {
	log.Printf("Flushing %d events\n", len(evts))
	var traces [][]*datadog_formatter.Span
	for _, te := range evts {
		var allSpans []*datadog_formatter.Span
		for _, e := range te.Events {
			switch event := e.(type) {
			case observe.CallEvent:
				traceId := te.TelemetryId.ToUint64()
				spans := d.makeCallSpans(event, nil, traceId)
				if len(spans) > 0 {
					allSpans = append(allSpans, spans...)
				}
			case observe.MemoryGrowEvent:
				log.Println("MemoryGrowEvent should be attached to a span")
			case observe.CustomEvent:
				log.Println("Datadog adapter does not respect custom events")
			}
		}

		if len(allSpans) == 0 {
			log.Println("No spans built for datadog trace")
			return nil
		}

		if te.AdapterMeta != nil {
			if meta, ok := te.AdapterMeta.(DatadogMetadata); ok {
				topSpan := allSpans[0]
				if topSpan.Meta == nil {
					topSpan.Meta = make(map[string]string)
				}
				if meta.ResourceName != nil {
					topSpan.Resource = *meta.ResourceName
				}
				if meta.HttpUrl != nil {
					topSpan.Meta["http.url"] = *meta.HttpUrl
				}
				if meta.HttpStatusCode != nil {
					topSpan.Meta["http.status_code"] = fmt.Sprintf("%d", *meta.HttpStatusCode)
				}
				if meta.HttpClientIp != nil {
					topSpan.Meta["http.client_ip"] = *meta.HttpClientIp
				}
				if meta.HttpRequestContentLength != nil {
					topSpan.Meta["http.request.content_length"] = fmt.Sprintf("%d", *meta.HttpRequestContentLength)
				}
				if meta.HttpRequestContentLengthUncompressed != nil {
					topSpan.Meta["http.request.content_length_uncompressed"] = fmt.Sprintf("%d", *meta.HttpRequestContentLengthUncompressed)
				}
				if meta.HttpResponseContentLength != nil {
					topSpan.Meta["http.response.content_length"] = fmt.Sprintf("%d", *meta.HttpResponseContentLength)
				}
				if meta.HttpResponseContentLengthUncompressed != nil {
					topSpan.Meta["http.response.content_length_uncompressed"] = fmt.Sprintf("%d", *meta.HttpResponseContentLengthUncompressed)
				}
				if meta.SpanKind != nil {
					topSpan.Meta["span.kind"] = meta.SpanKind.String()
				}
			} else {
				log.Println("The Datadog adapter was expecting a DatadogMetadata object on the trace")
			}
		}

		tt := d.Config.TraceType.String()
		allSpans[0].Type = &tt

		traces = append(traces, allSpans)
	}

	output := datadog_formatter.New()
	log.Println("formatted traces ", len(traces))

	for _, trace := range traces {
		output.AddTrace(trace)
	}

	b, err := json.Marshal(output)
	if err != nil {
		log.Println("failed to encode trace data to json", err)
		return nil
	}

	data := bytes.NewBuffer(b)

	url, err := url.JoinPath(d.Config.AgentHost, "v0.3", "traces")
	if err != nil {
		log.Println("failed to create datadog agent endpoint url:", err)
		return nil
	}

	client := http.Client{
		Timeout: time.Second * 2,
	}
	req, err := http.NewRequest("POST", url, data)
	if err != nil {
		log.Println("failed to create datadog endpoint url:", err)
	}

	req.Header = http.Header{
		"content-type": {"application/json"},
	}

	resp, err := client.Do(req)
	if err != nil {
		log.Println("failed to send data to datadog", err)
	}

	if resp.StatusCode != http.StatusOK {
		log.Println("unexpected status code from datadog:", resp.StatusCode)
	}

	return nil
}

func (d *DatadogAdapter) makeCallSpans(event observe.CallEvent, parentId *uint64, traceId uint64) []*datadog_formatter.Span {
	name := event.FunctionName()
	span := datadog_formatter.NewSpan(d.Config.ServiceName, traceId, parentId, name, event.Time, event.Time.Add(event.Duration))

	spans := []*datadog_formatter.Span{span}
	for _, ev := range event.Within() {
		if call, ok := ev.(observe.CallEvent); ok {
			spans = append(spans, d.makeCallSpans(call, &span.SpanId, traceId)...)
		}
		if alloc, ok := ev.(observe.MemoryGrowEvent); ok {
			span := spans[len(spans)-1]
			span.AddAllocation(alloc.MemoryGrowAmount())
		}
	}

	return spans
}

type DatadogMetadata struct {
	HttpUrl                               *string
	HttpMethod                            *string
	HttpStatusCode                        *int
	ResourceName                          *string
	HttpClientIp                          *string
	HttpRequestContentLength              *int
	HttpRequestContentLengthUncompressed  *int
	HttpResponseContentLength             *int
	HttpResponseContentLengthUncompressed *int
	SpanKind                              *DatadogSpanKind
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
