use opentelemetry::{
    global::{self, set_tracer_provider, shutdown_tracer_provider, BoxedTracer},
    sdk::trace::TracerProvider,
    trace::{Span, SpanBuilder, Tracer},
    KeyValue,
};

use crate::Event;
use opentelemetry_stdout;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread, time,
};
use tokio::sync::Mutex;

use super::Adapter;

//use super::json_span_exporter;

pub type OtelAdapterContainer = Arc<Mutex<OtelStdoutAdapter>>;

#[derive(Clone, Copy)]
pub struct OtelStdoutAdapter {}

impl OtelStdoutAdapter {
    pub fn new() -> OtelAdapterContainer {
        let exporter = opentelemetry_stdout::SpanExporter::default();
        let provider = TracerProvider::builder()
            .with_simple_exporter(exporter)
            .build();
        set_tracer_provider(provider);

        let adapter = OtelStdoutAdapter {};

        Arc::new(Mutex::new(adapter))
    }

    pub fn new_collector(&mut self) -> usize {
        next_id()
    }

    pub fn start_trace<F>(module_name: String, action_name: String, f: F)
    where
        F: FnOnce(),
    {
        global::tracer(module_name).in_span(action_name, |_cx| {
            f();
        });
    }

    fn _handle_event(&mut self, event: Event, tracer: &BoxedTracer) {
        match event {
            Event::Func(_id, f) => {
                let mut x = tracer.build(SpanBuilder {
                    start_time: Some(f.start),
                    name: format!(
                        "function-call-{}",
                        &f.name.clone().unwrap_or("unknown-name".to_string())
                    )
                    .into(),
                    ..Default::default()
                });

                if let Some(name) = f.name {
                    x.set_attribute(KeyValue::new("function_name", name));
                }

                for e in f.within.iter() {
                    self.handle_event(e.to_owned());
                }

                x.end_with_timestamp(f.end);
            }
            Event::Alloc(_id, a) => {
                tracer
                    .build(SpanBuilder {
                        name: "allocation".into(),
                        start_time: Some(a.ts),
                        end_time: Some(a.ts),
                        ..Default::default()
                    })
                    .add_event_with_timestamp(
                        "allocation",
                        a.ts,
                        vec![KeyValue::new("amount", a.amount as i64)],
                    );
            }
            Event::Metadata(_id, _) => {}
            Event::Shutdown(_id) => {}
        }
    }
}

impl Adapter for OtelStdoutAdapter {
    // flush any remaning spans
    fn shutdown(&self) {
        thread::sleep(time::Duration::from_millis(5));
        shutdown_tracer_provider();
    }

    fn handle_event(&mut self, event: Event) {
        let tracer = global::tracer("event");
        self._handle_event(event, &tracer);
    }
}

fn next_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
