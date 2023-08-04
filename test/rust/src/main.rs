use log::Level;
use observe_api::{instrument, log, metric, span_enter, span_exit, span_tags, MetricFormat};

#[instrument]
fn log_something(msg: &str) {
    metric(MetricFormat::Statsd, "worlds.helloed:1|c");
    log(Level::Warn, msg);
}

#[instrument]
fn run() {
    log_something("Hello World 1");
    log_something("Hello World 2");
    log_something("Hello World 3");
}

fn main() {
    span_tags(vec!["user_id:123", "world:hello"]);
    run();
}

