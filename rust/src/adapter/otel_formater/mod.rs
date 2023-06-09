use std::alloc::System;
use std::time::SystemTime;

use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtelFormatter {
    pub resource_spans: Vec<ResourceSpan>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSpan {
    pub resource: Resource,
    pub scope_spans: Vec<ScopeSpan>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub attributes: Vec<Attribute>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub key: String,
    pub value: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub string_value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeSpan {
    pub scope: Scope,
    pub spans: Vec<Span>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: String,
    pub name: String,
    pub kind: i64,
    pub start_time_unix_nano: u128,
    pub end_time_unix_nano: u128,
    pub attributes: Vec<Attribute2>,
    pub dropped_attributes_count: i64,
    pub dropped_events_count: i64,
    pub dropped_links_count: i64,
    pub status: Status,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute2 {
    pub key: String,
    pub value: Value2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value2 {
    pub string_value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {}

impl OtelFormatter {
    pub fn new() -> OtelFormatter {
        OtelFormatter {
            resource_spans: Vec::new(),
        }
    }

    pub fn add_resource_span(&mut self, rs: ResourceSpan) {
        self.resource_spans.push(rs)
    }
}

impl ResourceSpan {
    pub fn new() -> ResourceSpan {
        ResourceSpan {
            scope_spans: vec![],
            resource: Resource {
                attributes: Vec::new(),
            },
        }
    }

    pub fn add_attribute(mut self, key: String, value: String) -> ResourceSpan {
        let attribute = Attribute {
            key,
            value: Value {
                string_value: value,
            },
        };
        self.resource.attributes.push(attribute);
        self
    }

    pub fn add_spans(&mut self, spans: Vec<Span>) {
        self.scope_spans = vec![ScopeSpan {
            scope: Scope {
                name: "event".to_string(),
            },
            spans: spans,
        }]
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
        let span_id = new_span_id();

        let p_id = match parent_id {
            Some(id) => id,
            None => new_span_id(),
        };

        Span {
            trace_id,
            span_id,
            parent_span_id: p_id,
            name,
            kind: 1,
            start_time_unix_nano: start_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            end_time_unix_nano: end_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            attributes: Vec::new(),
            dropped_attributes_count: 0,
            dropped_events_count: 0,
            dropped_links_count: 0,
            status: Status {},
        }
    }
}
fn new_span_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn build_resource() {
        let rs =
            ResourceSpan::new().add_attribute("service.name".to_owned(), "something".to_owned());
        assert_eq!(
            rs.resource.attributes.first().unwrap().value.string_value,
            "something"
        );
    }

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
