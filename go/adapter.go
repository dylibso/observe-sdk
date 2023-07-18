package observe

import "log"

type Adapter interface {
  Start()
  Stop()
  HandleTraceEvent(TraceEvent)
}

type TraceEvent struct {
  Events      []Event
  TelemetryId *TelemetryId
}

type AdapterBase struct {
  TraceEvents chan TraceEvent
  stop        chan bool
}

func (a *AdapterBase) NewTraceCtx(wasm []byte, config *Config) (*TraceCtx, error) {
  if config == nil {
    config = NewDefaultConfig()
  }
  return NewTraceCtx(a, wasm, config)
}

func NewAdapterBase() AdapterBase {
  return AdapterBase{
    // TODO set to some kind of max, add dump logic
    TraceEvents: make(chan TraceEvent, 100),
  }
}

func (b *AdapterBase) Start(a Adapter) {
  b.stop = make(chan bool)

  go func() {
    for {
      select {
      case event := <-b.TraceEvents:
        log.Println("Adapter Got TraceEvent")
        a.HandleTraceEvent(event)
      case <-b.stop:
        log.Println("Adapter Stopped")
        return
      }
    }
  }()
}

func (b *AdapterBase) Stop() {
  log.Println("Stopping adapter")
  b.stop <- true
}
