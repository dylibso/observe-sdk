use std::io::prelude::*;
use std::{collections::HashMap, fs::File};

use axum::{
    extract::Multipart,
    extract::Query,
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use dylibso_observe_sdk::adapter::{
    datadog::{DatadogAdapter, DatadogConfig},
    AdapterHandle,
};
use serde::Deserialize;
use serde_json::{json, Value};
use wasi_common::{pipe::ReadPipe, pipe::WritePipe};
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

#[derive(Deserialize)]
struct ModuleParams {
    name: String,
}

#[tokio::main]
async fn main() {
    // configure the DataDog adapter to share with runtime instances
    let mut default_tags = HashMap::new();
    default_tags.insert("host_language".to_string(), "rust".to_string());
    let ddconfig = DatadogConfig {
        agent_host: "http://ddagent:8126".into(),
        service_name: "iota".into(),
        default_tags,
        ..Default::default()
    };
    let adapter = DatadogAdapter::create(ddconfig);

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/run", post(run_module))
        .route("/upload", post(upload))
        .layer(axum::extract::DefaultBodyLimit::max(1024 * 1024 * 10))
        .with_state(adapter);

    println!("Binding to 0.0.0.0:3000");

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn run_module(
    State(state): State<AdapterHandle>,
    params: Query<ModuleParams>,
    body: String,
) -> Json<Value> {
    let query: ModuleParams = params.0;
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();
    let data = std::fs::read(format!("/tmp/{}", query.name)).unwrap();
    let module = wasmtime::Module::new(&engine, &data).unwrap();

    let stdin = ReadPipe::from(body);
    let stdout = WritePipe::new_in_memory();

    let wasi = WasiCtxBuilder::new()
        .stdin(Box::new(stdin.clone()))
        .stdout(Box::new(stdout.clone()))
        .build();

    let mut store = wasmtime::Store::new(&engine, wasi);
    let mut linker = wasmtime::Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |wasi| wasi).unwrap();

    let adapter = state.clone();
    let trace_ctx = adapter.start(&mut linker, &data).unwrap();

    let instance = linker.instantiate(&mut store, &module).unwrap();

    let f = instance
        .get_func(&mut store, "_start")
        .expect("function exists");
    f.call(&mut store, &[], &mut []).unwrap();

    trace_ctx.shutdown().await;

    drop(store);

    let contents: Vec<u8> = stdout
        .try_into_inner()
        .map_err(|_err| anyhow::Error::msg("sole remaining reference"))
        .unwrap()
        .into_inner();

    Json(json!({
        "stdout": std::str::from_utf8(&contents).unwrap().trim()
    }))
}

async fn upload(params: Query<ModuleParams>, mut multipart: Multipart) {
    let query: ModuleParams = params.0;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        if name != "wasm" {
            continue;
        }
        let data = field.bytes().await.unwrap();

        let filepath = format!("/tmp/{}", &query.name);
        let mut file = File::create(&filepath).unwrap();
        file.write_all(&data).unwrap();

        println!("Storing `{}`. Length is {} bytes", filepath, data.len());
    }
}
