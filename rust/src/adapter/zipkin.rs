use anyhow::{Context, Result};
use log::warn;
use serde_json::json;
use std::time::Duration;
use ureq;

use crate::{Event, TraceEvent};

use super::{
    zipkin_formatter::{LocalEndpoint, Span, ZipkinFormatter},
    Adapter, AdapterHandle,
};

/// An adapter to send events from your module to a [Zipkin Instance](https://zipkin.io/).
pub struct ZipkinAdapter {}

impl Adapter for ZipkinAdapter {
    fn handle_trace_event(&mut self, trace_evt: TraceEvent) -> Result<()> {
        let mut spans = vec![];
        let trace_id = trace_evt.telemetry_id.to_hex_16();
        for span in trace_evt.events {
            self.event_to_spans(&mut spans, span, None, trace_id.clone())?;
        }
        self.dump_trace(spans)?;
        Ok(())
    }
}

impl ZipkinAdapter {
    /// Creates the Zipkin adapter and spawns a task for it.
    /// This should ideally be created once per process of
    /// your rust application.
    pub fn create() -> AdapterHandle {
        Self::spawn(Self {})
    }

    fn event_to_spans(
        &self,
        spans: &mut Vec<Span>,
        event: Event,
        parent_id: Option<String>,
        trace_id: String,
    ) -> Result<()> {
        match event {
            Event::Func(f) => {
                let name = f.name.clone().unwrap_or("unknown-name".to_string());

                let span = Span::new(trace_id.clone(), parent_id, name, f.start, f.end);
                let span_id = Some(span.id.clone());
                spans.push(span);

                for e in f.within.iter() {
                    self.event_to_spans(spans, e.to_owned(), span_id.clone(), trace_id.clone())?;
                }
            }
            Event::Alloc(a) => {
                let mut span = Span::new(trace_id, parent_id, "allocation".to_string(), a.ts, a.ts);
                span.add_tag_i64("amount".to_string(), a.amount.into());
                spans.push(span);
            }
            _ => {}
        }
        Ok(())
    }

    fn dump_trace(&mut self, spans: Vec<Span>) -> Result<()> {
        let mut ztf = ZipkinFormatter::new();
        ztf.spans = spans;

        let mut first_span = ztf
            .spans
            .first_mut()
            .context("No spans to send to zipkin")?;
        first_span.local_endpoint = Some(LocalEndpoint {
            service_name: Some("my_service".into()),
        });

        let url = "http://localhost:9411/api/v2/spans";
        let j = json!(&ztf.spans);
        let body = serde_json::to_string(&j)?;

        // perhaps this should be an async operation with something
        // like reqwest?
        let response = ureq::post(url)
            .timeout(Duration::from_secs(1))
            .set("Content-Type", "application/json")
            .send_string(&body);

        // should maybe retry or throw an exception
        // TODO check default logic of http client
        if response.is_ok() {
            warn!("Request to Zipkin failed: {:#?}", response);
        }

        Ok(())
    }
}
