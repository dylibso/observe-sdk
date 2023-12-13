package main

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"

	"github.com/dylibso/observe-sdk/go/adapter/datadog"
	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
)

type Output struct {
	Stdout string `json:"stdout"`
}

type server struct {
	adapter *datadog.DatadogAdapter
}

func main() {
	config := datadog.DefaultDatadogConfig()
	config.ServiceName = "iota"
	config.AgentHost = "http://ddagent:8126"
	config.DefaultTags = make(map[string]string)
	config.DefaultTags["host_language"] = "go"
	dd := datadog.NewDatadogAdapter(config)

	s := server{
		adapter: dd,
	}
	http.HandleFunc("/", index)
	http.HandleFunc("/upload", upload)
	http.HandleFunc("/run", s.runModule)

	log.Println("starting server on :3000")
	http.ListenAndServe(":3000", nil)
}

func index(res http.ResponseWriter, req *http.Request) {
	res.WriteHeader(http.StatusOK)
	res.Header().Set("Content-Type", "application/text")
	res.Write([]byte("Hello, World!\n"))
}

func upload(res http.ResponseWriter, req *http.Request) {
	if req.Method != http.MethodPost {
		res.WriteHeader(http.StatusMethodNotAllowed)
		return
	}

	mpFile, _, err := req.FormFile("wasm")
	if err != nil {
		log.Println("upload error:", err)
		res.WriteHeader(http.StatusBadRequest)
		res.Write([]byte("Bad Request"))
		return
	}

	name := req.URL.Query().Get("name")
	if name == "" {
		log.Println("upload error:", err)
		res.WriteHeader(http.StatusBadRequest)
		res.Write([]byte("Bad Request"))
		return
	}

	path := filepath.Join(os.TempDir(), name)
	tmpFile, err := os.Create(path)
	if err != nil {
		log.Println("file error:", err)
		res.WriteHeader(http.StatusInternalServerError)
		res.Write([]byte("Internal Service Error"))
		return
	}

	n, err := io.Copy(tmpFile, mpFile)
	if err != nil {
		log.Println("copy error:", err)
		res.WriteHeader(http.StatusInternalServerError)
		res.Write([]byte("Internal Service Error"))
		return
	}

	fmt.Printf("Length of `%s` is %d bytes\n", path, n)
	res.WriteHeader(http.StatusOK)
}

func (s *server) runModule(res http.ResponseWriter, req *http.Request) {
	ctx := context.Background()
	if req.Method != http.MethodPost {
		res.WriteHeader(http.StatusMethodNotAllowed)
		return
	}

	name := req.URL.Query().Get("name")
	if name == "" {
		log.Println("name error: no name on url query")
		res.WriteHeader(http.StatusBadRequest)
		res.Write([]byte("Bad Request"))
		return
	}

	// NOTE: The wasm code loaded here will only report any metrics via the adapter _if the code is instrumented_. 
	// If you expect to see telemetry data, please be sure you're running instrumented code. 
	// This section of the docs is a good place to start: 
	// https://dev.dylibso.com/docs/observe/overview#2-instrumenting-your-code-automatic-or-manual
	path := filepath.Join(os.TempDir(), name)
	wasm, err := os.ReadFile(path)
	if err != nil {
		log.Println("name error: no module found", err)
		res.WriteHeader(http.StatusNotFound)
		res.Write([]byte("Not Found"))
		return
	}

	s.adapter.Start(ctx)
	defer s.adapter.Stop(true)

	cfg := wazero.NewRuntimeConfig().WithCustomSections(true)
	rt := wazero.NewRuntimeWithConfig(ctx, cfg)
	traceCtx, err := s.adapter.NewTraceCtx(ctx, rt, wasm, nil)
	if err != nil {
		log.Panicln(err)
	}
	wasi_snapshot_preview1.MustInstantiate(ctx, rt)
	output := &bytes.Buffer{}
	config := wazero.NewModuleConfig().WithStdin(req.Body).WithStdout(output).WithArgs(name)
	defer req.Body.Close()

	mod, err := rt.InstantiateWithConfig(ctx, wasm, config)
	if err != nil {
		log.Println("module instance error:", err)
		res.WriteHeader(http.StatusInternalServerError)
		res.Write([]byte("Internal Service Error"))
		return
	}
	defer mod.Close(ctx)

	resourceName := "iota-go"
	httpUrl := req.URL.String()
	httpStatusCode := 200
	spanKind := datadog.Server
	httpClientIp := req.RemoteAddr
	meta := datadog.DatadogMetadata{
		ResourceName:   &resourceName,
		HttpMethod:     &req.Method,
		HttpUrl:        &httpUrl,
		HttpStatusCode: &httpStatusCode,
		SpanKind:       &spanKind,
		HttpClientIp:   &httpClientIp,
	}
	traceCtx.Metadata(meta)
	traceCtx.Finish()
	log.Println("stopped collector, sent to datadog")

	res.WriteHeader(http.StatusOK)
	res.Header().Add("content-type", "application/json")
	res.Write(output.Bytes())
}
