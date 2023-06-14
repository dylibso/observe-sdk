use crate::{
    adapter::datadog_formatter::DatadogFormatter,
    Event, Metadata,
};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    }, collections::HashMap, str::from_utf8
};
use serde_json::json;
use tokio::sync::Mutex;
use ureq;
use std::net::UdpSocket;
use url::Url;

use super::{
    datadog_formatter::{self, Trace, Span},
    Adapter, Collector,
};

pub type DatadogAdapterContainer = Arc<Mutex<DatadogAdapter>>;

#[derive(Clone)]
pub struct DatadogAdapter {
    pub trace_id: u64,
    pub spans: Vec<Span>,
    pub stats: Vec<Vec<u8>>,
    pub config: DatadogConfig,
}

#[derive(Clone)]
pub struct DatadogConfig {
    pub agent_host: String,
    pub service_name: String,
    pub default_tags: HashMap<String, String>,
    pub trace_type: String,
}

impl DatadogConfig {
    pub fn new() -> DatadogConfig {
        DatadogConfig {
            agent_host: "http://localhost:8126".into(),
            service_name: "my-wasm-service".into(),
            default_tags: HashMap::new(),
            trace_type: "web".into(),
        }
    }
}

impl DatadogAdapter {
    pub fn new(config: DatadogConfig) -> DatadogAdapterContainer {
        let adapter = DatadogAdapter {
            trace_id: datadog_formatter::new_span_id(),
            spans: vec![],
            stats: vec![],
            config,
        };

        Arc::new(Mutex::new(adapter))
    }

    pub fn new_collector(&mut self) -> usize {
        next_id()
    }

    fn _handle_event(&mut self, event: Event, parent_id: Option<u64>) -> Option<Vec<Span>> {
        match event {
            Event::Func(_id, f) => {
                let function_name = &f.name.clone().unwrap_or("unknown-name".to_string());
                let name = format!("function-call-{}", &function_name);

                let config = self.config.clone();
                let span =
                    Span::new(config, self.trace_id.clone(), parent_id, name, f.start, f.end);

                let span_id = Some(span.span_id.clone());
                let mut spans = vec![span];

                for e in f.within.iter() {
                    if let Some(mut s) = self._handle_event(e.to_owned(), span_id.clone()) {
                        spans.append(&mut s);
                    };
                }

                Some(spans)
            }
            Event::Alloc(_id, a) => {
                let config = self.config.clone();
                let span = Span::new(
                    config,
                    self.trace_id,
                    parent_id,
                    "allocation".to_string(),
                    a.ts,
                    a.ts,
                );
                Some(vec![span])
            }
            Event::Metadata(_id, Metadata { key, value }) => {
                if key == "trace_id" {
                    self.trace_id = value;
                }
                None
            }
            Event::Stats(stat) => {
                self.stats.push(stat);
                None
            }
            Event::Shutdown(_id) => {
                // when we receive the shutdown
                // then dump the trace to the agent
                self.shutdown();
                self.spans.clear();
                self.stats.clear();
                None
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
    // flush any remaning spans
    fn shutdown(&self) {
        let mut dtf = DatadogFormatter::new();
        let mut trace = Trace::new();
        for span in &self.spans {
            trace.push(span.clone());
        }
        dtf.traces.push(trace);

        let host = Url::parse(&self.config.agent_host).unwrap();
        let url = host.join("/v0.3/traces").unwrap().to_string();
        let j = json!(&dtf.traces);
        let body = serde_json::to_string(&j).unwrap();

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body);

        // Check the response status
        if response.is_ok() {
            println!("Request was successful!");
        } else {
            println!("Request failed with status: {:#?}", response);
        }

        let socket = UdpSocket::bind("127.0.0.1:8126").unwrap();
        socket.connect("127.0.0.1:8125").unwrap();
        for stat in &self.stats {
            socket.send(&stat).unwrap();
            println!("Sending stat: {}", from_utf8(&stat).unwrap());
        }

    }

    fn handle_event(&mut self, event: Event) {
        if let Some(spans) = self._handle_event(event, None) {
            for span in spans {
                self.spans.push(span);
            }
        };
    }
}

fn next_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
