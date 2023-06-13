package dylibso_observe

import (
	"log"
	"time"

	"github.com/ianlancetaylor/demangle"
	"github.com/tetratelabs/wazero"
)

type Adapter interface {
	Start(collector Collector)
	Stop()
	Event(Event)
}

type AdapterBase struct {
	StopChan  chan bool
	Collector Collector
	Names     map[uint32]string
}

type StdoutAdapter struct {
	AdapterBase
}

func (a *AdapterBase) GetNames(cm wazero.CompiledModule) {
	d := map[uint32]string{}

	for name, f := range cm.ExportedFunctions() {
		d[f.Index()] = name
	}

	for _, f := range cm.ImportedFunctions() {
		d[f.Index()] = f.Name()
	}

	a.Names = d
}

func NewAdataperBase(cm wazero.CompiledModule) AdapterBase {
	a := AdapterBase{
		StopChan:  make(chan bool, 1),
		Collector: Collector{},
	}
	a.GetNames(cm)
	return a
}

func NewStdoutAdapter(cm wazero.CompiledModule) StdoutAdapter {
	return StdoutAdapter{AdapterBase: NewAdataperBase(cm)}
}

func (s *StdoutAdapter) Event(e Event) {
	switch event := e.(type) {
	case CallEvent:
		name := event.DemangledFunctionName()
		log.Println("Call to", name, "took", event.Duration)
	case MemoryGrowEvent:
		name := event.DemangledFunctionName()
		name, _ = demangle.ToString(name)
		log.Println("Allocated", event.MemoryGrowAmount(), "pages of memory in", name)
	case ModuleBeginEvent:
		log.Println("Starting module:", event.Name)
	}
}

func (a *StdoutAdapter) Stop() {
	a.StopChan <- true
}

func (a *StdoutAdapter) Start(collector Collector) {
	go func() {
		for {
			select {
			case event := <-collector.Events:
				a.Event(event)
			case <-a.StopChan:
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
