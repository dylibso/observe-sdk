include!(concat!(env!("OUT_DIR"), "/_includes.rs"));

use crate::adapter::otlp_formatter::opentelemetry::proto::common::v1::any_value::Value::{
    IntValue, StringValue,
};
use crate::new_span_id;
use std::time::SystemTime;

pub struct OtelFormatter {
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

    pub fn add_attribute_i64_to_span(
        span: &mut opentelemetry::proto::trace::v1::Span,
        key: String,
        value: i64,
    ) {
        let attr = opentelemetry::proto::common::v1::KeyValue {
            key: key.into(),
            value: Some(opentelemetry::proto::common::v1::AnyValue {
                value: Some(IntValue(value)),
            }),
        };
        span.attributes.push(attr)
    }

    pub fn add_attribute_string_to_span(
        span: &mut opentelemetry::proto::trace::v1::Span,
        key: String,
        value: String,
    ) {
        let attr = opentelemetry::proto::common::v1::KeyValue {
            key: key.into(),
            value: Some(opentelemetry::proto::common::v1::AnyValue {
                value: Some(StringValue(value)),
            }),
        };
        span.attributes.push(attr)
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
