package dylibso_observe

import (
	"context"

	"github.com/tetratelabs/wazero/api"
	"github.com/tetratelabs/wazero/experimental"
)

func (c Collector) NewListener(def api.FunctionDefinition) experimental.FunctionListener {
	if def.GoFunction() == nil {
		return nil
	}
	return c
}

func (c Collector) Before(ctx context.Context, _ api.Module, def api.FunctionDefinition, inputs []uint64, stack experimental.StackIterator) context.Context {
	var event RawEvent
	name := def.Name()

	switch name {
	case "instrument_enter":
		event.Kind = RawEnter
		event.FunctionIndex = uint32(inputs[0])
	case "instrument_exit":
		event.Kind = RawExit
		event.FunctionIndex = uint32(inputs[0])
	case "instrument_memory_grow":
		event.Kind = RawMemoryGrow
		event.MemoryGrowAmount = uint32(inputs[0])
	default:
		return ctx
	}
	for stack.Next() {
		f := stack.FunctionDefinition()
		event.Stack = append(event.Stack, f)
	}
	go func() {
		c.raw <- event
	}()
	return ctx
}

func (c Collector) After(ctx context.Context, _ api.Module, def api.FunctionDefinition, _ error, _ []uint64) {
}

func (c Collector) Abort(_ context.Context, _ api.Module, _ api.FunctionDefinition, _ error) {
}
