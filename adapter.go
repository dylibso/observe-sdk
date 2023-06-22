package dylibso_observe

import (
	"bytes"
	"errors"
	"fmt"
	"log"
	"time"

	"github.com/tetratelabs/wabin/binary"
	"github.com/tetratelabs/wabin/leb128"
	"github.com/tetratelabs/wabin/wasm"
)

type Adapter interface {
	Start(collector Collector)
	Stop()
	Event(Event)
	Names() map[uint32]string
}

type AdapterBase struct {
	stop      chan bool
	Collector Collector
	names     map[uint32]string
}

type StdoutAdapter struct {
	AdapterBase
}

func (a AdapterBase) Names() map[uint32]string {
	return a.names
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

func (a *AdapterBase) GetNames(data []byte) error {
	features := wasm.CoreFeaturesV2
	m, err := binary.DecodeModule(data, features)
	if err != nil {
		return err
	}

	// Check for version globals
	if err := checkVersion(m); err != nil {
		return err
	}

	if m.NameSection == nil {
		return errors.New("Name section not found")
	}

	a.names = map[uint32]string{}

	for _, v := range m.NameSection.FunctionNames {
		a.names[v.Index] = v.Name
	}

	return nil
}

func NewAdataperBase(wasm []byte) (AdapterBase, error) {
	a := AdapterBase{
		stop:      make(chan bool, 1),
		Collector: Collector{},
	}
	err := a.GetNames(wasm)
	if err != nil {
		return AdapterBase{}, err
	}
	return a, nil
}

func NewStdoutAdapter(wasm []byte) (StdoutAdapter, error) {
	base, err := NewAdataperBase(wasm)
	if err != nil {
		return StdoutAdapter{}, err
	}
	return StdoutAdapter{AdapterBase: base}, nil
}

func (s *StdoutAdapter) Event(e Event) {
	switch event := e.(type) {
	// TODO: read name from name section instead of printing the function index
	case CallEvent:
		name := event.FunctionName(s)
		log.Println("Call to", name, "took", event.Duration)
	case MemoryGrowEvent:
		name := event.FunctionName(s)
		log.Println("Allocated", event.MemoryGrowAmount(), "pages of memory in", name)
	case CustomEvent:
		log.Println(event.Name, event.Time)
	}
}

func (a *AdapterBase) Stop() {
	a.stop <- true
}

func (a *StdoutAdapter) Start(collector Collector) {
	go func() {
		for {
			select {
			case event := <-collector.Events:
				a.Event(event)
			case <-a.stop:
				return
			}
		}
	}()
}

func (a *StdoutAdapter) Wait(collector Collector, timeout time.Duration) {
	select {
	case <-time.After(timeout):
		if len(collector.Events) > 0 {
			a.Wait(collector, timeout)
			return
		}
		return
	}
}
