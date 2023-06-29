use std::sync::Arc;
use std::time::Duration;
use log::warn;
use serde_json::json;

use crate::adapter::zipkin_formatter::{ZipkinFormatter, Span, LocalEndpoint};
use crate::adapter::{Adapter, Collector, new_trace_id};
use crate::{add_to_linker, Event};

use anyhow::Result;
use tokio::sync::Mutex;
use wasmtime::Linker;

use super::{next_id, new_span_id, AdapterMetadata, TelemetryId};

pub struct ZipkinAdapterContainer(Arc<Mutex<ZipkinAdapter>>);

pub struct ZipkinTraceCtx(Collector);

impl ZipkinTraceCtx {
    pub async fn set_metadata(&mut self, meta: ZipkinMetadata) {
        self.0.set_metadata(AdapterMetadata::Zipkin(meta)).await;
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


#[derive(Builder, Debug, Clone)]
pub struct ZipkinMetadata {
    pub trace_id: String,
}

#[derive(Clone)]
pub struct ZipkinAdapter {
    pub trace_id: String,
    pub meta: Option<ZipkinMetadata>,
    pub spans: Vec<Span>,
}

impl ZipkinAdapter {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ZipkinAdapterContainer {
        let adapter = Self {
            trace_id: new_trace_id().to_hex_16(),
            meta: None,
            spans: vec![],
        };

        ZipkinAdapterContainer(Arc::new(Mutex::new(adapter)))
    }

    fn _handle_event(&mut self, event: Event, parent_span_id: Option<String>) -> Option<Vec<Span>> {
        match event {
            Event::Func(_id, f) => {
                let name = f.name.clone().unwrap_or("unknown-name".to_string());

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
            Event::Metadata(AdapterMetadata::Zipkin(meta)) => {
                self.meta = Some(meta);
                None
            }
            Event::TelemetryId(id) => {
                self.trace_id = id.to_hex_16();
                None
            }
            Event::Shutdown(_id) => {
                if let Err(e) = self.shutdown() {
                    warn!("Failed to shutdown Zipkin adapter {}", e);
                }
                self.spans.clear();
                None
            }
            Event::Metadata(_) => {
                log::warn!("Received non Zipkin metadata event");
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

        let mut first_span = ztf.spans.first_mut().unwrap();
        first_span.local_endpoint = Some(LocalEndpoint { service_name: Some("my_service".into()) });

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

    fn handle_event(&mut self, event: Event) {
        if let Some(spans) = self._handle_event(event, None) {
            for span in spans {
                self.spans.push(span);
            }
        };
    }
}