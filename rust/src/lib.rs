use adapter::AdapterMetadata;
use rand::Rng;
use std::time::SystemTime;

pub mod adapter;
pub mod collector;
pub mod context;
pub mod wasm_instr;

#[derive(Debug, Clone)]
pub struct TelemetryId(u128);

impl TelemetryId {
    /// format as 8-byte zero-prefixed hex string
    pub fn to_hex_8(&self) -> String {
        // 8 bytes is 16 chars
        format!("{:016x}", self.0 as u64)
    }
    /// format as 16-byte zero-prefixed hex string
    pub fn to_hex_16(&self) -> String {
        // 16 bytes is 32 chars
        format!("{:032x}", self.0)
    }
}

impl From<TelemetryId> for u64 {
    fn from(v: TelemetryId) -> Self {
        v.0 as u64
    }
}

impl From<TelemetryId> for u128 {
    fn from(v: TelemetryId) -> Self {
        v.0
    }
}

pub fn new_trace_id() -> TelemetryId {
    let x = rand::thread_rng().gen::<u128>();
    TelemetryId(x)
}

pub fn new_span_id() -> TelemetryId {
    let x = rand::thread_rng().gen::<u128>();
    TelemetryId(x)
}

#[derive(Debug, Clone)]
pub enum Event {
    Func(FunctionCall),
    Alloc(Allocation),
    TraceId(TelemetryId),
    Metadata(AdapterMetadata),
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct TraceEvent {
    events: Vec<Event>,
    telemetry_id: TelemetryId,
    metadata: Option<AdapterMetadata>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: Option<String>,
    pub raw_name: Option<String>,
    pub index: u32,
    pub start: SystemTime,
    pub end: SystemTime,
    pub within: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Allocation {
    pub ts: SystemTime,
    pub amount: u32,
}
