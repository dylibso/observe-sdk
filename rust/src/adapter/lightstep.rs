use prost::Message;
use std::time::Duration;

use crate::{Event, TraceEvent};
use anyhow::Result;

use super::{
    otlp_formatter::{opentelemetry, OtelFormatter},
    Adapter, AdapterHandle, AdapterMetadata,
};

/// Config options for LightstepAdapter
#[derive(Debug, Clone)]
pub struct LightstepConfig {
    pub api_key: String,
    pub host: String,
    pub dataset: String,
}

/// An adapter to send events from your module to lightstep using OpenTelemetry json format.
pub struct LightstepAdapter {
    trace_events: Vec<TraceEvent>,
    config: LightstepConfig,
}

impl Adapter for LightstepAdapter {
    fn handle_trace_event(&mut self, trace_evt: TraceEvent) -> Result<()> {
        self.trace_events.push(trace_evt);
        self.dump_traces()?;
        Ok(())
    }
}

impl LightstepAdapter {
    /// Creates the Lightstep adapter and spawns a task for it.
    /// This should ideally be created once per process of
    /// your rust application.
    pub fn create(config: LightstepConfig) -> AdapterHandle {
        Self::spawn(Self {
            config,
            trace_events: vec![],
        })
    }

    fn event_to_spans(
        &self,
        spans: &mut Vec<opentelemetry::proto::trace::v1::Span>,
        event: Event,
        parent_id: Vec<u8>,
        trace_id: String,
        meta: &Option<AdapterMetadata>,
    ) -> Result<()> {
        match event {
            Event::Func(f) => {
                let name = f.name.clone().unwrap_or("unknown-name".to_string());

                let span =
                    OtelFormatter::new_span(trace_id.clone(), parent_id, name, f.start, f.end);
                let span_id = span.span_id.clone();
                // if let Some(m) = meta {
                // if let AdapterMetadata::OpenTelemetry(m) = m {
                //     for entry in m.iter() {
                //         if let Some(v) = entry.value.int_value {
                //             span.add_attribute_i64(entry.key.clone(), v)
                //         } else if let Some(v) = entry.value.string_value.clone() {
                //             span.add_attribute_string(entry.key.clone(), v)
                //         }
                //     }
                // }
                // }
                spans.push(span);

                for e in f.within.iter() {
                    self.event_to_spans(
                        spans,
                        e.to_owned(),
                        span_id.clone(),
                        trace_id.clone(),
                        &meta,
                    )?;
                }
            }
            Event::Alloc(a) => {
                let span = OtelFormatter::new_span(
                    trace_id,
                    parent_id,
                    "allocation".to_string(),
                    a.ts,
                    a.ts,
                );
                // span.add_attribute_i64("amount".to_string(), a.amount.into());
                spans.push(span);
            }
            _ => {}
        }
        Ok(())
    }

    fn dump_traces(&mut self) -> Result<()> {
        let mut spans = vec![];
        for te in &self.trace_events {
            let trace_id = te.telemetry_id.to_hex_16();
            for span in &te.events {
                self.event_to_spans(
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
        println!("{}", otf.traces_data.resource_spans.len());
        println!("{}", otf.traces_data.resource_spans[0].scope_spans.len());
        println!(
            "{}",
            otf.traces_data.resource_spans[0].scope_spans[0].spans.len()
        );
        // rs.add_spans(spans);
        // rs = rs.add_attribute("service.name".into(), dataset.into());
        // otf.add_resource_span(rs);

        let host = url::Url::parse(&self.config.host)?;
        let url = host.join("traces/otlp/v0.9")?.to_string();
        // let j = serde_json::json!(&otf);
        // let body = serde_json::to_string(&j)?;

        let response = ureq::post(&url)
            .timeout(Duration::from_secs(1))
            .set("content-type", "application/x-protobuf")
            .set("lightstep-access-token", &self.config.api_key)
            .send_bytes(&otf.traces_data.encode_to_vec());

        match response {
            Ok(r) => {
                if r.status() != 200 {
                    log::warn!("Request to lightstep agent failed with status: {:#?}", r);
                } else {
                    println!("success!");
                    // clear the traces because we've successfully submitted them
                    self.trace_events.clear();
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }

        Ok(())
    }
}
