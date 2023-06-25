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

func NewStdoutAdapter(wasm []byte) (StdoutAdapter, error) {
	base, err := observe.NewAdataperBase(wasm)
	if err != nil {
		return StdoutAdapter{}, err
	}

	return StdoutAdapter{AdapterBase: base}, nil
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

func (a *StdoutAdapter) Start(collector observe.Collector) {
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
}

func (a *StdoutAdapter) Wait(collector observe.Collector, timeout time.Duration) {
	select {
	case <-time.After(timeout):
		if len(collector.Events) > 0 {
			a.Wait(collector, timeout)
			return
		}
		return
	}
}
