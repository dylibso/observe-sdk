package main

import (
	"context"
	"log"
	"os"
	"time"

	observe "github.com/dylibso/observe-sdk/go"
	"github.com/dylibso/observe-sdk/go/adapter/stdout"
	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
)

func main() {
	ctx := context.Background()

	// we only need to create and start once per instance of our host app
	adapter := stdout.NewStdoutAdapter()
	defer adapter.Stop(true)
	adapter.Start(ctx)

	// Load WASM from disk
	wasm, err := os.ReadFile(os.Args[1])
	if err != nil {
		log.Panicln(err)
	}

	cfg := wazero.NewRuntimeConfig().WithCustomSections(true)
	rt := wazero.NewRuntimeWithConfig(ctx, cfg)

	opts := observe.Options{
		ChannelBufferSize: 1024,
		SpanFilter: &observe.SpanFilter{
			MinDuration: 0,
		},
	}

	traceCtx, err := adapter.NewTraceCtx(ctx, rt, wasm, &opts)
	if err != nil {
		log.Panicln(err)
	}
	wasi_snapshot_preview1.MustInstantiate(ctx, rt)

	config := wazero.NewModuleConfig().
		WithStdin(os.Stdin).
		WithStdout(os.Stdout).
		WithStderr(os.Stderr).
		WithArgs(os.Args[1:]...).
		WithStartFunctions("_start")
	m, err := rt.InstantiateWithConfig(ctx, wasm, config)
	if err != nil {
		log.Panicln(err)
	}
	defer m.Close(ctx)

	traceCtx.Finish()
	time.Sleep(2 * time.Second)
}
