package observe

import (
	"context"
	"log"
	"time"

	"github.com/tetratelabs/wazero"
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
	minDuration, _ := time.ParseDuration("50us")
	return &Options{
		ChannelBufferSize: 1024,
		SpanFilter: &SpanFilter{
			MinDuration: minDuration,
		},
	}
}
