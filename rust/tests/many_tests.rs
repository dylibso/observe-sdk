#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::process::Command;
    use std::thread;
    use std::time;

    #[test]
    fn integration_many() -> Result<()> {
        // cargo run --example many ../test/test.c.instr.wasm
        let output = Command::new("cargo")
            .args(&["run", "--example", "many", "../test/test.c.instr.wasm"])
            .output()
            .expect("Failed to run the example `examples/basic`");
        thread::sleep(time::Duration::from_millis(150));
        let output = String::from_utf8(output.stdout)?;
        let output_lines = output.lines();
        println!("{}", output);
        // First test that the expected output was printed 10 times
        assert_eq!(
            output_lines
                .clone()
                .filter(|l| l.contains("Hello, world!"))
                .count(),
            1000
        );

        // Check that printf was also called 10 times
        assert_eq!(
            output_lines
                .clone()
                .filter(|l| l.contains("printf"))
                .count(),
            100
        );

        Ok(())
    }
}
