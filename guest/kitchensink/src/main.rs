use log::Level;
use observe_api::{log, span_enter, span_exit, statsd};
use observe_instrument::instrument;

#[instrument]
fn log_something(msg: &str) {
    statsd("worlds.helloed:1|c");
    log(Level::Warn, msg);
}

#[instrument]
fn run() {
    log_something("Hello World 1");
    log_something("Hello World 2");
    log_something("Hello World 3");
}

fn main() {
    span_enter("main");
    run();
    span_exit();
}

