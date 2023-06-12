package main

import (
	"log"
	"os"
	"time"

	observe "github.com/dylibso/observe-sdk-wazero"
	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
)

func main() {
	// BEGIN OUR API
	//
	//
	// Collector API
	collector := observe.NewCollector(nil)
	ctx, r, err := collector.InitRuntime()
	if err != nil {
		log.Panicln(err)
	}
	defer r.Close(ctx) // This closes everything this Runtime created.

	//
	// Adapter API
	adapter := observe.StdoutAdapter{}
	adapter.Start(collector)
	defer adapter.Wait(collector, time.Millisecond*5)
	//
	// END OUR API

	// Instantiate WASI
	wasi_snapshot_preview1.MustInstantiate(ctx, r)

	// Load WASM from disk
	wasm, err := os.ReadFile(os.Args[1])
	if err != nil {
		log.Panicln(err)
	}

	// BEGIN OUR API
	//
	// Signal module start
	collector.ModuleBegin("something")
	//
	// END OUR API

	config := wazero.NewModuleConfig().WithStdin(os.Stdin).WithStdout(os.Stdout).WithStderr(os.Stderr).WithArgs(os.Args[2:]...).WithStartFunctions("_start")
	m, err := r.InstantiateWithConfig(ctx, wasm, config)
	if err != nil {
		log.Panicln(err)
	}
	defer m.Close(ctx)
}
