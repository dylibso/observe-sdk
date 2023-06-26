package stdout

import (
	"log"
	"strings"
	"time"

	observe "github.com/dylibso/observe-sdk-wazero"
)

type StdoutAdapter struct {
	observe.AdapterBase
}

func NewStdoutAdapter() StdoutAdapter {
	base := observe.NewAdapterBase()
	return StdoutAdapter{AdapterBase: base}
}

func (s *StdoutAdapter) printEvents(event observe.CallEvent, indentation int) {
	name := event.FunctionName(s)
	log.Println(strings.Repeat("  ", indentation), "Call to", name, "took", event.Duration)
	for _, event := range event.Within() {
		if call, ok := event.(observe.CallEvent); ok {
			s.printEvents(call, indentation+1)
		}
	}
}

func (s *StdoutAdapter) Event(e observe.Event) {
	switch event := e.(type) {
	// TODO: read name from name section instead of printing the function index
	case observe.CallEvent:
		s.printEvents(event, 0)
	case observe.MemoryGrowEvent:
		name := event.FunctionName(s)
		log.Println("Allocated", event.MemoryGrowAmount(), "pages of memory in", name)
	case observe.CustomEvent:
		log.Println(event.Name, event.Time)
	}
}

func (a *StdoutAdapter) Start(collector *observe.Collector, wasm []byte) error {
	if err := a.AdapterBase.Start(collector, wasm); err != nil {
		return err
	}

	go func() {
		for {
			select {
			case event := <-collector.Events:
				a.Event(event)
			case <-a.StopChan():
				return
			}
		}
	}()

	return nil
}

func (a *StdoutAdapter) Wait(timeout time.Duration) {
	a.AdapterBase.Wait(timeout, func() {})
}
