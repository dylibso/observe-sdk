use anyhow::Result;
use log::warn;
use tokio::sync::mpsc::{channel, Sender};
use wasmtime::Linker;

use crate::{
    adapter::otel_formatter::{
        opentelemetry,
        opentelemetry::proto::common::v1::{any_value::Value::IntValue, AnyValue},
        OtelFormatter,
    },
    collector::{Collector, CollectorHandle},
    context::add_to_linker,
    Event, TelemetryId, TraceEvent,
};

#[cfg(feature = "component-model")]
use crate::{ wasm_instr::WasmInstrInfo, context::{ InstrumentationContext, component::ObserveSdkBindings } };

use self::datadog::DatadogMetadata;

pub mod datadog;
pub mod datadog_formatter;
pub mod honeycomb;
pub mod lightstep;
pub mod otel_formatter;
pub mod otelstdout;
pub mod stdout;
pub mod zipkin;
pub mod zipkin_formatter;

pub use self::otel_formatter::Attribute;

/// An adapter represents a sink for events and is mostly implementation specific
/// to the sink that the data is being sent to and the format that the data is in.
/// Collectors batch up events and metadata into a TraceEvent and then sends this
/// payload to the Adapter when the module is done running. When implementing an adapter,
/// your job is to handle this event by implementing the function handle_trace_event.
///
/// ┌────────────────┐               ┌──────────────┐                ┌────────────────┐
/// │   Collector    │               │    Adapter   │                │   Collector    │
/// │                │               │              │                │                │
/// │                │               │              │                │                │
/// ├────────────────┤  <TraceEvent> ├──────────────┤  <TraceEvent>  ├────────────────┤
/// │  adapter_tx    ├──────────────►│ adapter_rx   │◄───────────────┤  adapter_tx    │
/// └────────────────┘               └──────────────┘                └────────────────┘
pub trait Adapter {
    /// Callback which is used when a TraceEvent is sent from the Collector
    /// This must be implemented by an Adapter
    fn handle_trace_event(&mut self, evt: TraceEvent) -> Result<()>;

    /// Spawns the tokio task and returns a cloneable handle to this adapter
    fn spawn(mut adapter: impl Adapter + Send + 'static) -> AdapterHandle {
        let (adapter_tx, mut adapter_rx) = channel(128);
        tokio::spawn(async move {
            while let Some(evt) = adapter_rx.recv().await {
                if let Err(e) = adapter.handle_trace_event(evt) {
                    warn!("Error handling events in adapter {e}");
                }
            }
        });
        AdapterHandle { adapter_tx }
    }

