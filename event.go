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
	ModuleBegin
)

type RawEvent struct {
	Kind             RawEventKind
	Stack            []api.FunctionDefinition
	FunctionIndex    uint32
	FunctionName     string
	MemoryGrowAmount uint64
}

type Event interface {
	RawEvents() []RawEvent
}

type CallEvent struct {
	Raw      []RawEvent
	Time     time.Time
	Duration time.Duration
}

func (e CallEvent) RawEvents() []RawEvent {
	return e.Raw
}

type ModuleBeginEvent struct {
	Time time.Time
	Name string
}

func (e ModuleBeginEvent) RawEvents() []RawEvent {
	return []RawEvent{}
}

type MemoryGrowEvent struct {
	Raw  RawEvent
	Time time.Time
}

func (e MemoryGrowEvent) RawEvents() []RawEvent {
	return []RawEvent{e.Raw}
}

func (e MemoryGrowEvent) DemangledFunctionName() string {
	s, err := demangle.ToString(e.Raw.FunctionName)
	if err != nil {
		return e.Raw.FunctionName
	}
	return s
}

func (e MemoryGrowEvent) FunctionName() string {
	return e.Raw.FunctionName
}

func (e MemoryGrowEvent) FunctionIndex() uint32 {
	return e.Raw.FunctionIndex
}

func (e CallEvent) FunctionName() string {
	return e.Raw[0].FunctionName
}

func (e CallEvent) DemangledFunctionName() string {
	s, err := demangle.ToString(e.Raw[0].FunctionName)
	if err != nil {
		return e.Raw[0].FunctionName
	}
	return s
}

func (e CallEvent) FunctionIndex() uint32 {
	return e.Raw[0].FunctionIndex
}

func (e MemoryGrowEvent) MemoryGrowAmount() uint64 {
	return e.Raw.MemoryGrowAmount
}
