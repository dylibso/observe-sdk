package observe

import (
	"context"

	"github.com/tetratelabs/wazero/api"
	"github.com/tetratelabs/wazero/experimental"
)

func (t *TraceCtx) NewListener(def api.FunctionDefinition) experimental.FunctionListener {
	if def.GoFunction() == nil {
		return nil
	}
	return t
}

func (t *TraceCtx) NewFunctionListener(_ api.FunctionDefinition) experimental.FunctionListener {
	return t
}

func (t *TraceCtx) Before(ctx context.Context, _ api.Module, def api.FunctionDefinition, inputs []uint64, stack experimental.StackIterator) {
	var event RawEvent
	name := def.Name()

	switch name {
	case "instrument_enter":
		event.Kind = RawEnter
		event.FunctionIndex = uint32(inputs[0])
		event.FunctionName = t.names[event.FunctionIndex]
	case "instrument_exit":
		event.Kind = RawExit
		event.FunctionIndex = uint32(inputs[0])
		event.FunctionName = t.names[event.FunctionIndex]
	case "instrument_memory_grow":
		event.Kind = RawMemoryGrow
		event.MemoryGrowAmount = uint32(inputs[0])
	default:
		return
	}
	for stack.Next() {
		f := stack.Function()
		event.Stack = append(event.Stack, f)
	}
	t.raw <- event
}

func (t *TraceCtx) After(context.Context, api.Module, api.FunctionDefinition, []uint64) {}

func (t *TraceCtx) Abort(context.Context, api.Module, api.FunctionDefinition, error) {}
