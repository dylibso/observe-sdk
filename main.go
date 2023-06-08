package main

import (
	"context"
	"log"
	"os"

	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/api"
	"github.com/tetratelabs/wazero/experimental"
	"github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
	"github.com/tetratelabs/wazero/sys"
)

type Collector struct {
	Event   chan RawEvent
	Adapter Adapter
}

type Adapter interface {
	Event(event RawEvent)
}

type StdoutAdapter struct {
}

func (s StdoutAdapter) Event(event RawEvent) {
	switch event.Kind {
	case Enter:
		log.Println("Enter", event.FunctionName)
	case Exit:
		log.Println("Exit", event.FunctionName)
	case MemoryGrow:
		log.Println("MemoryGrow", event.FunctionName)
	}
}

func NewCollector(a Adapter) Collector {
	return Collector{
		Event:   make(chan RawEvent, 1024),
		Adapter: a,
	}
}

type EventKind int

const (
	Enter EventKind = iota
	Exit
	MemoryGrow
)

type RawEvent struct {
	Kind          EventKind
	Stack         []api.FunctionDefinition
	FunctionIndex uint32
	FunctionName  string
}

func (c Collector) NewListener(def api.FunctionDefinition) experimental.FunctionListener {
	if def.GoFunction() == nil {
		return nil
	}
	return c
}

func (c Collector) Before(ctx context.Context, _ api.Module, def api.FunctionDefinition, _ []uint64, stack experimental.StackIterator) context.Context {
	var event RawEvent
	name := def.Name()

	switch name {
	case "instrument_enter":
		event.Kind = Enter
	case "instrument_exit":
		event.Kind = Exit
	case "instrument_memory_grow":
		event.Kind = MemoryGrow
	default:
		return ctx
	}
	stack.Next()
	for stack.Next() {
		f := stack.FunctionDefinition()
		if len(event.Stack) == 0 {
			event.FunctionName = f.Name()
			event.FunctionIndex = f.Index()
		}
		event.Stack = append(event.Stack, f)
	}
	c.Event <- event
	return ctx
}

func (c Collector) After(ctx context.Context, _ api.Module, def api.FunctionDefinition, _ error, _ []uint64) {
}

func (c Collector) Abort(_ context.Context, _ api.Module, _ api.FunctionDefinition, _ error) {
}

func main() {
	u := NewCollector(StdoutAdapter{})
	ctx := context.WithValue(context.Background(), experimental.FunctionListenerFactoryKey{}, u)

	r := wazero.NewRuntime(ctx)
	defer r.Close(ctx) // This closes everything this Runtime created.

	wasi_snapshot_preview1.MustInstantiate(ctx, r)

	wasm, err := os.ReadFile(os.Args[1])
	if err != nil {
		log.Panicln(err)
	}

	observe := r.NewHostModuleBuilder("dylibso_observe")
	functions := observe.NewFunctionBuilder()
	functions.WithFunc(func(ctx context.Context) {
		// TODO: get Enter event and pass to adapter
		ev := <-u.Event
		if ev.Kind != Enter {
			log.Panicln("XXX")
		}
		u.Adapter.Event(ev)
	}).Export("instrument_enter")
	functions.WithFunc(func(ctx context.Context) {
		// TODO: get Exit event and pass to adapter
		ev := <-u.Event
		if ev.Kind != Exit {
			log.Panicln("XXX")
		}
		u.Adapter.Event(ev)
	}).Export("instrument_exit")
	functions.WithFunc(func(ctx context.Context, amt int32) {
		// TODO: get MemoryGrow event and pass to adapter
		ev := <-u.Event
		if ev.Kind != MemoryGrow {
			log.Panicln("XXX")
		}
		u.Adapter.Event(ev)
	}).Export("instrument_memory_grow")
	_, err = observe.Instantiate(ctx)
	if err != nil {
		log.Panicln(err)
	}

	config := wazero.NewModuleConfig().WithStdin(os.Stdin).WithStdout(os.Stdout).WithStderr(os.Stderr).WithArgs(os.Args[2:]...).WithStartFunctions("_start")
	m, err := r.InstantiateWithConfig(ctx, wasm, config)
	if err != nil {
		log.Panicln(err)
	}
	defer m.Close(ctx)

	f := m.ExportedFunction("_start")
	f.Call(ctx)
	if exitErr, ok := err.(*sys.ExitError); ok && exitErr.ExitCode() != 0 {
		log.Printf("exit_code: %d\n", exitErr.ExitCode())
	} else if !ok && err != nil {
		log.Panicln(err)
	}

	log.Println("OK")

}
