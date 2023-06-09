#[cfg(test)]
mod tests {
    use std::convert::identity;
    use std::process::Command;

    use anyhow::Result;
    use serde_json::Value;

    mod helpers;
    use helpers::otel_json::*;

    #[test]
    fn otel_stdout() -> Result<()> {
        // cargo run --example otel-stdout ../test/test.c.instr.wasm 'Test'
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--example",
                "otel-stdout",
                "../test/test.c.instr.wasm",
                "'Test'",
            ])
            .output()
            .expect("Failed to run the example `examples/otel-stdout`");

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

        // let trace_id = attribute_of_first_span(traces.first().unwrap(), "traceId".to_string());
        // assert_eq!(trace_id.unwrap().len(), 32); // TODO freeze random seed or pass in known value

        // // test.c.instr.wasm spits out ? allocations at the top level, this may change with the
        // // function naming work
        // let allocations = traces
        //     .iter()
        //     .filter(|t| attribute_of_first_span(t, "name".to_string()).unwrap() == "allocation");
        // assert_eq!(allocations.count(), 1);

        Ok(())
    }
}
