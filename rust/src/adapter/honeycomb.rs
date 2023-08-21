use prost::Message;
use std::time::Duration;

use crate::TraceEvent;
use anyhow::Result;

use super::{otel_formatter::OtelFormatter, Adapter, AdapterHandle};

pub use super::{
    otel_formatter::{Attribute, Value},
    AdapterMetadata, Options, SpanFilter,
};

/// Config options for HoneycombAdapter
#[derive(Debug, Clone)]
pub struct HoneycombConfig {
    pub api_key: String,
    pub host: String,
    pub dataset: String,
}

/// An adapter to send events from your module to honeycomb using OpenTelemetry json format.
pub struct HoneycombAdapter {
    trace_events: Vec<TraceEvent>,
    config: HoneycombConfig,
}

impl Adapter for HoneycombAdapter {
    fn handle_trace_event(&mut self, trace_evt: TraceEvent) -> Result<()> {
        self.trace_events.push(trace_evt);
        self.dump_traces()?;
        Ok(())
    }
}

impl HoneycombAdapter {
    /// Creates the Honeycomb adapter and spawns a task for it.
    /// This should ideally be created once per process of
    /// your rust application.
    pub fn create(config: HoneycombConfig) -> AdapterHandle {
        Self::spawn(Self {
            config,
            trace_events: vec![],
        })
    }

    fn dump_traces(&mut self) -> Result<()> {
        let mut spans = vec![];
        for te in &self.trace_events {
            let trace_id = te.telemetry_id.to_hex_16();
            for span in &te.events {
                self.event_to_otel_spans(
                    &mut spans,
                    span.clone(),
                    vec![],
                    trace_id.clone(),
                    &te.metadata,
                )?;
            }
        }
        let dataset = &self.config.dataset.clone();
        let otf = OtelFormatter::new(spans, dataset.into());

        let host = url::Url::parse(&self.config.host)?;
        let url = host.join("/v1/traces")?.to_string();

        let response = ureq::post(&url)
            .timeout(Duration::from_secs(1))
            .set("content-type", "application/protobuf")
            .set("x-honeycomb-team", &self.config.api_key)
            .send_bytes(&otf.traces_data.encode_to_vec());

        match response {
            Ok(r) => {
                if r.status() != 200 {
                    log::warn!("Request to honeycomb agent failed with status: {:#?}", r);
                } else {
                    // clear the traces because we've successfully submitted them
                    self.trace_events.clear();
                }
            }
            Err(e) => {
                log::warn!("Request to honeycomb agent failed: {:#?}", e);
            }
        }

        Ok(())
    }
}
