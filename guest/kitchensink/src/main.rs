use log::Level;
use observe_api::{log, metric, span_enter, span_exit, span_tags, MetricFormat};
use observe_instrument::instrument;

#[instrument]
fn log_something(msg: &str) {
    span_tags(vec!["user_id:123", "world:hello"]);
    metric(MetricFormat::Statsd, "worlds.helloed:1|c");
    log(Level::Warn, msg);
}

#[instrument]
fn run() {
    log_something("Hello World 1");
    log_something("Hello World 2");
    log_something("Hello World 3");
}

#[instrument]
fn main() {
    span_tags(vec!["user_id:123", "world:hello"]);
    run();
}
