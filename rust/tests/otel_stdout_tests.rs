#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::process::Command;

    #[test]
    fn otel_stdout() -> Result<()> {
        // cargo run --example many ../test/test.c.instr.wasm
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--example",
                "otel-stdout",
                "../test/test.c.instr.wasm",
            ])
            .output()
            .expect("Failed to run the example `examples/otel-stdout`");

        let output = String::from_utf8(output.stdout)?;
        let output_lines = output.lines();
        let start_fns = output_lines
            .clone()
            .filter(|l| l.contains("_start"))
            .count();

        // First test that the modules ran the expected number of times
        assert_eq!(start_fns, 1);

        Ok(())
    }
}
