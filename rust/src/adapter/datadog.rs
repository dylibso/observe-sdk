use anyhow::Result;
use log::warn;
use serde_json::json;
//use std::io::prelude::*;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    //fs::OpenOptions,
    //io,
    //net::UdpSocket,
    time::Duration,
};
use ureq;
use url::Url;

use crate::{Event, Log, Metric, TraceEvent};

pub use super::{
    datadog_formatter::{DatadogFormatter, Span, Trace},
    Adapter, AdapterHandle, AdapterMetadata,
};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum DatadogLanguage {
    Cpp,
    Dotnet,
    Go,
    Jvm,
    Javascript,
    Php,
    Ruby,
    Python,
}

impl Display for DatadogLanguage {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            DatadogLanguage::Cpp => write!(f, "cpp"),
            DatadogLanguage::Dotnet => write!(f, "dotnet"),
            DatadogLanguage::Go => write!(f, "go"),
            DatadogLanguage::Jvm => write!(f, "jvm"),
            DatadogLanguage::Javascript => write!(f, "javascript"),
            DatadogLanguage::Php => write!(f, "php"),
            DatadogLanguage::Python => write!(f, "python"),
            DatadogLanguage::Ruby => write!(f, "ruby"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DatadogMetadata {
    pub resource_name: Option<String>,
    pub http_status_code: Option<u16>,
    pub http_url: Option<String>,
    pub http_method: Option<String>,
    pub http_client_ip: Option<String>,
    pub http_request_content_length: Option<u64>,
    pub http_request_content_length_uncompressed: Option<u64>,
    pub http_response_content_length: Option<u64>,
    pub http_response_content_length_uncompressed: Option<u64>,
    pub span_kind: Option<DatadogSpanKind>,
    pub language: Option<DatadogLanguage>,
    pub component: Option<String>,
}

/// Config options for DatadogAdapter
#[derive(Debug, Clone)]
pub struct DatadogConfig {
    pub agent_host: String,
    pub service_name: String,
    pub default_tags: HashMap<String, String>,
    pub trace_type: DatadogTraceType,
}

impl Default for DatadogConfig {
    fn default() -> Self {
        Self {
            agent_host: "http://localhost:8126".into(),
            service_name: "default-wasm-service".into(),
            default_tags: HashMap::new(),
            trace_type: DatadogTraceType::Web,
        }
    }
}

struct DatadogEvents {
    pub spans: Vec<Span>,
    pub metrics: Vec<Metric>,
    pub logs: Vec<Log>,
}

/// An adapter to send events from your module to a [Datadog Agent](https://docs.datadoghq.com/agent/).
pub struct DatadogAdapter {
    trace_events: Vec<TraceEvent>,
    config: DatadogConfig,
    // TODO add bucketing / flushing implementation
    //delay: Delay,
    //handle: AdapterHandle,
    //triggered_flush: bool,
}

impl Adapter for DatadogAdapter {
    fn handle_trace_event(&mut self, trace_evt: TraceEvent) -> Result<()> {
        self.trace_events.push(trace_evt);
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
            trace_events: vec![],
        }
    }

    /// Creates the Datadog adapter and spawns a task for it.
    /// This should ideally be created once per process of
    /// your rust application.
    pub fn create(config: DatadogConfig) -> AdapterHandle {
        let adapter = Self::new(config);
        Self::spawn(adapter)
    }

    fn event_to_spans(
        &self,
        ddevents: &mut DatadogEvents,
        event: Event,
        parent_id: Option<u64>,
        trace_id: u64,
    ) -> Result<()> {
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

                ddevents.spans.push(span);

                for e in f.within.iter() {
                    self.event_to_spans(ddevents, e.to_owned(), span_id, trace_id)?
                }
            }
            Event::Alloc(a) => {
                // TODO i seem to be losing this value
                if let Some(span) = ddevents.spans.last_mut() {
                    span.add_allocation(a.amount);
                }
            }
            Event::Metric(mut m) => {
                m.trace_id = Some(trace_id);
                ddevents.metrics.push(m);
            }
            Event::Tags(t) => {
                if let Some(span) = ddevents.spans.last_mut() {
                    for tag in t.tags {
                        span.add_tag(tag);
                    }
                }
            }
            Event::Log(l) => {
                ddevents.logs.push(l);
            }
            ev => {
                log::error!("Unknown event {:#?}", ev);
            }
        }
        Ok(())
    }

    fn dump_traces(&mut self) -> Result<()> {
        let mut dtf = DatadogFormatter::new();
        let mut log_events = vec![];
        let mut metric_events = vec![];

        for trace_evt in &self.trace_events {
            let trace_id = &trace_evt.telemetry_id;
            let mut ddevents = DatadogEvents {
                spans: vec![],
                logs: vec![],
                metrics: vec![],
            };
            for span in &trace_evt.events {
                let tid = trace_id.clone().into();
                self.event_to_spans(&mut ddevents, span.to_owned(), None, tid)?;
            }

            metric_events.extend(ddevents.metrics);
            log_events.extend(ddevents.logs);

            let mut trace = Trace::new();
            let mut first_span = true;
            for span in ddevents.spans {
                let mut span = span.clone();
                if first_span {
                    let mut dd_meta = self.config.default_tags.clone();
                    span.resource = "request".into();
                    span.typ = Some(DatadogTraceType::Web.to_string());

                    if let Some(AdapterMetadata::Datadog(meta)) = trace_evt.metadata.to_owned() {
                        if let Some(resource) = meta.resource_name {
                            span.resource = resource;
                        }
                        if let Some(status) = meta.http_status_code {
                            dd_meta.insert("http.status_code".to_string(), status.to_string());
                        }
                        if let Some(method) = meta.http_method {
                            dd_meta.insert("http.method".to_string(), method.to_string());
                        }
                        if let Some(url) = meta.http_url {
                            dd_meta.insert("http.url".to_string(), url.to_string());
                        }
                        if let Some(ip) = meta.http_client_ip {
                            dd_meta.insert("http.client_ip".to_string(), ip.to_string());
                        }
                        if let Some(rl) = meta.http_request_content_length {
                            dd_meta
                                .insert("http.request.content_length".to_string(), rl.to_string());
                        }
                        if let Some(rl) = meta.http_request_content_length_uncompressed {
                            dd_meta.insert(
                                "http.request.content_length_uncompressed".to_string(),
                                rl.to_string(),
                            );
                        }
                        if let Some(rl) = meta.http_response_content_length {
                            dd_meta
                                .insert("http.response.content_length".to_string(), rl.to_string());
                        }
                        if let Some(rl) = meta.http_response_content_length_uncompressed {
                            dd_meta.insert(
                                "http.response.content_length_uncompressed".to_string(),
                                rl.to_string(),
                            );
                        }
                        if let Some(span_kind) = meta.span_kind {
                            dd_meta.insert("span.kind".to_string(), span_kind.to_string());
                        }
                    }

                    span.meta.extend(dd_meta);
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
            self.trace_events.clear();
        }

        // if metric_events.len() > 0 {
        //     let socket = UdpSocket::bind("0.0.0.0:9999").unwrap();
        //     for stat in metric_events {
        //         // TODO we are just assumign they are statsd format but we can check stat.format
        //         let message = format!("{}|#trace_id:{}", stat.message, stat.trace_id.unwrap());
        //         // TODO get the host from config
        //         socket
        //             .send_to(message.as_bytes(), "127.0.0.1:8125")
        //             .unwrap();
        //     }
        // }

        // TODO find a better way to get logs to the agent

        // if log_events.len() > 0 {
        //     let file = OpenOptions::new()
        //         .write(true)
        //         .append(true)
        //         .create(true)
        //         .open("/tmp/planktonic.log")?;

        //     let mut file_writer = io::BufWriter::new(file);
        //     for evt in log_events {
        //         dbg!(&evt);
        //         file_writer.write_all(evt.message.as_bytes()).unwrap();
        //     }
        //     file_writer.flush()?;
        // }

        Ok(())
    }
}
