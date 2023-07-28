#[link(wasm_import_module = "dylibso_observe")]
extern "C" {
    #[link_name = "statsd"]
    fn _statsd(ptr: u32, len: u32);
    #[link_name = "log"]
    fn _log(level: u32, ptr: u32, len: u32);
    #[link_name = "span_enter"]
    fn _span_enter(ptr: u32, len: u32);
    #[link_name = "span_exit"]
    fn _span_exit();
}

pub fn log(level: log::Level, message: &str) {
    let level = level as u32;
    let ptr = message.as_ptr() as *const u8;
    let ptr = ptr as u32;
    let len = message.len() as u32;
    unsafe { _log(level, ptr, len) };
}

pub fn statsd(message: &str) {
    let ptr = message.as_ptr() as *const u8;
    let ptr = ptr as u32;
    let len = message.len() as u32;
    unsafe { _statsd(ptr, len) };
}

pub fn span_enter(name: &str) {
    let ptr = name.as_ptr() as *const u8;
    let ptr = ptr as u32;
    let len = name.len() as u32;
    unsafe { _span_enter(ptr, len) };
}

pub fn span_exit() {
    unsafe { _span_exit() };
}
