#[link(wasm_import_module = "dylibso_observe")]
extern "C" {
    #[link_name = "metric"]
    fn _metric(format: u32, ptr: u64, len: u32);
    #[link_name = "log"]
    fn _log(level: u32, ptr: u64, len: u32);
    #[link_name = "span_enter"]
    fn _span_enter(ptr: u64, len: u32);
    #[link_name = "span_tags"]
    fn _span_tags(ptr: u64, len: u32);
    #[link_name = "span_exit"]
    fn _span_exit();
}

pub fn log(level: log::Level, message: &str) {
    let level = level as u32;
    let ptr = message.as_ptr() as *const u8;
    let ptr = ptr as u64;
    let len = message.len() as u32;
    unsafe { _log(level, ptr, len) };
}

pub enum MetricFormat {
    Statsd = 1,
}

pub fn metric(format: MetricFormat, message: &str) {
    let format = format as u32;
    let ptr = message.as_ptr() as *const u8;
    let ptr = ptr as u64;
    let len = message.len() as u32;
    unsafe { _metric(format, ptr, len) };
}

pub fn span_enter(name: &str) {
    let ptr = name.as_ptr() as *const u8;
    let ptr = ptr as u64;
    let len = name.len() as u32;
    unsafe { _span_enter(ptr, len) };
}

pub fn span_tags(tags: Vec<&str>) {
    let tags = tags.join(",");
    let ptr = tags.as_ptr() as *const u8;
    let ptr = ptr as u64;
    let len = tags.len() as u32;
    unsafe { _span_tags(ptr, len) };
}

pub fn span_exit() {
    unsafe { _span_exit() };
}
