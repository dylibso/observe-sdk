use anyhow::Result;
use std::{time::SystemTime, collections::HashMap};

use rand::Rng;
use serde::Serialize;

use super::datadog::DatadogConfig;

pub struct DatadogFormatter {
    pub traces: Vec<Trace>
}

pub type Trace = Vec<Span>;

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Span {
    pub trace_id: u64,
    pub span_id: u64,
    pub parent_id: u64,
    pub name: String,
    pub start: u64,
    pub duration: u64,
    pub resource: String,
    pub error: u8,
    pub meta: HashMap<String, String>,
    pub metrics: HashMap<String, f64>,
    pub service: String,
    #[serde(rename = "type")]
    pub typ: String,
}

impl DatadogFormatter {
    pub fn new() -> DatadogFormatter {
        DatadogFormatter {
            traces: Vec::new(),
        }
    }

    pub fn add_trace(&mut self, trace: Trace) {
        self.traces.push(trace)
    }
}

impl Span {
    pub fn new(
        config: DatadogConfig,
        trace_id: u64,
        parent_id: Option<u64>,
        name: String,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Result<Span> {
        let span_id = new_span_id();

        let p_id = match parent_id {
            Some(id) => id,
            None => new_span_id(),
        };

        let mut meta = config.default_tags.clone();

        meta.insert("http.status_code".to_string(), "200".to_string());
        meta.insert("http.method".to_string(), "POST".to_string());
        meta.insert("http.url".to_string(), "http://localhost:3000".to_string());

        Ok(Span {
            trace_id,
            span_id,
            parent_id: p_id,
            name,
            meta,
            metrics: HashMap::new(),
            service: "planktonic".to_string(),
            start: start_time
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_nanos() as u64,
            duration: end_time.duration_since(start_time)?.as_nanos() as u64,
            resource: "request".to_string(),
            typ: config.trace_type.to_string(),
            error: 0,
        })
    }
}

pub fn new_trace_id() -> u64 {
    rand::thread_rng().gen::<u64>()
}

pub fn new_span_id() -> u64 {
    rand::thread_rng().gen::<u64>()
}

