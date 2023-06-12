package dylibso_observe

import (
	"log"
	"time"
)

type Adapter interface {
	Start(collector Collector)
	Stop()
	Event(Event)
}

type StdoutAdapter struct {
	stop      chan bool
	collector Collector
}

func (s StdoutAdapter) Event(e Event) {
	switch event := e.(type) {
	case CallEvent:
		log.Println("Call to", event.FunctionName(), "took", event.Duration)
	case MemoryGrowEvent:
		log.Println("Allocated", event.MemoryGrowAmount(), "pages of memory in", event.FunctionName())
	case ModuleBeginEvent:
		log.Println("Starting module:", event.Name)
	}
}

func (a StdoutAdapter) Stop() {
	a.stop <- true
}

func (a StdoutAdapter) Start(collector Collector) {
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

func (a StdoutAdapter) Wait(collector Collector, timeout time.Duration) {
	for {
		select {
		case <-time.After(timeout):
			if len(collector.Events) > 0 {
				continue
			}
			return
		}
	}
}
