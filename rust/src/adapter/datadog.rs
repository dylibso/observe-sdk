use anyhow::Result;
use log::warn;
use crate::{
    adapter::datadog_formatter::DatadogFormatter,
    Event, Metadata, add_to_linker,
};
use std::{
    fmt::{Display, Formatter},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    }, collections::HashMap
};
use serde_json::json;
use tokio::sync::Mutex;
use ureq;
use url::Url;
use wasmtime::Linker;

use super::{
    datadog_formatter::{self, Trace, Span},
    Adapter, Collector,
};

pub struct DatadogAdapterContainer(Arc<Mutex<DatadogAdapter>>);

impl DatadogAdapterContainer {
    pub async fn start<T: 'static>(&self, linker: &mut Linker<T>) -> Result<DatadogTraceCtx> {
        let id = next_id();
        let events = add_to_linker(id, linker)?;
        let collector = Collector::new(self.0.clone(), id, events).await?;

        Ok(DatadogTraceCtx(collector))
    }
}

pub struct DatadogTraceCtx(Collector);

impl DatadogTraceCtx {
    pub async fn set_trace_id(&mut self, id: u64) {
        self.0.set_metadata("trace_id".to_string(), id).await;
    }

    pub async fn shutdown(&self) {
        self.0.shutdown().await;
    }
}

#[derive(Clone)]
pub struct DatadogAdapter {
    pub trace_id: u64,
    pub spans: Vec<Span>,
    pub config: DatadogConfig,
}

#[derive(Clone)]
pub enum DatadogTraceType {
    Web,
    Db,
    Cache,
    Custom,
}

impl Display for DatadogTraceType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            DatadogTraceType::Web => write!(f, "web"),
            DatadogTraceType::Db => write!(f, "db"),
            DatadogTraceType::Cache => write!(f, "cache"),
            DatadogTraceType::Custom => write!(f, "custom"),
        }
    }
}

pub enum DatadogSpanKind {
    Server,
    Client,
    Producer,
    Consumer,
    Internal,
}

impl Display for DatadogSpanKind {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            DatadogSpanKind::Server => write!(f, "server"),
            DatadogSpanKind::Client => write!(f, "client"),
            DatadogSpanKind::Producer => write!(f, "producer"),
            DatadogSpanKind::Consumer => write!(f, "consumer"),
            DatadogSpanKind::Internal => write!(f, "internal"),
        }
    }
}

#[derive(Clone)]
pub struct DatadogConfig {
    pub agent_host: String,
    pub service_name: String,
    pub default_tags: HashMap<String, String>,
    pub trace_type: DatadogTraceType,
}

impl DatadogConfig {
    pub fn new() -> DatadogConfig {
        DatadogConfig {
            agent_host: "http://localhost:8126".into(),
            service_name: "my-wasm-service".into(),
            default_tags: HashMap::new(),
            trace_type: DatadogTraceType::Web,
        }
    }
}

impl DatadogAdapter {
    pub fn new(config: DatadogConfig) -> DatadogAdapterContainer {
        let adapter = DatadogAdapter {
            trace_id: datadog_formatter::new_span_id(),
            spans: vec![],
            config,
        };

        DatadogAdapterContainer(Arc::new(Mutex::new(adapter)))
    }

    pub fn new_collector(&mut self) -> usize {
        next_id()
    }

    fn _handle_event(&mut self, event: Event, parent_id: Option<u64>) -> Result<Option<Vec<Span>>> {
        match event {
            Event::Func(_id, f) => {
                let function_name = &f.name.clone().unwrap_or("unknown-name".to_string());
                let name = format!("{}", &function_name);

                let config = self.config.clone();
                let span =
                    Span::new(config, self.trace_id.clone(), parent_id, name, f.start, f.end)?;

                let span_id = Some(span.span_id.clone());
                let mut spans = vec![span];

                for e in f.within.iter() {
                    if let Some(mut s) = self._handle_event(e.to_owned(), span_id.clone())? {
                        spans.append(&mut s);
                    };
                }

                Ok(Some(spans))
            }
            Event::Alloc(_id, a) => {
                // TODO i seem to be losing this value
                if let Some(span) = self.spans.last_mut() {
                    span.add_allocation(a.amount);
                }
                Ok(None)
            }
            Event::Metadata(_id, Metadata { key, value }) => {
                if key == "trace_id" {
                    self.trace_id = value;
                }
                Ok(None)
            }
            Event::Shutdown(_id) => {
                // when we receive the shutdown
                // then dump the trace to the agent
                self.shutdown()?;
                self.spans.clear();
                Ok(None)
            },
        }
    }

    pub async fn set_trace_id(collector: &Collector, trace_id: u64) {
        collector
            .set_metadata("trace_id".to_string(), trace_id)
            .await;
    }
}

impl Adapter for DatadogAdapter {
    // flush any remaining spans to Datadog Agent
    fn shutdown(&self) -> Result<()> {
        let mut dtf = DatadogFormatter::new();
        let mut trace = Trace::new();
        let mut first_span = true;
        for span in &self.spans {
            let mut span = span.clone();
            if first_span {
                let mut meta = self.config.default_tags.clone();
                meta.insert("http.status_code".to_string(), "200".to_string());
                meta.insert("http.method".to_string(), "POST".to_string());
                meta.insert("http.url".to_string(), "http://localhost:3000".to_string());
                meta.insert("span.kind".to_string(), DatadogSpanKind::Internal.to_string());
                // TODO don't throw away what be be some existing meta here
                span.meta = meta;
                span.resource = "request".into();
                span.typ = Some(self.config.trace_type.to_string());
                first_span = false;
            }
            trace.push(span);
        }
        dtf.traces.push(trace);

        let host = Url::parse(&self.config.agent_host)?;
        let url = host.join("/v0.3/traces")?.to_string();
        let j = json!(&dtf.traces);
        let body = serde_json::to_string(&j)?;
        println!("{}", body);

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body);

        if !response.is_ok() {
            warn!("Request to datadog agent failed with status: {:#?}", response);
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        match self._handle_event(event, None) {
            Ok(result) => {
                if let Some(spans) = result {
                    for span in spans {
                        self.spans.push(span);
                    }
                }
            }
            Err(e) => warn!("{}", e),
        } 
    }
}

fn next_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
