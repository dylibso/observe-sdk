package observe

import (
	"context"

	"github.com/tetratelabs/wazero"
)

// The primary interface that every Adapter needs to follow
// Start() and Stop() can just call the implementations on AdapterBase
// or provide some custom logic. HandleTraceEvent is called after
// an invocation of a wasm module is done and all events are collected.
type Adapter interface {
	Start()
	Stop()
	HandleTraceEvent(TraceEvent)
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
}

func (a *AdapterBase) NewTraceCtx(ctx context.Context, r wazero.Runtime, wasm []byte, config *Config) (*TraceCtx, error) {
	if config == nil {
		config = NewDefaultConfig()
	}
	return newTraceCtx(ctx, a, r, wasm, config)
}

func NewAdapterBase() AdapterBase {
	return AdapterBase{
		// TODO set to some kind of max, add dump logic
		TraceEvents: make(chan TraceEvent, 100),
	}
}

func (b *AdapterBase) Start(a Adapter) {
	b.stop = make(chan bool)

	go func() {
		for {
			select {
			case event := <-b.TraceEvents:
				a.HandleTraceEvent(event)
			case <-b.stop:
				return
			}
		}
	}()
}

func (b *AdapterBase) Stop() {
	b.stop <- true
}
