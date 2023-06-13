use anyhow::Result;
use opentelemetry::{
    global::{self, set_tracer_provider, shutdown_tracer_provider, BoxedTracer},
    sdk::trace::TracerProvider,
    trace::{Span, SpanBuilder, Tracer},
    KeyValue,
};

use crate::{add_to_linker, Event};
use opentelemetry_stdout;
use std::sync::atomic::{AtomicUsize, Ordering};
use wasmtime::Linker;

//use super::json_span_exporter;

#[derive(Clone)]
pub struct OtelStdoutAdapter {}

impl OtelStdoutAdapter {
    pub async fn new<T: 'static>(linker: &mut Linker<T>) -> Result<OtelStdoutAdapter> {
        // let _ = stdout::new_pipeline()
        //                .install_simple();

        let exporter = opentelemetry_stdout::SpanExporter::default();
        let provider = TracerProvider::builder()
            .with_simple_exporter(exporter)
            .build();
        set_tracer_provider(provider);

        let (mut recv_events, _) = add_to_linker(next_id(), linker, &Vec::new())?;
        let adapter = OtelStdoutAdapter {};

        let a2 = adapter.clone();

        tokio::spawn(async move {
            let tracer = global::tracer("demo");
            tracer.in_span("receiver", |_cx| {
                while let Some(event) = recv_events.blocking_recv() {
                    a2.handle_event(event, &tracer);
                }
            })
        });

        Ok(adapter)
    }

    pub fn handle_event(&self, event: Event, tracer: &BoxedTracer) {
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
                    self.handle_event(e.to_owned(), tracer);
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
            Event::Metadata(_id, _) => todo!(),
            Event::Shutdown(_id) => todo!(),
        }
    }

    // flush any remaning spans
    pub async fn shutdown() {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        shutdown_tracer_provider();
    }
}

fn next_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
