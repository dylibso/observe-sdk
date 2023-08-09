#[cfg(test)]
mod tests {
    use std::process::Command;

    use anyhow::Result;

    #[test]
    fn integration_basic() -> Result<()> {
        // cargo run --example basic ../test/test.c.instr.wasm
        #[cfg(feature = "async")]
        let features = "";
        #[cfg(not(feature = "async"))]
        let features = "--no-default-features";

        let output = Command::new("cargo")
            .args(&[
                "run",
                features,
                "--example",
                "basic",
                "../test/test.c.instr.wasm",
            ])
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
}
