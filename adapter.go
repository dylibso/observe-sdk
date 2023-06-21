package dylibso_observe

import (
	"errors"
	"log"
	"time"

	"github.com/tetratelabs/wabin/binary"
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

func (a *AdapterBase) GetNames(data []byte) error {
	features := wasm.CoreFeaturesV2
	m, err := binary.DecodeModule(data, features)
	if err != nil {
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

func NewAdataperBase(wasm []byte) AdapterBase {
	a := AdapterBase{
		stop:      make(chan bool, 1),
		Collector: Collector{},
	}
	err := a.GetNames(wasm)
	if err != nil {
		log.Println("WARNING name parsing failed", err)
	}
	return a
}

func NewStdoutAdapter(wasm []byte) StdoutAdapter {
	return StdoutAdapter{AdapterBase: NewAdataperBase(wasm)}
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
