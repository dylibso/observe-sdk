package honeycomb

import (
	"bytes"
	"context"
	"log"
	"net/http"
	"net/url"
	"time"

	observe "github.com/dylibso/observe-sdk/go"
	trace "go.opentelemetry.io/proto/otlp/trace/v1"
	proto "google.golang.org/protobuf/proto"
)

type HoneycombConfig struct {
	ApiKey             string
	Dataset            string
	EmitTracesInterval time.Duration
	TraceBatchMax      uint32
	Host               string
}

type HoneycombAdapter struct {
	*observe.AdapterBase
	Config *HoneycombConfig
}

func NewHoneycombAdapter(config *HoneycombConfig) *HoneycombAdapter {
	base := observe.NewAdapterBase(1, 0)
	adapter := &HoneycombAdapter{
		AdapterBase: &base,
		Config:      config,
	}

	adapter.AdapterBase.SetFlusher(adapter)

	return adapter
}

func (h *HoneycombAdapter) Start(ctx context.Context) {
	h.AdapterBase.Start(ctx, h)
}

func (h *HoneycombAdapter) HandleTraceEvent(te observe.TraceEvent) {
	h.AdapterBase.HandleTraceEvent(te)
}

func (h *HoneycombAdapter) Flush(evts []observe.TraceEvent) error {
	for _, te := range evts {
		traceId := te.TelemetryId.ToHex16()

		var allSpans []*trace.Span
		for _, e := range te.Events {
			switch event := e.(type) {
			case observe.CallEvent: // TODO: consider renaming to FunctionCall for consistency across Rust & JS
				spans := h.MakeOtelCallSpans(event, nil, traceId)
				if len(spans) > 0 {
					allSpans = append(allSpans, spans...)
				}
			case observe.MemoryGrowEvent:
				log.Println("MemoryGrowEvent should be attached to a span")
			case observe.CustomEvent:
				log.Println("Honeycomb adapter does not respect custom events")
			case observe.MetricEvent:
				log.Println("Honeycomb adapter does not respect metric events")
			case observe.LogEvent:
				log.Println("Honeycomb adapter does not respect log events")
			}
		}

		if len(allSpans) == 0 {
			return nil
		}

		t := observe.NewOtelTrace(traceId, h.Config.Dataset, allSpans)
		if te.AdapterMeta != nil {
			meta, ok := te.AdapterMeta.(map[string]string)
			if ok {
				t.SetMetadata(&te, meta)
			} else {
				log.Println("metadata must be of type map[string]string")
			}
		}
		data, err := proto.Marshal(t.TracesData)
		if err != nil {
			log.Println("failed to marshal TracesData:", err)
			return nil
		}

		url, err := url.JoinPath(h.Config.Host, "v1", "traces")
		if err != nil {
			log.Println("failed to create honeycomb endpoint url:", err)
			return nil
		}

		client := http.Client{
			Timeout: time.Second * 2,
		}
		req, err := http.NewRequest("POST", url, bytes.NewBuffer(data))
		if err != nil {
			log.Println("failed to create honeycomb endpoint url:", err)
		}

		req.Header = http.Header{
			"content-type":     {"application/protobuf"},
			"x-honeycomb-team": {h.Config.ApiKey},
		}

		resp, err := client.Do(req)
		if err != nil {
			log.Println("failed to send data to honeycomb", err)
		}

		if resp.StatusCode != http.StatusOK {
			log.Println("unexpected status code from honeycomb:", resp.StatusCode)
		}
	}

	return nil
}
