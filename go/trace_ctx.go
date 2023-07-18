package observe

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"log"
	"time"

	"github.com/tetratelabs/wabin/binary"
	"github.com/tetratelabs/wabin/leb128"
	"github.com/tetratelabs/wabin/wasm"
	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/api"
	"github.com/tetratelabs/wazero/experimental"
)


type Config struct {
	ChannelBufferSize int
	RuntimeConfig     wazero.RuntimeConfig
}

func NewDefaultConfig() *Config {
	return &Config{
		ChannelBufferSize: 1024,
  }
}

type TraceCtx struct {
  adapter chan TraceEvent
  raw chan RawEvent
  events []Event
  stack  []CallEvent
  Config *Config
  names map[uint32]string
  telemetryId TelemetryId
}

func NewTraceCtx(adapter *AdapterBase, data []byte, config *Config) (*TraceCtx, error) {
  names, err := parseNames(data)
  if err != nil {
    return nil, err
  }

  return &TraceCtx {
    adapter: adapter.TraceEvents,
    raw: make(chan RawEvent, config.ChannelBufferSize),
    names: names,
    telemetryId: NewTraceId(),
    Config: config,
  }, nil
}

// Finish() will stop the trace and send the
// TraceEvent payload to the adapter
func (t *TraceCtx) Finish() {
  traceEvent := TraceEvent {
    Events: t.events,
    TelemetryId: &t.telemetryId,
  }
  t.adapter <- traceEvent
  // clear the trace context
  t.events = nil
  t.telemetryId = NewTraceId()
}

func (t *TraceCtx) Names() map[uint32]string {
	return t.names
}

func (t *TraceCtx) pushFunction(ev CallEvent) {
	t.stack = append(t.stack, ev)
}

func (t *TraceCtx) popFunction() (CallEvent, bool) {
	if len(t.stack) == 0 {
		return CallEvent{}, false
	}

	event := t.stack[len(t.stack)-1]
	t.stack = t.stack[:len(t.stack)-1]

	return event, true
}

func (t *TraceCtx) peekFunction() (CallEvent, bool) {
	if len(t.stack) == 0 {
		return CallEvent{}, false
	}

	return t.stack[len(t.stack)-1], true
}

func (t *TraceCtx) WithListener(ctx context.Context) context.Context {
	return context.WithValue(ctx, experimental.FunctionListenerFactoryKey{}, t)
}

func (t *TraceCtx) Init(ctx context.Context, r wazero.Runtime) error {
  ctx = t.WithListener(ctx)
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

func checkVersion(m *wasm.Module) error {
	var minorGlobal *wasm.Export = nil
	var majorGlobal *wasm.Export = nil
	for _, export := range m.ExportSection {
		if export.Type != wasm.ExternTypeGlobal {
			continue
		}

		if export.Name == "wasm_instr_version_minor" {
			minorGlobal = export
		} else if export.Name == "wasm_instr_version_major" {
			majorGlobal = export
		}
	}

	if minorGlobal == nil || majorGlobal == nil {
		return errors.New("wasm_instr_version functions not found")
	}

	minor, _, err := leb128.DecodeUint32(bytes.NewReader(m.GlobalSection[minorGlobal.Index].Init.Data))
	if err != nil {
		return err
	}
	major, _, err := leb128.DecodeUint32(bytes.NewReader(m.GlobalSection[majorGlobal.Index].Init.Data))
	if err != nil {
		return err
	}

	if major != wasmInstrVersionMajor || minor < wasmInstrVersionMinor {
		return errors.New(fmt.Sprintf("Expected instrumentation version >= %d.%d but got %d.%d", wasmInstrVersionMajor, wasmInstrVersionMinor, major, minor))
	}

	return nil
}

func parseNames(data []byte) (map[uint32]string, error) {
	features := wasm.CoreFeaturesV2
	m, err := binary.DecodeModule(data, features)
	if err != nil {
		return nil, err
	}

	// Check for version globals
	if err := checkVersion(m); err != nil {
		return nil, err
	}

	if m.NameSection == nil {
		return nil, errors.New("Name section not found")
	}

  names := make(map[uint32]string, len(m.NameSection.FunctionNames))

	for _, v := range m.NameSection.FunctionNames {
		names[v.Index] = v.Name
	}

	return names, nil
}

