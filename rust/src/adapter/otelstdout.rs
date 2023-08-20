use std::vec;

use crate::{Event, TraceEvent};
use anyhow::Result;

use super::{
    otel_formatter::{opentelemetry, OtelFormatter},
    Adapter, AdapterHandle,
};

/// An adapter to send events from your module to stdout using OpenTelemetry json format.
pub struct OtelStdoutAdapter {}

impl Adapter for OtelStdoutAdapter {
    fn handle_trace_event(&mut self, trace_evt: TraceEvent) -> Result<()> {
        let mut spans = vec![];
        let trace_id = trace_evt.telemetry_id.to_hex_16();
        for span in trace_evt.events {
            self.event_to_spans(&mut spans, span, vec![], trace_id.clone())?;
        }
        if !spans.is_empty() {
            let otf = OtelFormatter::new(spans, "stdout".into());
            println!("{:?}", &otf.traces_data);
        }
        Ok(())
    }
}

impl OtelStdoutAdapter {
    /// Creates the Otel Stdout adapter and spawns a task for it.
    /// This should ideally be created once per process of
    /// your rust application.
    pub fn create() -> AdapterHandle {
        Self::spawn(Self {})
    }

    fn event_to_spans(
        &self,
        spans: &mut Vec<opentelemetry::proto::trace::v1::Span>,
        event: Event,
        parent_id: Vec<u8>,
        trace_id: String,
    ) -> Result<()> {
        match event {
            Event::Func(f) => {
                let name = f.name.clone().unwrap_or("unknown-name".to_string());

                let span =
                    OtelFormatter::new_span(trace_id.clone(), parent_id, name, f.start, f.end);
                let span_id = span.span_id.clone();
                spans.push(span);

                for e in f.within.iter() {
                    self.event_to_spans(spans, e.to_owned(), span_id.clone(), trace_id.clone())?;
                }
            }
            Event::Alloc(a) => {
                if let Some(span) = spans.last_mut() {
                    OtelFormatter::add_attribute_i64_to_span(
                        span,
                        "allocation".to_string(),
                        a.amount.into(),
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }
}
