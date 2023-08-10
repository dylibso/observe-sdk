include!(concat!(env!("OUT_DIR"), "/_includes.rs"));

use crate::adapter::otlp_formatter::opentelemetry::proto::common::v1::any_value::Value::StringValue;
use crate::new_span_id;
use std::time::SystemTime;

// #[derive(Default, Debug, Clone, PartialEq, Serialize)]
// #[serde(rename_all = "camelCase")]
pub struct OtelFormatter {
    // pub resource_spans: Vec<ResourceSpan>,
    pub traces_data: opentelemetry::proto::trace::v1::TracesData,
}

impl OtelFormatter {
    pub fn new(
        spans: Vec<opentelemetry::proto::trace::v1::Span>,
        service_name: String,
    ) -> OtelFormatter {
        OtelFormatter {
            traces_data: opentelemetry::proto::trace::v1::TracesData {
                resource_spans: new_resource_spans(spans, service_name),
            },
        }
    }

    pub fn new_span(
        trace_id: String,
        parent_id: Vec<u8>,
        name: String,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> opentelemetry::proto::trace::v1::Span {
        opentelemetry::proto::trace::v1::Span {
            trace_id: trace_id.into(),
            span_id: new_span_id().to_hex_8().as_bytes().to_vec(),
            parent_span_id: parent_id,
            name,
            kind: 1,
            start_time_unix_nano: start_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            end_time_unix_nano: end_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            attributes: Vec::new(),
            dropped_attributes_count: 0,
            dropped_events_count: 0,
            dropped_links_count: 0,
            status: None,
            events: vec![],
            links: vec![],
            trace_state: "".into(),
        }
    }
}

pub fn new_resource_spans(
    spans: Vec<opentelemetry::proto::trace::v1::Span>,
    service_name: String,
) -> Vec<opentelemetry::proto::trace::v1::ResourceSpans> {
    let scope_spans = vec![opentelemetry::proto::trace::v1::ScopeSpans {
        scope: None,
        spans,
        schema_url: "".into(),
    }];
    vec![opentelemetry::proto::trace::v1::ResourceSpans {
        resource: Some(opentelemetry::proto::resource::v1::Resource {
            attributes: vec![opentelemetry::proto::common::v1::KeyValue {
                key: "service.name".into(),
                value: Some(opentelemetry::proto::common::v1::AnyValue {
                    value: Some(StringValue(service_name.into())),
                }),
            }],
            dropped_attributes_count: 0,
        }),
        scope_spans,
        schema_url: "".into(),
    }]
}

// impl ResourceSpan {
//     pub fn new() -> ResourceSpan {
//         ResourceSpan {
//             scope_spans: vec![],
//             resource: Resource {
//                 attributes: Vec::new(),
//             },
//         }
//     }

//     pub fn add_attribute(mut self, key: String, value: String) -> ResourceSpan {
//         let attribute = Attribute {
//             key,
//             value: Value {
//                 string_value: Some(value),
//                 int_value: None,
//             },
//         };
//         self.resource.attributes.push(attribute);
//         self
//     }

//     pub fn add_spans(&mut self, spans: Vec<Span>) {
//         self.scope_spans = vec![ScopeSpan {
//             scope: Scope {
//                 name: "event".to_string(),
//             },
//             spans,
//         }]
//     }
// }

// impl Span {
//     pub fn new(
//         trace_id: String,
//         parent_id: Option<String>,
//         name: String,
//         start_time: SystemTime,
//         end_time: SystemTime,
//     ) -> Span {
//         Span {
//             trace_id,
//             span_id: new_span_id().to_hex_8(),
//             parent_span_id: parent_id,
//             name,
//             kind: 1,
//             start_time_unix_nano: start_time
//                 .duration_since(SystemTime::UNIX_EPOCH)
//                 .unwrap()
//                 .as_nanos(),
//             end_time_unix_nano: end_time
//                 .duration_since(SystemTime::UNIX_EPOCH)
//                 .unwrap()
//                 .as_nanos(),
//             attributes: Vec::new(),
//             dropped_attributes_count: 0,
//             dropped_events_count: 0,
//             dropped_links_count: 0,
//             status: Status {},
//         }
//     }

//     pub fn add_attribute_string(&mut self, key: String, value: String) {
//         self.attributes.push(Attribute {
//             key,
//             value: Value {
//                 string_value: Some(value),
//                 int_value: None,
//             },
//         });
//     }
//     pub fn add_attribute_i64(&mut self, key: String, value: i64) {
//         self.attributes.push(Attribute {
//             key,
//             value: Value {
//                 int_value: Some(value),
//                 string_value: None,
//             },
//         });
//     }
// }
