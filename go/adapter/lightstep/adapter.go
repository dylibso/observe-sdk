package lightstep

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

type LightstepConfig struct {
	ApiKey             string
	ServiceName        string
	EmitTracesInterval time.Duration
	TraceBatchMax      uint32
	Host               string
}

type LightstepAdapter struct {
	*observe.AdapterBase
	Config *LightstepConfig
}

func NewLightstepAdapter(config *LightstepConfig) *LightstepAdapter {
	base := observe.NewAdapterBase(1, 0)
	adapter := &LightstepAdapter{
		AdapterBase: &base,
		Config:      config,
	}

	adapter.AdapterBase.SetFlusher(adapter)

	return adapter
}

func (l *LightstepAdapter) Start(ctx context.Context) {
	l.AdapterBase.Start(ctx, l)
}

func (l *LightstepAdapter) HandleTraceEvent(te observe.TraceEvent) {
	l.AdapterBase.HandleTraceEvent(te)
}

func (l *LightstepAdapter) Flush(evts []observe.TraceEvent) error {
	for _, te := range evts {
		traceId := te.TelemetryId.ToHex16()

		var allSpans []*trace.Span
		for _, e := range te.Events {
			switch event := e.(type) {
			case observe.CallEvent: // TODO: consider renaming to FunctionCall for consistency across Rust & JS
				spans := l.MakeOtelCallSpans(event, nil, traceId, nil)
				if len(spans) > 0 {
					allSpans = append(allSpans, spans...)
				}
			case observe.MemoryGrowEvent:
				log.Println("MemoryGrowEvent should be attached to a span")
			case observe.CustomEvent:
				log.Println("lightstep adapter does not respect custom events")
			}
		}

		if len(allSpans) == 0 {
			return nil
		}

		t := observe.NewOtelTrace(traceId, l.Config.ServiceName, allSpans)
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

		url, err := url.JoinPath(l.Config.Host, "traces", "otlp", "v0.9")
		if err != nil {
			log.Println("failed to create lightstep endpoint url:", err)
			return nil
		}

		client := http.Client{
			Timeout: time.Second * 2,
		}
		req, err := http.NewRequest("POST", url, bytes.NewBuffer(data))
		if err != nil {
			log.Println("failed to create lightstep endpoint url:", err)
		}

		req.Header = http.Header{
			"content-type":           {"application/x-protobuf"},
			"lightstep-access-token": {l.Config.ApiKey},
		}

		resp, err := client.Do(req)
		if err != nil {
			log.Println("failed to send data to lightstep", err)
		}

		if resp.StatusCode != http.StatusOK {
			log.Println("unexpected status code from lightstep:", resp.StatusCode)
		}

	}

	return nil
}
