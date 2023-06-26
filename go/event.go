package dylibso_observe

import (
	"time"

	"github.com/ianlancetaylor/demangle"
	"github.com/tetratelabs/wazero/api"
)

type RawEventKind int

const (
	RawEnter RawEventKind = iota
	RawExit
	RawMemoryGrow
)

type EventKind int

const (
	Call EventKind = iota
	MemoryGrow
	Custom
)

type RawEvent struct {
	Kind             RawEventKind
	Stack            []api.FunctionDefinition
	FunctionIndex    uint32
	FunctionName     string
	MemoryGrowAmount uint32
	Time             time.Time
	Duration         time.Duration
}

type Event interface {
	RawEvents() []RawEvent
}

type CallEvent struct {
	Raw      []RawEvent
	Time     time.Time
	Duration time.Duration
	within   []Event
}

func (e *CallEvent) Stop(at time.Time) {
	e.Duration = at.Sub(e.Time)
}

func (e CallEvent) RawEvents() []RawEvent {
	return e.Raw
}

func (e CallEvent) Within() []Event {
	return e.within
}

type CustomEvent struct {
	Time     time.Time
	Name     string
	Metadata map[string]interface{}
}

func NewCustomEvent(name string) CustomEvent {
	return CustomEvent{
		Time:     time.Now(),
		Name:     name,
		Metadata: map[string]interface{}{},
	}
}

func (e CustomEvent) RawEvents() []RawEvent {
	return []RawEvent{}
}

type MemoryGrowEvent struct {
	Raw  RawEvent
	Time time.Time
}

func (e MemoryGrowEvent) RawEvents() []RawEvent {
	return []RawEvent{e.Raw}
}

func (e MemoryGrowEvent) FunctionName() string {
	s, err := demangle.ToString(e.Raw.FunctionName)
	if err != nil {
		return e.Raw.FunctionName
	}
	return s
}

func (e MemoryGrowEvent) FunctionIndex() uint32 {
	return e.Raw.FunctionIndex
}

func (e CallEvent) FunctionName() string {
	s, err := demangle.ToString(e.Raw[0].FunctionName)
	if err != nil {
		return e.Raw[0].FunctionName
	}
	return s
}

func (e CallEvent) FunctionIndex() uint32 {
	return e.Raw[0].FunctionIndex
}

func (e MemoryGrowEvent) MemoryGrowAmount() uint32 {
	return e.Raw.MemoryGrowAmount
}
