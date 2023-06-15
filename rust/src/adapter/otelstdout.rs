use std::sync::Arc;
use std::{thread, time};

use crate::adapter::otel_formatter::{OtelFormatter, ResourceSpan, Span};
use crate::adapter::{Adapter, Collector};
use crate::{add_to_linker, Event, Metadata};

use anyhow::Result;
use tokio::sync::Mutex;
use wasmtime::Linker;

use super::{next_id, TelemetryId, new_trace_id};

pub struct OtelAdapterContainer(Arc<Mutex<OtelStdoutAdapter>>);

pub struct OtelTraceCtx(Collector);

impl OtelTraceCtx {
    pub async fn set_trace_id(&mut self, id: TelemetryId) {
        self.0.set_metadata("trace_id".to_string(), id).await;
    }

    pub async fn shutdown(&self) {
        self.0.shutdown().await;
    }
}

impl OtelAdapterContainer {
    pub async fn start<T: 'static>(&self, linker: &mut Linker<T>) -> Result<OtelTraceCtx> {
        let id = next_id();
        let events = add_to_linker(id, linker)?;
        let collector = Collector::new(self.0.clone(), id, events).await?;

        Ok(OtelTraceCtx(collector))
    }
}

#[derive(Clone)]
pub struct OtelStdoutAdapter {
    pub trace_id: String,
}

impl OtelStdoutAdapter {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> OtelAdapterContainer {
        let adapter = Self {
            trace_id: new_trace_id().to_string(),
        };

        OtelAdapterContainer(Arc::new(Mutex::new(adapter)))
    }

    fn _handle_event(&mut self, event: Event, parent_span_id: Option<String>) -> Option<Vec<Span>> {
        match event {
            Event::Func(_id, f) => {
                let function_name = &f.name.clone().unwrap_or("unknown-name".to_string());
                let name = format!("function-call-{}", &function_name);

                let mut span =
                    Span::new(self.trace_id.clone(), parent_span_id, name, f.start, f.end);
                span.add_attribute_string("function_name".to_string(), function_name.to_string());

                let span_id = Some(span.span_id.clone());
                let mut spans = vec![span];

                for e in f.within.iter() {
                    if let Some(mut s) = self._handle_event(e.to_owned(), span_id.clone()) {
                        spans.append(&mut s);
                    };
                }

                Some(spans)
            }
            Event::Alloc(_id, a) => {
                let mut span = Span::new(
                    self.trace_id.clone(),
                    parent_span_id,
                    "allocation".to_string(),
                    a.ts,
                    a.ts,
                );
                span.add_attribute_i64("amount".to_string(), a.amount.into());
                Some(vec![span])
            }
            Event::Metadata(_id, Metadata { key, value }) => {
                if key == "trace_id" {
                    self.trace_id = value.to_string();
                }

                None
            }
            Event::Shutdown(_id) => None,
        }
    }
}

impl Adapter for OtelStdoutAdapter {
    // flush any remaning spans
    fn shutdown(&self) -> Result<()> {
        thread::sleep(time::Duration::from_millis(5));
        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        if let Some(spans) = self._handle_event(event, None) {
            let mut otf = OtelFormatter::new();
            let mut rs = ResourceSpan::new();
            rs.add_spans(spans);
            otf.add_resource_span(rs);

            println!("{}", serde_json::to_string(&otf).unwrap());
        };
    }
}

