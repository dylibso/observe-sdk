use crate::{
    adapter::otel_formater::{OtelFormatter, ResourceSpan},
    Event, Metadata,
};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread, time,
};
use tokio::sync::Mutex;

use super::{
    otel_formater::{self, Span},
    Adapter, Collector,
};

pub type OtelAdapterContainer = Arc<Mutex<OtelStdoutAdapter>>;

#[derive(Clone)]
pub struct OtelStdoutAdapter {
    pub trace_id: String,
}

impl OtelStdoutAdapter {
    pub fn new() -> OtelAdapterContainer {
        let adapter = OtelStdoutAdapter {
            trace_id: otel_formater::new_span_id(),
        };

        Arc::new(Mutex::new(adapter))
    }

    pub fn new_collector(&mut self) -> usize {
        next_id()
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
                    self.trace_id = value;
                }

                None
            }
            Event::Shutdown(_id) => None,
        }
    }

    pub async fn set_trace_id(collector: &Collector, trace_id: String) {
        collector
            .set_metadata("trace_id".to_string(), trace_id)
            .await;
    }
}

impl Adapter for OtelStdoutAdapter {
    // flush any remaning spans
    fn shutdown(&self) {
        thread::sleep(time::Duration::from_millis(5));
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

fn next_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
