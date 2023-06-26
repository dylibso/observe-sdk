package dylibso_observe

import (
	"context"
	"errors"
	"log"
	"time"

	"github.com/tetratelabs/wabin/binary"
	"github.com/tetratelabs/wabin/wasm"
	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/api"
	"github.com/tetratelabs/wazero/experimental"
)

type Collector struct {
	raw    chan RawEvent
	stack  []CallEvent
	Events chan Event
	Config *Config
	names  map[uint32]string
}

func (c *Collector) Names() map[uint32]string {
	return c.names
}

func (c *Collector) GetNames(data []byte) error {
	features := wasm.CoreFeaturesV2
	m, err := binary.DecodeModule(data, features)
	if err != nil {
		return err
	}

	// Check for version globals
	if err := checkVersion(m); err != nil {
		return err
	}

	if m.NameSection == nil {
		return errors.New("Name section not found")
	}

	c.names = make(map[uint32]string, len(m.NameSection.FunctionNames))

	for _, v := range m.NameSection.FunctionNames {
		c.names[v.Index] = v.Name
	}

	return nil
}

func (c *Collector) clearEvents() {
	for {
		select {
		case <-c.Events:
			continue
		default:
			break
		}
	}
}

func (c *Collector) pushFunction(ev CallEvent) {
	c.stack = append(c.stack, ev)
}

func (c *Collector) popFunction() (CallEvent, bool) {
	if len(c.stack) == 0 {
		return CallEvent{}, false
	}

	event := c.stack[len(c.stack)-1]
	c.stack = c.stack[:len(c.stack)-1]

	return event, true
}

func (c *Collector) peekFunction() (CallEvent, bool) {
	if len(c.stack) == 0 {
		return CallEvent{}, false
	}

	return c.stack[len(c.stack)-1], true
}

type Config struct {
	ChannelBufferSize int
	RuntimeConfig     wazero.RuntimeConfig
}

func NewDefaultConfig() *Config {
	return &Config{
		ChannelBufferSize: 1024,
		RuntimeConfig:     wazero.NewRuntimeConfig(),
	}
}

func NewCollector(config *Config) *Collector {
	if config == nil {
		config = NewDefaultConfig()
	}
	return &Collector{
		raw:    make(chan RawEvent, config.ChannelBufferSize),
		Events: make(chan Event, config.ChannelBufferSize),
		Config: config,
		names:  nil,
	}
}

// TODO: consider a different initial entrypoint to create the runtime using an provided config and context:
// func (c *Collector) NewRuntimeWithConfig(ctx context.Context, config wazero.RuntimeConfig)

func (c *Collector) InitRuntime() (context.Context, wazero.Runtime, error) {
	ctx := context.WithValue(context.Background(), experimental.FunctionListenerFactoryKey{}, c)
	r := wazero.NewRuntimeWithConfig(ctx, c.Config.RuntimeConfig.WithCustomSections(true))
	observe := r.NewHostModuleBuilder("dylibso_observe")
	functions := observe.NewFunctionBuilder()

	functions.WithFunc(func(ctx context.Context, m api.Module, i int32) {
		start := time.Now()
		ev := <-c.raw
		if ev.Kind != RawEnter {
			log.Println("Expected event", RawEnter, "but got", ev.Kind)
		}
		c.pushFunction(CallEvent{Raw: []RawEvent{ev}, Time: start})
	}).Export("instrument_enter")

	functions.WithFunc(func(ctx context.Context, i int32) {
		end := time.Now()
		ev := <-c.raw
		if ev.Kind != RawExit {
			log.Println("Expected event", RawExit, "but got", ev.Kind)
			return
		}
		fn, ok := c.peekFunction()
		if !ok {
			log.Println("Expected values on started function stack, but none were found")
			return
		}
		if ev.FunctionIndex != fn.FunctionIndex() {
			log.Println("Expected call to", ev.FunctionIndex, "but found call to", fn.FunctionIndex())
			return
		}

		fn, _ = c.popFunction()
		fn.Stop(end)
		fn.Raw = append(fn.Raw, ev)

		f, ok := c.popFunction()
		if !ok {
			c.Events <- fn
			return
		}

		f.within = append(f.within, fn)
		c.pushFunction(f)
	}).Export("instrument_exit")

	functions.WithFunc(func(ctx context.Context, amt int32) {
		ev := <-c.raw
		if ev.Kind != RawMemoryGrow {
			log.Println("Expected event", MemoryGrow, "but got", ev.Kind)
			return
		}

		if len(c.stack) > 0 {
			f := c.stack[len(c.stack)-1]
			ev.FunctionIndex = f.FunctionIndex()
			ev.FunctionName = f.FunctionName()
		}

		event := MemoryGrowEvent{
			Raw:  ev,
			Time: time.Now(),
		}

		c.Events <- event
	}).Export("instrument_memory_grow")

	_, err := observe.Instantiate(ctx)
	if err != nil {
		return nil, nil, err
	}
	return ctx, r, nil
}

func (c *Collector) CustomEvent(name string, metadata map[string]interface{}) {
	ev := NewCustomEvent(name)
	ev.Metadata = metadata
	c.Events <- ev
}

func Init(config *Config) (context.Context, wazero.Runtime, *Collector, error) {
	c := NewCollector(config)
	ctx, r, err := c.InitRuntime()
	if err != nil {
		return nil, nil, nil, err
	}

	return ctx, r, c, nil
}
