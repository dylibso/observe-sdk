pub mod otel_formatter;
pub mod otelstdout;
pub mod stdout;

use core::time;
use std::{sync::Arc, thread};

use anyhow::Result;
use tokio::sync::{mpsc::Sender, Mutex};

use crate::{Event, EventChannel, Metadata};

pub trait Adapter {
    fn handle_event(&mut self, event: Event);
    fn shutdown(&self);
}

pub struct Collector {
    id: usize,
    send_events: Sender<Event>,
}

impl Collector {
    pub async fn new<A: Adapter + Send + Sync + 'static>(
        adapter: Arc<Mutex<A>>,
        id: usize,
        events: EventChannel,
    ) -> Result<Collector> {
        let (events_tx, mut events_rx) = events;
        tokio::spawn(async move {
            while let Some(event) = events_rx.recv().await {
                adapter.lock().await.handle_event(event);
            }
        });

        Ok(Collector {
            id,
            send_events: events_tx,
        })
    }

    // flush any remaning spans
    pub async fn shutdown(&self) {
        // close event stream and join the thread handle
        self.send_events
            .send(Event::Shutdown(self.id))
            .await
            .unwrap();
        thread::sleep(time::Duration::from_millis(150));
    }

    pub async fn set_metadata(&self, key: String, value: String) {
        self.send_events
            .send(Event::Metadata(0, Metadata { key, value }))
            .await
            .unwrap();
    }
}
