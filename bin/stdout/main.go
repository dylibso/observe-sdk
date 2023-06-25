package main

import (
	"log"
	"os"
	"time"

	observe "github.com/dylibso/observe-sdk-wazero"
	"github.com/dylibso/observe-sdk-wazero/adapter/stdout"
	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
)

func main() {
	//
	// Collector API
	collector := observe.NewCollector(nil)
	ctx, r, err := collector.InitRuntime()
	if err != nil {
		log.Panicln(err)
	}
	defer r.Close(ctx) // This closes everything this Runtime created.

	// Load WASM from disk
	wasm, err := os.ReadFile(os.Args[1])
	if err != nil {
		log.Panicln(err)
	}

	// Wazero stuff
	// Instantiate WASI
	wasi_snapshot_preview1.MustInstantiate(ctx, r)
	// collector.CustomEvent("Start module", map[string]interface{}{"name": "testing"})

	//
	// Adapter API
	adapter, err := stdout.NewStdoutAdapter(wasm)
	if err != nil {
		log.Panicln(err)
	}
	adapter.Start(collector)
	defer adapter.Wait(collector, time.Millisecond)

	config := wazero.NewModuleConfig().
		WithStdin(os.Stdin).
		WithStdout(os.Stdout).
		WithStderr(os.Stderr).
		WithArgs(os.Args[1:]...).
		WithStartFunctions("_start")
	m, err := r.InstantiateWithConfig(ctx, wasm, config)
	if err != nil {
		log.Panicln(err)
	}
	defer m.Close(ctx)
}
