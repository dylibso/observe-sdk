package observe

import (
	"context"
	"log"
	"time"

	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/api"
	"github.com/tetratelabs/wazero/experimental"
)

// The configuration object for the observe SDK
type Config struct {
	ChannelBufferSize int
}

// Create a default configuration
func NewDefaultConfig() *Config {
	return &Config{
		ChannelBufferSize: 1024,
	}
}

// TraceCtx holds the context for a trace, or wasm module invocation.
// It collects holds a channel to the Adapter and from the wazero Listener
// It will collect events throughout the invocation of the function. Calling
// Finish() will then submit those events to the Adapter to be processed and sent
type TraceCtx struct {
	adapter     chan TraceEvent
	raw         chan RawEvent
	events      []Event
	stack       []CallEvent
	Config      *Config
	names       map[uint32]string
	telemetryId TelemetryId
}

// Creates a new TraceCtx. Used internally by the Adapter. The user should create the trace context from the Adapter.
func newTraceCtx(ctx context.Context, adapter *AdapterBase, r wazero.Runtime, data []byte, config *Config) (*TraceCtx, error) {
	names, err := parseNames(data)
	if err != nil {
		return nil, err
	}

	traceCtx := &TraceCtx{
		adapter:     adapter.TraceEvents,
		raw:         make(chan RawEvent, config.ChannelBufferSize),
		names:       names,
		telemetryId: NewTraceId(),
		Config:      config,
	}

	err = traceCtx.init(ctx, r)
	if err != nil {
		return nil, err
	}

	return traceCtx, nil
}

// Finish() will stop the trace and send the
// TraceEvent payload to the adapter
func (t *TraceCtx) Finish() {
	traceEvent := TraceEvent{
		Events:      t.events,
		TelemetryId: &t.telemetryId,
	}
	t.adapter <- traceEvent
	// clear the trace context
	t.events = nil
	t.telemetryId = NewTraceId()
}

// Attaches the wazero FunctionListener to the context
func (t *TraceCtx) withListener(ctx context.Context) context.Context {
	return context.WithValue(ctx, experimental.FunctionListenerFactoryKey{}, t)
}

// Initializes the TraceCtx. This connects up the channels with events from the FunctionListener.
// Should only be called once.
func (t *TraceCtx) init(ctx context.Context, r wazero.Runtime) error {
	ctx = t.withListener(ctx)
	observe := r.NewHostModuleBuilder("dylibso_observe")
	functions := observe.NewFunctionBuilder()

	functions.WithFunc(func(ctx context.Context, m api.Module, i int32) {
		start := time.Now()
		ev := <-t.raw
		if ev.Kind != RawEnter {
			log.Println("Expected event", RawEnter, "but got", ev.Kind)
		}
		t.pushFunction(CallEvent{Raw: []RawEvent{ev}, Time: start})
	}).Export("instrument_enter")

	functions.WithFunc(func(ctx context.Context, i int32) {
		end := time.Now()
		ev := <-t.raw
		if ev.Kind != RawExit {
			log.Println("Expected event", RawExit, "but got", ev.Kind)
			return
		}
		fn, ok := t.peekFunction()
		if !ok {
			log.Println("Expected values on started function stack, but none were found")
			return
		}
		if ev.FunctionIndex != fn.FunctionIndex() {
			log.Println("Expected call to", ev.FunctionIndex, "but found call to", fn.FunctionIndex())
			return
		}

		fn, _ = t.popFunction()
		fn.Stop(end)
		fn.Raw = append(fn.Raw, ev)

		f, ok := t.popFunction()
		if !ok {
			t.events = append(t.events, fn)
			return
		}

		f.within = append(f.within, fn)
		t.pushFunction(f)
	}).Export("instrument_exit")

	functions.WithFunc(func(ctx context.Context, amt int32) {
		ev := <-t.raw
		if ev.Kind != RawMemoryGrow {
			log.Println("Expected event", MemoryGrow, "but got", ev.Kind)
			return
		}

		if len(t.stack) > 0 {
			f := t.stack[len(t.stack)-1]
			ev.FunctionIndex = f.FunctionIndex()
			ev.FunctionName = f.FunctionName()
		}

		event := MemoryGrowEvent{
			Raw:  ev,
			Time: time.Now(),
		}

		t.events = append(t.events, event)
	}).Export("instrument_memory_grow")

	_, err := observe.Instantiate(ctx)
	if err != nil {
		return err
	}
	return nil
}

// Pushes a function onto the stack
func (t *TraceCtx) pushFunction(ev CallEvent) {
	t.stack = append(t.stack, ev)
}

// Pops a function off the stack
func (t *TraceCtx) popFunction() (CallEvent, bool) {
	if len(t.stack) == 0 {
		return CallEvent{}, false
	}

	event := t.stack[len(t.stack)-1]
	t.stack = t.stack[:len(t.stack)-1]

	return event, true
}

// Peek at the function on top of the stack without modifying
func (t *TraceCtx) peekFunction() (CallEvent, bool) {
	if len(t.stack) == 0 {
		return CallEvent{}, false
	}

	return t.stack[len(t.stack)-1], true
}
