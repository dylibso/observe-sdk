use anyhow::{Result, Context};
use crate::{Event, TraceEvent};

use super::{
    otel_formatter::{Span, OtelFormatter, ResourceSpan}, AdapterHandle, Adapter
};

/// An adapter to send events from your module to stdout using OpenTelemetry json format.
pub struct OtelStdoutAdapter {
}

impl Adapter for OtelStdoutAdapter {
    fn handle_trace_event(&mut self, trace_evt: TraceEvent) -> Result<()> {
        let mut otf = OtelFormatter::new();
        let mut rs = ResourceSpan::new();
        let mut spans = vec![];
        let trace_id = trace_evt.telemetry_id.to_hex_16();
        for span in trace_evt.events {
            self.event_to_spans(&mut spans, span, None, trace_id.clone())?;
        }
        rs.add_spans(spans);
        otf.add_resource_span(rs);
        println!("{}", serde_json::to_string(&otf).context("Otel formatter could not create json")?);
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

    fn event_to_spans(&self, spans: &mut Vec<Span>, event: Event, parent_id: Option<String>, trace_id: String) -> Result<()> {
        match event {
            Event::Func(f) => {
                let name = f.name.clone().unwrap_or("unknown-name".to_string());

                let span =
                    Span::new(trace_id.clone(), parent_id, name, f.start, f.end);
                let span_id = Some(span.span_id.clone());
                spans.push(span);

                for e in f.within.iter() {
                    self.event_to_spans(spans, e.to_owned(), span_id.clone(), trace_id.clone())?;
                }
            }
            Event::Alloc(a) => {
                let mut span = Span::new(
                    trace_id,
                    parent_id,
                    "allocation".to_string(),
                    a.ts,
                    a.ts,
                );
                span.add_attribute_i64("amount".to_string(), a.amount.into());
                spans.push(span);
            }
            _ => {}
        }
        Ok(())
    }
}
