cargo_component_bindings::generate!();

use observe_api::*;

use bindings::Guest;

struct Component;

impl Guest for Component {
    /// Say hello!
    fn hello_world() -> String {
        span_enter("hello world");
        log(log::Level::Info, "hello world");
        span_exit();
        "Hello, World!".to_string()
    }
}
