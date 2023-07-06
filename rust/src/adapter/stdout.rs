use std::time::SystemTime;

use crate::{Event, TraceEvent};
use anyhow::{bail, Context, Result};

use super::{Adapter, AdapterHandle};

/// An adapter to send events from your module to stdout in our own format.
/// Mostly useful for debugging.
pub struct StdoutAdapter {}

impl Adapter for StdoutAdapter {
    fn handle_trace_event(&mut self, trace_evt: TraceEvent) -> Result<()> {
        let first = trace_evt
            .events
            .iter()
            .find(|&e| matches!(e, Event::Func(_f)))
            .context("Stdout adapter did not get any events")?;
        let start = match first {
            Event::Func(f) => f.start,
            _ => bail!("Stdout adapter is expecting at least one function"),
        };
        for span in trace_evt.events {
            self.handle_event(span, start, 0 as usize)?;
        }
        Ok(())
    }
}

impl StdoutAdapter {
    /// Creates the Otel Stdout adapter and spawns a task for it.
    /// This should ideally be created once per process of
    /// your rust application.
    pub fn create() -> AdapterHandle {
        Self::spawn(Self {})
    }

    fn handle_event(&self, event: Event, start: SystemTime, depth: usize) -> Result<()> {
        match event {
            Event::Func(f) => {
                println!(
                    "{}{:?} Func: {}",
                    "\t".repeat(depth),
                    f.start.duration_since(start),
                    f.name.unwrap_or("unknown-name".to_string()),
                );
                for e in f.within.iter() {
                    self.handle_event(e.to_owned(), start, depth + 1)?;
                }
            }
            Event::Alloc(a) => {
                println!("{:?} Alloc: {}", a.ts.duration_since(start), a.amount);
            }
            _ => {}
        }
        Ok(())
    }
}
