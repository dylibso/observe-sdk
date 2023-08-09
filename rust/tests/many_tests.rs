#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::process::Command;

    mod helpers;

    #[test]
    fn integration_many() -> Result<()> {
        // cargo run --example many ../test/test.c.instr.wasm
        #[cfg(feature = "async")]
        let features = "";
        #[cfg(not(feature = "async"))]
        let features = "--no-default-features";
        let output = Command::new("cargo")
            .args(&[
                "run",
                features,
                "--example",
                "many",
                "../test/test.c.instr.wasm",
            ])
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

        Ok(())
    }
}
