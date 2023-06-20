use std::time::SystemTime;

use serde::{Serialize, Serializer};

use super::new_span_id;

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtelFormatter {
    pub resource_spans: Vec<ResourceSpan>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSpan {
    pub resource: Resource,
    pub scope_spans: Vec<ScopeSpan>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub attributes: Vec<Attribute>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub key: String,
    pub value: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub string_value: Option<String>,
    pub int_value: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeSpan {
    pub scope: Scope,
    pub spans: Vec<Span>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    #[serde(serialize_with = "default_parent_span_id")]
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: i64,
    pub start_time_unix_nano: u128,
    pub end_time_unix_nano: u128,
    pub attributes: Vec<Attribute>,
    pub dropped_attributes_count: i64,
    pub dropped_events_count: i64,
    pub dropped_links_count: i64,
    pub status: Status,
}

fn default_parent_span_id<S>(psi: &Option<String>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match psi {
        Some(x) => s.serialize_str(x.as_str()),
        None => s.serialize_str(""),
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Value2 {
    pub string_value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize)]
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
                string_value: Some(value),
                int_value: None,
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
            spans,
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
        Span {
            trace_id,
            span_id: new_span_id().to_hex_8(),
            parent_span_id: parent_id,
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

    pub fn add_attribute_string(&mut self, key: String, value: String) {
        self.attributes.push(Attribute {
            key,
            value: Value {
                string_value: Some(value),
                int_value: None,
            },
        });
    }
    pub fn add_attribute_i64(&mut self, key: String, value: i64) {
        self.attributes.push(Attribute {
            key,
            value: Value {
                int_value: Some(value),
                string_value: None,
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn build_resource() {
        let rs =
            ResourceSpan::new().add_attribute("service.name".to_owned(), "something".to_owned());
        let attrib = rs.resource.attributes.first().unwrap();
        assert_eq!(attrib.value.string_value.as_ref().unwrap(), "something");
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

    #[test]
    fn new_span_parent_id() {
        let mut span = Span::new(
            "trace-id".to_string(),
            None,
            "name".to_string(),
            SystemTime::now(),
            SystemTime::now(),
        );

        assert_eq!(span.parent_span_id, None);
        let json = serde_json::to_string(&span).unwrap();
        assert!(json.contains("parentSpanId\":\"\""));

        span.parent_span_id = Some("abcd".to_string());
        let json = serde_json::to_string(&span).unwrap();
        assert!(json.contains("parentSpanId\":\"abcd\""));
    }
}
