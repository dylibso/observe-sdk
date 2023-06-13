#[cfg(test)]
mod tests {
    use std::{convert::identity, process::Command};

    use anyhow::Result;
    use serde_json::Value;

    #[test]
    fn basic() -> Result<()> {
        // cargo run --example basic ../test/test.c.instr.wasm
        let output = Command::new("cargo")
            .args(&["run", "--example", "basic", "../test/test.c.instr.wasm"])
            .output()
            .expect("Failed to run the example `examples/basic`");

        let output = String::from_utf8(output.stdout)?;
        let output_lines = output.lines();

        // First test that the expected output was printed 10 times
        assert_eq!(
            output_lines
                .clone()
                .filter(|l| l.contains("Hello, world!"))
                .count(),
            10
        );

        // Check that printf was also called 10 times
        assert_eq!(
            output_lines
                .clone()
                .filter(|l| l.ends_with("Func: printf"))
                .count(),
            10
        );

        Ok(())
    }

    #[test]
    fn otel_stdout() -> Result<()> {
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

        let traces = output
            .lines()
            .map(|l| match serde_json::from_str(l) {
                Ok(x) => Some(x),
                Err(e) => None,
            })
            .collect::<Vec<Option<Value>>>()
            .into_iter()
            .filter_map(identity)
            .collect::<Vec<Value>>();

        let mut trace_id = "";
        if let Some(trace) = traces.first() {
            if let Some(resource_spans) = trace["resourceSpans"].as_array().unwrap().first() {
                if let Some(spans) = resource_spans["scopeSpans"].as_array().unwrap().first() {
                    if let Some(span) = spans["spans"].as_array().unwrap().first() {
                        trace_id = span["traceId"].as_str().unwrap_or_default();
                    }
                }
            }
        }

        assert_eq!(trace_id, "any-old-trace-id");
        Ok(())
    }
}
