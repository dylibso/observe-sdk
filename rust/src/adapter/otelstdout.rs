use std::vec;

use crate::TraceEvent;
use anyhow::Result;

use super::{otel_formatter::OtelFormatter, Adapter, AdapterHandle};

/// An adapter to send events from your module to stdout using OpenTelemetry json format.
pub struct OtelStdoutAdapter {}

impl Adapter for OtelStdoutAdapter {
    fn handle_trace_event(&mut self, trace_evt: TraceEvent) -> Result<()> {
        let mut spans = vec![];
        let trace_id = trace_evt.telemetry_id.to_hex_16();
        for span in trace_evt.events {
            self.event_to_otel_spans(
                &mut spans,
                span,
                vec![],
                trace_id.clone(),
                &trace_evt.metadata,
            )?;
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
}
