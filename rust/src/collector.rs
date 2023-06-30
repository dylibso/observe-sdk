use anyhow::Result;
use log::warn;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{Event, adapter::AdapterHandle, TelemetryId, TraceEvent};

pub type CollectorHandle = Sender<Event>;

/// A collector is spawned for each Wasm module. It receives Events
/// from the InsturmentationContext object in the background as the Wasm
/// module is running. It collects Events until ti receives the Shutdown message,
/// then it crafts a TraceEvent to send to the Adapter.
///
/// ┌────────────────┐         ┌──────────────┐               ┌───────────────┐
/// │  InstrContext  │         │  Collector   │               │               │
/// │                │         │              │               │    Adapter    │
/// │                │         │              │               │               │
/// ├────────────────┤ <Event> ├──────────────┤               │               │
/// │  collector_tx  ├────────►│ collector_rx │               │               │
/// └────────────────┘         ├──────────────┤ <TraceEvent>  ├───────────────┤
///                            │  adapter_tx  ├──────────────►│  adapter_rx   │
///                            └──────────────┘               └───────────────┘
pub struct Collector {
    events: Vec<Event>,
    adapter: AdapterHandle,
    telemetry_id: Option<TelemetryId>,
}

impl Collector {
    fn new(adapter: AdapterHandle) -> Collector {
        Collector {
            adapter,
            events: Vec::new(),
            telemetry_id: None,
        }
    }

    /// Spawns the collector in it's own task given an rx channel
    /// It needs the Event receiver and an AdapterHandle to send the TraceEvent to
    pub fn start(mut events_rx: Receiver<Event>, adapter: AdapterHandle) {
        tokio::spawn(async move {
            let mut collector = Collector::new(adapter);
            while let Some(event) = events_rx.recv().await {
                if let Err(e) = collector.handle_event(event) {
                    warn!("Collector error occurred while handling event {e}");
                };
            }
        });
    }

    /// Events coming in from the InstrumentationContext
    /// Here we queue up events until we get the collector shutdown event
    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::TraceId(id) => {
                self.telemetry_id = Some(id);
            }
            Event::Func(func) => {
                self.events.push(Event::Func(func));
            }
            Event::Shutdown => {
                let trace = TraceEvent {
                    events: self.events.clone(),
                    telemetry_id: self.telemetry_id.clone(),
                };
                self.adapter.try_send(trace)?;
            }
            _ => {
                self.events.push(event.clone());
            }
        }
        Ok(())
    }
}


