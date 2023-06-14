#[cfg(test)]
mod tests {
    use std::convert::identity;
    use std::process::Command;

    use anyhow::Result;
    use serde_json::Value;

    #[test]
    fn otel_stdout() -> Result<()> {
        // cargo run --example otel-stdout ../test/test.c.instr.wasm
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--example",
                "otel-stdout",
                "../test/test.c.instr.wasm",
            ])
            .output()
            .expect("Failed to run the example `examples/basic`");

        let output = String::from_utf8(output.stdout)?;

        // traces is the collection of all traces emitted from this run
        let traces = output
            .lines()
            .map(|l| match serde_json::from_str(l) {
                Ok(x) => Some(x),
                Err(_) => None,
            })
            .collect::<Vec<Option<Value>>>()
            .into_iter()
            .filter_map(identity)
            .collect::<Vec<Value>>();

        let trace_id = attribute_of_first_span(traces.first().unwrap(), "traceId".to_string());
        assert_eq!(trace_id, Some("any-old-trace-id".to_string()));

        // test.c.instr.wasm spits out 10 allocations at the top level, this may change with the
        // function naming work
        let allocations = traces
            .iter()
            .filter(|t| attribute_of_first_span(t, "name".to_string()).unwrap() == "allocation");
        assert_eq!(allocations.count(), 10);

        // We know the 11th trace emitted is the first function call
        let (ids, parent_ids) = ids_and_parent_span_ids(traces.get(11).unwrap());

        // the first span won't have a parent in our list
        for parent_id in &parent_ids[1..] {
            println!("--- testing {} ", parent_id);
            assert!(ids.contains(&parent_id));
        }

        Ok(())
    }

    fn attribute_of_first_span(trace: &Value, attribute: String) -> Option<String> {
        if let Some(span) = spans(trace).first() {
            let value = span[attribute].as_str().unwrap_or_default();
            return Some(value.to_string());
        }
        None
    }

    fn ids_and_parent_span_ids(trace: &Value) -> (Vec<String>, Vec<String>) {
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

    fn spans(trace: &Value) -> Vec<Value> {
        if let Some(resource_spans) = trace["resourceSpans"].as_array().unwrap().first() {
            if let Some(spans) = resource_spans["scopeSpans"].as_array().unwrap().first() {
                return spans["spans"].as_array().unwrap().to_vec();
            }
        }
        vec![]
    }
}
