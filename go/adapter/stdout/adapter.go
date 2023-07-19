package stdout

import (
	"log"
	"strings"

	observe "github.com/dylibso/observe-sdk/go"
)

type StdoutAdapter struct {
	observe.AdapterBase
}

func NewStdoutAdapter() StdoutAdapter {
	base := observe.NewAdapterBase()
	return StdoutAdapter{AdapterBase: base}
}

func (s *StdoutAdapter) printEvents(event observe.CallEvent, indentation int) {
	name := event.FunctionName()
	log.Println(strings.Repeat("  ", indentation), "Call to", name, "took", event.Duration)
	for _, event := range event.Within() {
		if call, ok := event.(observe.CallEvent); ok {
			s.printEvents(call, indentation+1)
		}
		if alloc, ok := event.(observe.MemoryGrowEvent); ok {
			log.Println(strings.Repeat("  ", indentation), "Allocated", alloc.MemoryGrowAmount(), "pages of memory in", name)
		}
	}

}
func (s *StdoutAdapter) HandleTraceEvent(te observe.TraceEvent) {
	for _, e := range te.Events {
		switch event := e.(type) {
		case observe.CallEvent:
			s.printEvents(event, 0)
		case observe.MemoryGrowEvent:
			name := event.FunctionName()
			log.Println("Allocated", event.MemoryGrowAmount(), "pages of memory in", name)
		case observe.CustomEvent:
			log.Println(event.Name, event.Time)
		}
	}
}

func (s *StdoutAdapter) Start() {
	s.AdapterBase.Start(s)
}

func (s *StdoutAdapter) Stop() {
	s.AdapterBase.Stop()
}
