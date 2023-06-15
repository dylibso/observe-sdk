use serde_json::Value;

pub fn attribute_of_first_span(trace: &Value, attribute: String) -> Option<String> {
    if let Some(span) = spans(trace).first() {
        let value = span[attribute].as_str().unwrap_or_default();
        return Some(value.to_string());
    }
    None
}

pub fn ids_and_parent_span_ids(trace: &Value) -> (Vec<String>, Vec<String>) {
    let parent_ids = spans(trace)
        .iter()
        .map(|span| span["parentSpanId"].as_str().unwrap().to_string())
        .collect();
    let ids = spans(trace)
        .iter()
        .map(|span| span["spanId"].as_str().unwrap().to_string())
        .collect();
    (ids, parent_ids)
}

pub fn spans(trace: &Value) -> Vec<Value> {
    if let Some(resource_spans) = trace["resourceSpans"].as_array().unwrap().first() {
        if let Some(spans) = resource_spans["scopeSpans"].as_array().unwrap().first() {
            return spans["spans"].as_array().unwrap().to_vec();
        }
    }
    vec![]
}
