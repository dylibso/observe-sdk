#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::Value;
    use std::convert::identity;
    use std::process::Command;

    mod helpers;
    use helpers::otel_json::*;

    #[test]
    fn integration_many() -> Result<()> {
        // cargo run --example many ../test/test.c.instr.wasm
        let output = Command::new("cargo")
            .args(&["run", "--example", "many", "../test/test.c.instr.wasm"])
            .output()
            .expect("Failed to run the example `examples/basic`");

        let output = String::from_utf8(output.stdout)?;
        let output_lines = output.lines();
        let hellos = output_lines
            .clone()
            .filter(|l| l.contains("Hello, world!"))
            .count();

        // First test that the modules ran the expected number of times
        assert_eq!(hellos, 250);

        // check that every allocation was called
        let traces = output_lines
            .map(|l| match serde_json::from_str(l) {
                Ok(x) => Some(x),
                Err(_) => None,
            })
            .collect::<Vec<Option<Value>>>()
            .into_iter()
            .filter_map(identity)
            .collect::<Vec<Value>>();
        let allocations = traces
            .iter()
            .filter(|t| attribute_of_first_span(t, "name".to_string()).unwrap() == "allocation")
            .count();
        assert_eq!(allocations > 10, true);
        Ok(())
    }
}
