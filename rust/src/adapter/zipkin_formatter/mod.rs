use serde::Serialize;
use std::collections::HashMap;
use std::time::SystemTime;

use crate::new_span_id;

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZipkinFormatter {
    pub spans: Vec<Span>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalEndpoint {
    pub service_name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub id: String,
    pub trace_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub timestamp: u64,
    pub duration: u64,
    pub tags: HashMap<String, String>,
    pub local_endpoint: Option<LocalEndpoint>,
}

impl ZipkinFormatter {
    pub fn new() -> ZipkinFormatter {
        ZipkinFormatter { spans: vec![] }
    }
}

impl Span {
    pub fn new(
        trace_id: String,
        parent_id: Option<String>,
        name: String,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Span {
        let id = new_span_id().to_hex_8();

        Span {
            id,
            trace_id,
            parent_id,
            name,
            kind: "SERVER".to_string(),
            timestamp: start_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
            duration: end_time.duration_since(start_time).unwrap().as_micros() as u64,
            tags: HashMap::new(),
            local_endpoint: None,
        }
    }

    pub fn add_tag_i64(&mut self, name: String, value: i64) {
        self.tags.insert(name, value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn build_span() {
        let start = SystemTime::now()
            .checked_sub(Duration::new(0, 1500))
            .unwrap();
        let end = SystemTime::now();
        let name = "function-call-start".to_string();

        let span = Span::new(
            "1234abcd".to_string(),
            Some("f00df0b0b0".to_string()),
            name.clone(),
            start,
            end,
        );

        assert_eq!(span.name, name);
    }
}
