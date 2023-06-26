package observe

import (
	"bytes"
	"errors"
	"fmt"
	"math/rand"
	"time"

	"github.com/tetratelabs/wabin/leb128"
	"github.com/tetratelabs/wabin/wasm"
)

type Adapter interface {
	Start(collector *Collector, wasm []byte) error
	Stop(collector *Collector)
	Event(Event)
}

type AdapterBase struct {
	Collectors map[*Collector]chan bool
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

func (a *AdapterBase) Wait(collector *Collector, timeout time.Duration, callback func()) {
	for {
		select {
		case <-time.After(timeout):
			if len(collector.Events) > 0 {
				if callback != nil {
					callback()
				}
				continue
			}
			a.RemoveCollector(collector)
			return
		}
	}
}

func NewAdapterBase() AdapterBase {
	a := AdapterBase{
		Collectors: map[*Collector]chan bool{},
	}
	return a
}

func (a *AdapterBase) Start(collector *Collector, wasm []byte) error {
	a.Collectors[collector] = make(chan bool, 1)
	return collector.GetNames(wasm)
}

func (a *AdapterBase) RemoveCollector(collector *Collector) {
	delete(a.Collectors, collector)
}

func (a *AdapterBase) Stop(collector *Collector) {
	stop, ok := a.Collectors[collector]
	if ok {
		stop <- true
		a.RemoveCollector(collector)
	}
}

func (a AdapterBase) StopChan(collector *Collector) chan bool {
	return a.Collectors[collector]
}

type TelemetryId uint64

var rng rand.Source

func init() {
	rng = rand.NewSource(time.Now().UnixNano())
}

func NewTraceId() TelemetryId {
	return TelemetryId(rng.Int63())
}

func NewSpanId() TelemetryId {
	return TelemetryId(rng.Int63())
}

func (t TelemetryId) ToHex8() string {
	return fmt.Sprintf("%016x", t)
}

func (t TelemetryId) ToHex16() string {
	return fmt.Sprintf("%032x", t)
}
