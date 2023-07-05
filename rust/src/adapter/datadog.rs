use anyhow::Result;
use log::warn;
use serde_json::json;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    time::Duration,
};
use ureq;
use url::Url;

use crate::{Event, TraceEvent};

use super::{
    datadog_formatter::{Span, Trace, DatadogFormatter}, AdapterHandle, Adapter
};

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

/// Config options for DatadogAdapter
#[derive(Clone)]
pub struct DatadogConfig {
    pub agent_host: String,
    pub service_name: String,
    pub default_tags: HashMap<String, String>,
    pub trace_type: DatadogTraceType,
}

impl DatadogConfig {
    pub fn new() -> DatadogConfig {
        Default::default()
    }
}

impl Default for DatadogConfig {
    fn default() -> Self {
        DatadogConfig {
            agent_host: "http://localhost:8126".into(),
            service_name: "my-wasm-service".into(),
            default_tags: HashMap::new(),
            trace_type: DatadogTraceType::Web,
        }
    }
}

/// An adapter to send events from your module to a [Datadog Agent](https://docs.datadoghq.com/agent/).
pub struct DatadogAdapter {
    traces: Vec<Vec<Span>>,
    config: DatadogConfig,
    // TODO add bucketing / flushing implementation
    //delay: Delay,
    //handle: AdapterHandle,
    //triggered_flush: bool,
}

impl Adapter for DatadogAdapter {
    fn handle_trace_event(&mut self, trace_evt: TraceEvent) -> Result<()> {
        let mut spans = vec![];
        let trace_id = trace_evt.telemetry_id.into();
        for span in trace_evt.events {
            self.event_to_spans(&mut spans, span, None, trace_id)?;
        }
        self.traces.push(spans);
        // TODO add flush logic here instead of dumping
        // we should check if a flush is triggered, if not 
        // then we should kick off a flush at some point in the future
        self.dump_traces()?;

        Ok(())
    }
}

impl DatadogAdapter {
    fn new(config: DatadogConfig) -> Self {
        Self {
            config,
            traces: vec![]
        }
    }

    /// Creates the Datadog adapter and spawns a task for it.
    /// This should ideally be created once per process of
    /// your rust application.
    pub fn create(config: DatadogConfig) -> AdapterHandle {
        let adapter = Self::new(config);
        Self::spawn(adapter)
    }

    fn event_to_spans(&self, spans: &mut Vec<Span>, event: Event, parent_id: Option<u64>, trace_id: u64) -> Result<()> {
        match event {
            Event::Func(f) => {
                let function_name = &f.name.clone().unwrap_or("unknown-name".to_string());
                let config = self.config.clone();
                let span = Span::new(
                    config,
                    trace_id,
                    parent_id,
                    function_name.to_string(),
                    f.start,
                    f.end,
                )?;

                let span_id = Some(span.span_id);

                spans.push(span);

                for e in f.within.iter() {
                    self.event_to_spans(spans, e.to_owned(), span_id, trace_id)?
                }
            }
            Event::Alloc(a) => {
                // TODO i seem to be losing this value
                if let Some(span) = spans.last_mut() {
                    span.add_allocation(a.amount);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn dump_traces(&mut self) -> Result<()> {
        let mut dtf = DatadogFormatter::new();
        for trace_spans in &self.traces {
            let mut trace = Trace::new();
            let mut first_span = true;
            for span in trace_spans {
                let mut span = span.clone();
                if first_span {
                    // TODO for the moment i'm going to comment this stuff out until
                    // we come up with a nice API to let the programmer passs it in.
                    // maybe it can come in via shutdown or something?
                    //
                    // let mut meta = self.config.default_tags.clone();
                    // meta.insert("http.status_code".to_string(), "200".to_string());
                    // meta.insert("http.method".to_string(), "POST".to_string());
                    // meta.insert("http.url".to_string(), "http://localhost:3000".to_string());
                    // meta.insert("span.kind".to_string(), DatadogSpanKind::Internal.to_string());
                    // // TODO don't throw away what be be some existing meta here
                    // span.meta = meta;

                    // TODO this value should come from programmer
                    span.resource = "request".into();
                    span.typ = Some(DatadogTraceType::Web.to_string());
                    first_span = false;
                }
                trace.push(span);
            }
            dtf.traces.push(trace);
        }

        let host = Url::parse(&self.config.agent_host)?;
        let url = host.join("/v0.3/traces")?.to_string();
        let j = json!(&dtf.traces);
        let body = serde_json::to_string(&j)?;

        let response = ureq::post(&url)
            .timeout(Duration::from_secs(1))
            .set("Content-Type", "application/json")
            .send_string(&body);

        if response.is_err() {
            warn!(
                "Request to datadog agent failed with status: {:#?}",
                response
            );
        } else {
            // clear the traces because we've successfully submitted them
            self.traces.clear();
        }

        Ok(())
    }
}
