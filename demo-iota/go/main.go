package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"github.com/dylibso/observe-sdk/go"
	"github.com/dylibso/observe-sdk/go/adapter/datadog"
	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
)

type Output struct {
	Stdout string `json:"stdout"`
}

type server struct {
	adapter datadog.DatadogAdapter
}

func main() {
	//
	// Adapter API
	config := datadog.DefaultDatadogConfig()
	config.ServiceName = "iota-go"
	config.AgentHost = "http://ddagent:8126"
	dd, err := datadog.NewDatadogAdapter(config)
	if err != nil {
		log.Panicln(err)
	}

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

	path := filepath.Join(os.TempDir(), name)
	wasm, err := ioutil.ReadFile(path)
	if err != nil {
		log.Println("name error: no module found", err)
		res.WriteHeader(http.StatusNotFound)
		res.Write([]byte("Not Found"))
		return
	}

	//
	// Collector API
	collector := observe.NewCollector(nil)
	ctx, r, err := collector.InitRuntime()
	if err != nil {
		log.Panicln(err)
	}
	defer r.Close(ctx) // This closes everything this Runtime created.

	wasi_snapshot_preview1.MustInstantiate(ctx, r)
	output := &bytes.Buffer{}
	config := wazero.NewModuleConfig().WithStdin(req.Body).WithStdout(output)
	defer req.Body.Close()

	s.adapter.Start(collector, wasm)
	mod, err := r.InstantiateWithConfig(ctx, wasm, config)
	if err != nil {
		log.Println("module instance error:", err)
		res.WriteHeader(http.StatusInternalServerError)
		res.Write([]byte("Internal Service Error"))
		return
	}
	s.adapter.Stop(collector)
	log.Println("stopped collector, sent to datadog")
	defer mod.Close(ctx)

	data, err := json.Marshal(Output{Stdout: strings.Trim(output.String(), "\n")})
	if err != nil {
		log.Println("json encode error:", err)
		res.WriteHeader(http.StatusInternalServerError)
		res.Write([]byte("Internal Service Error"))
		return
	}
	res.WriteHeader(http.StatusOK)
	res.Header().Add("content-type", "application/json")
	res.Write(data)
}
