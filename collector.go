package dylibso_observe

import (
	"context"
	"log"
	"time"

	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/experimental"
)

type startedFunction struct {
	event     RawEvent
	startTime time.Time
}

type Collector struct {
	raw              chan RawEvent
	startedFunctions []startedFunction
	Events           chan Event
	Config           *Config
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

func (c *Collector) pushFunction(ev RawEvent) {
	c.startedFunctions = append(c.startedFunctions, startedFunction{
		startTime: time.Now(),
		event:     ev,
	})
}

func (c *Collector) popFunction() (bool, RawEvent, time.Time) {
	if len(c.startedFunctions) == 0 {
		return false, RawEvent{}, time.Time{}
	}

	event := c.startedFunctions[len(c.startedFunctions)-1]
	c.startedFunctions = c.startedFunctions[:len(c.startedFunctions)-1]
	// funcs := make([]startedFunction, len(c.startedFunctions)-1)
	// for i := range funcs {
	// 	funcs[i] = c.startedFunctions[i]
	// }
	// c.startedFunctions = funcs

	return true, event.event, event.startTime
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

func NewCollector(config *Config) Collector {
	if config == nil {
		config = NewDefaultConfig()
	}
	return Collector{
		raw:    make(chan RawEvent, config.ChannelBufferSize),
		Events: make(chan Event, config.ChannelBufferSize),
		Config: config,
	}
}

func (c *Collector) InitRuntime() (context.Context, wazero.Runtime, error) {
	ctx := context.WithValue(context.Background(), experimental.FunctionListenerFactoryKey{}, *c)
	r := wazero.NewRuntimeWithConfig(ctx, c.Config.RuntimeConfig)
	observe := r.NewHostModuleBuilder("dylibso_observe")
	functions := observe.NewFunctionBuilder()
	functions.WithFunc(func(ctx context.Context) {
		ev := <-c.raw
		if ev.Kind != RawEnter {
			log.Panicln("Expected event", RawEnter, "but got", ev.Kind)
		}
		c.pushFunction(ev)
	}).Export("instrument_enter")
	functions.WithFunc(func(ctx context.Context) {
		ev := <-c.raw
		if ev.Kind != RawExit {
			log.Panicln("Expected event", RawExit, "but got", ev.Kind)
		}
		ok, start, startTime := c.popFunction()
		if !ok {
			log.Println("Expected values on started function stack, but none were found")
			return
		}
		if ev.FunctionIndex != start.FunctionIndex {
			log.Panicln("Expected call to", ev.FunctionName, "but found call to", start.FunctionName)
		}
		event := CallEvent{
			Raw:      []RawEvent{start, ev},
			Duration: time.Now().Sub(startTime),
		}
		c.Events <- event
	}).Export("instrument_exit")
	functions.WithFunc(func(ctx context.Context, amt int32) {
		ev := <-c.raw
		if ev.Kind != RawMemoryGrow {
			log.Panicln("Expected event", MemoryGrow, "but got", ev.Kind)
		}
		event := MemoryGrowEvent{
			Raw: ev,
		}
		c.Events <- event
	}).Export("instrument_memory_grow")
	_, err := observe.Instantiate(ctx)
	if err != nil {
		return nil, nil, err
	}
	return ctx, r, nil
}

func (c *Collector) ModuleBegin(name string) {
	c.Events <- ModuleBeginEvent{
		Name: name,
	}
}

func Init(config *Config) (context.Context, wazero.Runtime, Collector, error) {
	c := NewCollector(config)
	ctx, r, err := c.InitRuntime()
	if err != nil {
		return nil, nil, Collector{}, err
	}

	return ctx, r, c, nil
}
