mod bindings;
use observe_api::*;

fn main() {
    span_enter("hello world");
    log(log::Level::Info, "hello world");
    span_exit();
    println!("Hello, world!");
}
