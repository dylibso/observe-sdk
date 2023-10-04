cargo_component_bindings::generate!();

use observe_api::*;

use bindings::Guest;

pub struct Component;

impl Guest for Component {
    /// Say hello!
    fn hello_world() -> u32 {
        log(log::Level::Info, "hello world");
        0xdeadbeef
    }
}
