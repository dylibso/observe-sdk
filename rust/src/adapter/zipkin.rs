use std::sync::Arc;
use log::warn;
use serde_json::json;

use crate::adapter::zipkin_formatter::{ZipkinFormatter, Span};
use crate::adapter::{Adapter, Collector};
use crate::{add_to_linker, Event, Metadata};

use anyhow::Result;
use tokio::sync::Mutex;
use wasmtime::Linker;

use super::{TelemetryId, next_id, new_span_id};

pub struct ZipkinAdapterContainer(Arc<Mutex<ZipkinAdapter>>);

pub struct ZipkinTraceCtx(Collector);

impl ZipkinTraceCtx {
    pub async fn set_trace_id(&mut self, id: TelemetryId) {
        self.0.set_metadata("trace_id".to_string(), id).await;
    }

    pub async fn shutdown(&self) {
        self.0.shutdown().await;
    }
}

impl ZipkinAdapterContainer {
    pub async fn start<T: 'static>(&self, linker: &mut Linker<T>, data: &[u8]) -> Result<ZipkinTraceCtx> {
        let id = next_id();
        let events = add_to_linker(id, linker, data)?;
        let collector = Collector::new(self.0.clone(), id, events).await?;

        Ok(ZipkinTraceCtx(collector))
    }
}

#[derive(Clone)]
pub struct ZipkinAdapter {
    pub trace_id: String,
    pub spans: Vec<Span>,
}

impl ZipkinAdapter {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ZipkinAdapterContainer {
        let adapter = Self {
            trace_id: new_span_id().to_hex_8(),
            spans: vec![],
        };

        ZipkinAdapterContainer(Arc::new(Mutex::new(adapter)))
    }

    fn _handle_event(&mut self, event: Event, parent_span_id: Option<String>) -> Option<Vec<Span>> {
        match event {
            Event::Func(_id, f) => {
                let function_name = &f.name.clone().unwrap_or("unknown-name".to_string());
                let name = format!("function-call-{}", &function_name);

                let span =
                    Span::new(self.trace_id.clone(), parent_span_id, name, f.start, f.end);

                let span_id = Some(span.id.clone());
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
                span.add_tag_i64("amount".to_string(), a.amount.into());
                Some(vec![span])
            }
            Event::Metadata(_id, Metadata { key, value }) => {
                if key == "trace_id" {
                    self.trace_id = value.to_hex_8();
                }

                None
            }
            Event::Shutdown(_id) => {
                if let Err(e) = self.shutdown() {
                    warn!("Failed to shutdown Datadog adapter {}", e);
                }
                self.spans.clear();
                None
            }
        }
    }
}

impl Adapter for ZipkinAdapter {
    // flush any remaning spans
    fn shutdown(&self) -> Result<()> {
        let mut ztf = ZipkinFormatter::new();
        ztf.spans = self.spans.clone();

        let url = "http://localhost:9411/api/v2/spans";
        let j = json!(&ztf.spans);
        let body = serde_json::to_string(&j)?;

        println!("{}", &body);

        // perhaps this should be an async operation with something
        // like reqwest?
        let response = ureq::post(url)
            .set("Content-Type", "application/json")
            .send_string(&body);

        // should maybe retry or throw an exception
        // TODO check default logic of http client
        if response.is_ok() {
            println!("Request was successful!");
        } else {
            println!("Request failed with status: {:#?}", response);
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        if let Some(spans) = self._handle_event(event, None) {
            for span in spans {
                self.spans.push(span);
            }

            //println!("{}", serde_json::to_string(&otf).unwrap());
        };
    }
}
