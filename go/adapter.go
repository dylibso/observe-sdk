package observe

import (
	"context"
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/tetratelabs/wazero"
	"go.opentelemetry.io/otel/metric"
	trace "go.opentelemetry.io/proto/otlp/trace/v1"
)

// The primary interface that every Adapter needs to follow
// Start() and Stop() can just call the implementations on AdapterBase
// or provide some custom logic. HandleTraceEvent is called after
// an invocation of a wasm module is done and all events are collected.
type Adapter interface {
	Start(context.Context)
	Stop(wait bool)
	HandleTraceEvent(e TraceEvent)
}

// The payload that contains all the Events
// from a single wasm module invocation
type TraceEvent struct {
	Events      []Event
	TelemetryId TelemetryId
	AdapterMeta interface{}
}

// Shared implementation for all Adapters
type AdapterBase struct {
	TraceEvents chan TraceEvent

	stop        chan bool
	eventBucket *EventBucket
	flusher     Flusher
}

func (a *AdapterBase) NewTraceCtx(ctx context.Context, r wazero.Runtime, wasm []byte, opts *Options) (*TraceCtx, error) {
	if opts == nil {
		opts = NewDefaultOptions()
	}
	return newTraceCtx(ctx, a.TraceEvents, r, wasm, opts)
}

func NewAdapterBase(batchSize int, flushPeriod time.Duration) AdapterBase {
	bucket := NewEventBucket(batchSize, flushPeriod)
	return AdapterBase{
		TraceEvents: make(chan TraceEvent, 100),
		eventBucket: bucket,
	}
}

func (b *AdapterBase) SetFlusher(f Flusher) {
	b.flusher = f
}

func (b *AdapterBase) HandleTraceEvent(te TraceEvent) {
	b.eventBucket.addEvent(te, b.flusher)
}

func (b *AdapterBase) Start(ctx context.Context, a Adapter) {
	b.stop = make(chan bool)

	go func() {
		for {
			select {
			case <-ctx.Done():
				log.Println("context cancelled")
				return
			case event := <-b.TraceEvents:
				a.HandleTraceEvent(event)
			case <-b.stop:
				return
			}
		}
	}()
}

// Stops the adapter and waits for all flushes to complete.
// Set wait parameter to false if you don't want to wait
func (b *AdapterBase) Stop(wait bool) {
	b.stop <- true
	if wait {
		b.eventBucket.Wait()
	}
}

// MakeOtelCallSpans recursively constructs call spans in open telemetry format
func (b *AdapterBase) MakeOtelCallSpans(event CallEvent, parentId []byte, traceId string, meter *metric.Meter) []*trace.Span {
	name := event.FunctionName()
	span := NewOtelSpan(traceId, parentId, name, event.Time, event.Time.Add(event.Duration))
	span.Attributes = append(span.Attributes, NewOtelKeyValueString("function-name", fmt.Sprintf("function-call-%s", name)))

	spans := []*trace.Span{span}
	for _, ev := range event.Within() {
		if call, ok := ev.(CallEvent); ok {
			spans = append(spans, b.MakeOtelCallSpans(call, span.SpanId, traceId, meter)...)
		}
		if alloc, ok := ev.(MemoryGrowEvent); ok {
			last := spans[len(spans)-1]

			kv := NewOtelKeyValueInt64("allocation", int64(alloc.MemoryGrowAmount()))
			i, existing := GetOtelAttrFromSpan("allocation", last)
			if existing != nil {
				last.Attributes[i] = AddOtelKeyValueInt64(kv, existing)
			} else {
				last.Attributes = append(last.Attributes, kv)
			}
		}
		if tags, ok := ev.(SpanTagsEvent); ok {
			last := spans[len(spans)-1]

			for _, tag := range tags.Tags {
				parts := strings.Split(tag, ":")
				if len(parts) != 2 {
					log.Printf("Invalid tag: %s\n", tag)
					continue
				}

				kv := NewOtelKeyValueString(parts[0], parts[1])
				last.Attributes = append(last.Attributes, kv)
			}
		}
		if metric, ok := ev.(MetricEvent); ok && meter != nil {

			if metric.Format != Statsd {
				log.Printf("Unsupported metric format: %v\n", metric.Format)
				continue
			}

			datagram, err := parseStatsdDataGram(metric.Message)
			if err != nil {
				log.Printf("Failed to parse statsd datagram: %v\n", err)
				continue
			}

			ctx := context.Background()

			m := *meter

			// TODO: maybe we should also support int64 metrics?
			// TODO: double check this
			// TODO: timestamps?
			switch datagram.Type {
			case StatsdCounter:
				counter, _ := m.Float64Counter(datagram.Name)
				counter.Add(ctx, datagram.Value)
			case StatsdGauge:
				gauge, _ := m.Float64UpDownCounter(datagram.Name)
				gauge.Add(ctx, datagram.Value)
			case StatsdTiming, StatsdHistogram:
				histogram, _ := m.Float64Histogram(datagram.Name)
				histogram.Record(ctx, datagram.Value)
			case StatsdSet:
				// TODO: how to support sets?
			}
		}
		if log, ok := ev.(LogEvent); ok {
			// TODO: since logs are not implemented in otel go, can we use span events instead?
			last := spans[len(spans)-1]
			event := trace.Span_Event{
				Name:         log.Message,
				TimeUnixNano: uint64(log.Time.UnixNano()),
			}

			last.Events = append(last.Events, &event)
		}
	}
	return spans
}

// Definition of how to filter our Spans to reduce noise
type SpanFilter struct {
	MinDuration time.Duration
}

// Specify options to change what or how the adapter receives ObserveEvents
type Options struct {
	SpanFilter        *SpanFilter
	ChannelBufferSize int
}

// Create a default configuration
func NewDefaultOptions() *Options {
	return &Options{
		ChannelBufferSize: 1024,
		SpanFilter: &SpanFilter{
			MinDuration: time.Microsecond * 20,
		},
	}
}