    fn event_to_otel_spans(
        &self,
        spans: &mut Vec<opentelemetry::proto::trace::v1::Span>,
        event: Event,
        parent_id: Vec<u8>,
        trace_id: String,
        meta: &Option<AdapterMetadata>,
    ) -> Result<()> {
        match event {
            Event::Func(f) => {
                let name = f.name.clone().unwrap_or("unknown-name".to_string());

                let mut span =
                    OtelFormatter::new_span(trace_id.clone(), parent_id, name, f.start, f.end);
                let span_id = span.span_id.clone();
                if let Some(m) = meta {
                    if let AdapterMetadata::OpenTelemetry(m) = m {
                        for entry in m.iter() {
                            if let Some(v) = entry.value.int_value {
                                OtelFormatter::add_attribute_i64_to_span(
                                    &mut span,
                                    entry.key.clone(),
                                    v,
                                )
                            } else if let Some(v) = entry.value.string_value.clone() {
                                OtelFormatter::add_attribute_string_to_span(
                                    &mut span,
                                    entry.key.clone(),
                                    v,
                                )
                            }
                        }
                    }
                }
                spans.push(span);

                for e in f.within.iter() {
                    self.event_to_otel_spans(
                        spans,
                        e.to_owned(),
                        span_id.clone(),
                        trace_id.clone(),
                        &meta,
                    )?;
                }
            }

            Event::Alloc(a) => {
                // add the allocation amount to the sum if exists, if not, create a new allocation attribute on the span
                if let Some(span) = spans.last_mut() {
                    let alloc_index = &span
                        .attributes
                        .iter()
                        .position(|attr| attr.key.eq("allocation"));
                    if let Some(i) = alloc_index {
                        if let Some(val) = &span.attributes[*i].value {
                            if let Some(IntValue(v)) = val.value {
                                let sum_amount = v as u32 + a.amount;
                                let kv = &mut span.attributes[*i];
                                kv.value = Some(AnyValue {
                                    value: Some(IntValue(sum_amount.into())),
                                });
                            }
                        }
                    } else {
                        OtelFormatter::add_attribute_i64_to_span(
                            span,
                            "allocation".to_string(),
                            a.amount.into(),
                        );
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Represents handle into the trace that is currently executing.
#[derive(Clone, Debug)]
pub struct TraceContext {
    collector: CollectorHandle,
}

impl TraceContext {
    pub async fn set_trace_id(&self, id: TelemetryId) {
        if let Err(e) = self.collector.send(Event::TraceId(id)).await {
            warn!("Failed to set the trace id {}", e);
        }
    }

    pub async fn set_metadata(&self, meta: AdapterMetadata) {
        if let Err(e) = self.collector.send(Event::Metadata(meta)).await {
            warn!("Failed to set the metdata {}", e);
        }
    }

    pub async fn shutdown(&self) {
        if let Err(e) = self.collector.send(Event::Shutdown).await {
            warn!("Failed to shutdown collector {}", e);
        }
    }
}

/// Represents a cloneable handle to the Adapter. Calling start gives
/// you a TraceContext that is linked to the Wasm module.
#[derive(Clone, Debug)]
pub struct AdapterHandle {
    adapter_tx: Sender<TraceEvent>,
}

impl AdapterHandle {
    pub fn start<T: 'static>(
        &self,
        linker: &mut Linker<T>,
        data: &[u8],
        options: Options,
    ) -> Result<TraceContext> {
        let (collector, collector_rx) = add_to_linker(linker, data, options)?;
        Collector::start(collector_rx, self.clone());
        Ok(TraceContext { collector })
    }

    #[cfg(feature = "component-model")]
    pub fn create_bindings(&self, data: &[u8], options: Options) -> Result<(ObserveSdkBindings, TraceContext)> {
        let (ctx, collector, collector_rx) = InstrumentationContext::new(options);
        let wasm_instr_info = WasmInstrInfo::new(data)?;

        // check that the version number is supported with this SDK
        // TODO decide what to do about this error?
        if let Err(e) = wasm_instr_info.check_version() {
            warn!("{}", e);
        }

        let bindings = ObserveSdkBindings {
            instr_context: ctx,
            wasm_instr_info
        };
        Collector::start(collector_rx, self.clone());
        Ok((bindings, TraceContext { collector }))
    }


    pub fn try_send(&self, event: TraceEvent) -> Result<()> {
        self.adapter_tx.try_send(event)?;
        Ok(())
    }
}

/// The different types of metadata we can send across to a Collector
#[derive(Clone, Debug)]
pub enum AdapterMetadata {
    Datadog(DatadogMetadata),
    OpenTelemetry(Vec<Attribute>),
}

const MIN_SPAN_FILTER_DURATION_DEFAULT: u64 = 20;

/// SpanFilter allows for specification of how to filter out spans
#[derive(Clone)]
pub struct SpanFilter {
    pub min_duration_microseconds: std::time::Duration,
}

impl Default for SpanFilter {
    fn default() -> Self {
        Self {
            min_duration_microseconds: std::time::Duration::from_micros(
                MIN_SPAN_FILTER_DURATION_DEFAULT,
            ),
        }
    }
}

/// Options allow you to tune certain characteristics of your telemetry
#[derive(Clone)]
pub struct Options {
    pub span_filter: SpanFilter,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            span_filter: SpanFilter {
                min_duration_microseconds: std::time::Duration::from_micros(
                    MIN_SPAN_FILTER_DURATION_DEFAULT,
                ),
            },
        }
    }
}
