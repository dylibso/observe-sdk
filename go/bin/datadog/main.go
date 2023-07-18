package main

import (
  "context"
  "log"
  "os"
  "time"

  "github.com/dylibso/observe-sdk/go/adapter/datadog"
  "github.com/tetratelabs/wazero"
  "github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
)

func main() {
  ctx := context.Background()

  log.Println("Starting adapter")

  // we only need to create and start once per instance of our host app
  ddconf := datadog.DefaultDatadogConfig()
  adapter, err := datadog.NewDatadogAdapter(ddconf)
  adapter.Start()
  defer adapter.Stop()

  log.Println("Adapter started")

  // Load WASM from disk
  wasm, err := os.ReadFile(os.Args[1])
  if err != nil {
    log.Panicln(err)
  }

  log.Println("Create trace ctx")
  traceCtx, err := adapter.NewTraceCtx(wasm, nil)
  if err != nil {
    log.Panicln(err)
  }
  log.Println("trace ctx created")
  cfg := wazero.NewRuntimeConfig().WithCustomSections(true)
  rt := wazero.NewRuntimeWithConfig(ctx, cfg)
  err = traceCtx.Init(ctx, rt)
  if err != nil {
    log.Panicln(err)
  }
  wasi_snapshot_preview1.MustInstantiate(ctx, rt)
  log.Println("wasi inited")

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
  log.Println("module run")
  defer m.Close(ctx)

  traceCtx.Finish()
  log.Println("trace ctx finish")

  time.Sleep(time.Second * 2)
}
