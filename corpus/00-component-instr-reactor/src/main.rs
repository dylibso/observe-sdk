cargo_component_bindings::generate!();

use observe_api::*;

use bindings::Guest;

pub struct Component;

impl Guest for Component {
    /// Say hello!
    fn hello_world() -> u32 {
        span_enter("hello world");
        log(log::Level::Info, "hello world");
        span_exit();
        0xdeadbeef
    }
}

fn main() {
    log(log::Level::Info, "hello world");
    println!("Hello, world!");
}
