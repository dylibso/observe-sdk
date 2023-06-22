pub mod datadog;
pub mod datadog_formatter;
pub mod otel_formatter;
pub mod otelstdout;
pub mod stdout;
pub mod zipkin;
pub mod zipkin_formatter;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::Result;
use rand::Rng;
use tokio::sync::{mpsc::Sender, Mutex};
use tokio::task::JoinHandle;

use crate::{Event, EventChannel, Metadata};

#[derive(Debug, Clone)]
pub struct TelemetryId(u128);

impl TelemetryId {
    /// format as 8-byte zero-prefixed hex string
    pub fn to_hex_8(&self) -> String {
        // 8 bytes is 16 chars
        format!("{:016x}", self.0 as u64)
    }
    /// format as 16-byte zero-prefixed hex string
    pub fn to_hex_16(&self) -> String {
        // 16 bytes is 32 chars
        format!("{:032x}", self.0)
    }
}

impl From<TelemetryId> for u64 {
    fn from(v: TelemetryId) -> Self {
        v.0 as u64
    }
}

impl From<TelemetryId> for u128 {
    fn from(v: TelemetryId) -> Self {
        v.0
    }
}

pub fn new_trace_id() -> TelemetryId {
    let x = rand::thread_rng().gen::<u128>();
    TelemetryId(x)
}

pub fn new_span_id() -> TelemetryId {
    let x = rand::thread_rng().gen::<u128>();
    TelemetryId(x)
}

pub trait Adapter {
    fn handle_event(&mut self, event: Event);
    fn shutdown(&self) -> Result<()>;
}

pub struct Collector {
    id: usize,
    send_events: Sender<Event>,
    join_handle: JoinHandle<()>,
}

impl Collector {
    pub async fn new<A: Adapter + Send + Sync + 'static>(
        adapter: Arc<Mutex<A>>,
        id: usize,
        events: EventChannel,
    ) -> Result<Collector> {
        let (events_tx, mut events_rx) = events;
        let join_handle = tokio::spawn(async move {
            while let Some(event) = events_rx.recv().await {
                if let Event::Shutdown(_id) = event {
                    adapter.lock().await.handle_event(event);
                    return;
                } else {
                    adapter.lock().await.handle_event(event);
                }
            }
        });

        Ok(Collector {
            id,
            send_events: events_tx,
            join_handle,
        })
    }

    // flush any remaining spans
    pub async fn shutdown(self) -> Result<()> {
        // close event stream and join the thread handle
        self.send_events
            .send(Event::Shutdown(self.id))
            .await
            .unwrap();
        self.join_handle.await?;
        Ok(())
    }

    pub async fn set_metadata(&self, key: String, value: TelemetryId) {
        self.send_events
            .send(Event::Metadata(0, Metadata { key, value }))
            .await
            .unwrap();
    }
}

fn next_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_ids() {
        let id = new_trace_id();
        assert_eq!(id.to_hex_8().len(), 16);
        assert_eq!(id.to_hex_16().len(), 32);
    }
}
