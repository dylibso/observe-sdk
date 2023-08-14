use super::datadog::DatadogConfig;
use crate::new_span_id;
use anyhow::Result;
use serde::Serialize;
use std::{collections::HashMap, time::SystemTime};

#[derive(Default)]
pub struct DatadogFormatter {
    pub traces: Vec<Trace>,
}

pub type Trace = Vec<Span>;

impl DatadogFormatter {
    pub fn new() -> DatadogFormatter {
        Default::default()
    }

    pub fn add_trace(&mut self, trace: Trace) {
        self.traces.push(trace)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
pub struct Span {
    pub trace_id: u64,
    pub span_id: u64,
    pub parent_id: Option<u64>,
    pub name: String,
    pub start: u64,
    pub duration: u64,
    pub resource: String,
    pub error: u8,
    pub meta: HashMap<String, String>,
    pub metrics: HashMap<String, f64>,
    pub service: String,
    #[serde(rename = "type")]
    pub typ: Option<String>,
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
        let span_id = new_span_id().into();

        Ok(Span {
            trace_id,
            span_id,
            parent_id,
            name: name.clone(),
            meta: HashMap::new(),
            metrics: HashMap::new(),
            service: config.service_name,
            start: start_time
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_nanos() as u64,
            duration: end_time.duration_since(start_time)?.as_nanos() as u64,
            resource: name,
            typ: None,
            error: 0,
        })
    }

    pub fn add_allocation(&mut self, amount: u32) {
        self.meta
            .insert("allocation".to_string(), amount.to_string());
    }

    pub fn add_tag(&mut self, tag: String) {
        // TODO need a more robust way to do this
        let parts: Vec<&str> = tag.split(|c| c == ':').collect();
        let key = parts.get(0).unwrap();
        let value = parts.get(1).unwrap();
        self.meta.insert(key.to_string(), value.to_string());
    }
}
