use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{self, SystemTime};

use crate::adapter::Adapter;
use crate::Event;

use tokio::sync::Mutex;

pub type StdoutAdapterContainer = Arc<Mutex<StdoutAdapter>>;

pub struct StdoutAdapter {
    collectors: HashMap<usize, Collector>,
}

struct Collector {
    start: SystemTime,
}

impl StdoutAdapter {
    pub fn new() -> StdoutAdapterContainer {
        let adapter = StdoutAdapter {
            collectors: HashMap::new(),
        };

        Arc::new(Mutex::new(adapter))
    }

    pub fn new_collector(&mut self) -> usize {
        let id = next_id();

        self.collectors.insert(
            id,
            Collector {
                start: SystemTime::now(),
            },
        );

        id
    }

    fn _handle_event(&self, id: usize, event: Event) {
        match self.collectors.get(&id) {
            Some(collector) => collector.handle_event(event, 0),
            None => todo!(), // scream loudly in the logs that we've got an event and don't know who it belongs to
        }
    }

    fn remove_collector(&mut self, id: usize) {
        self.collectors.remove(&id);
    }
}

impl Adapter for StdoutAdapter {
    fn handle_event(&mut self, event: Event) {
        // this is a littel screwy because the id is embeded in the event itself
        match event {
            Event::Func(id, _) => self._handle_event(id, event),
            Event::Alloc(id, _) => self._handle_event(id, event),
            Event::Metadata(id, _) => self._handle_event(id, event),
            Event::Shutdown(id) => self.remove_collector(id),
            Event::Stats(_stat) => todo!(),
        };
    }

    // flush any remaning spans
    fn shutdown(&self) {
        // close event stream and join the thread handle
        thread::sleep(time::Duration::from_millis(5));
    }
}

impl Collector {
    fn handle_event(&self, event: Event, depth: usize) {
        match event {
            Event::Func(_id, f) => {
                println!(
                    "{}{:?} Func: {}",
                    "\t".repeat(depth),
                    f.start.duration_since(self.start),
                    f.name.unwrap_or("unknown-name".to_string()),
                );
                for event in f.within.iter() {
                    self.handle_event(event.to_owned(), depth + 1);
                }
            }
            Event::Alloc(_id, a) => {
                println!("{:?} Alloc: {}", a.ts.duration_since(self.start), a.amount)
            }
            Event::Metadata(_id, _) => todo!(),
            Event::Stats(_stats) => todo!(),
            Event::Shutdown(_id) => {} //noop for now
        }
    }
}

fn next_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
