pub mod otelstdout;
pub mod stdout;

use core::time;
use std::{sync::Arc, thread};

use anyhow::Result;
use tokio::sync::{mpsc::Sender, Mutex};

use crate::{Event, EventChannels};

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
        events: EventChannels,
    ) -> Result<Collector> {
        let (mut recv_events, send_events) = events;
        tokio::spawn(async move {
            while let Some(event) = recv_events.recv().await {
                adapter.lock().await.handle_event(event);
            }
        });

        Ok(Collector { id, send_events })
    }

    // flush any remaning spans
    pub async fn shutdown(&self) {
        // close event stream and join the thread handle
        self.send_events
            .send(Event::Shutdown(self.id))
            .await
            .unwrap();
        thread::sleep(time::Duration::from_millis(50));
    }
}
