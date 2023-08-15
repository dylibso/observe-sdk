use std::io::Result;

fn main() -> Result<()> {
    let mut prost_build = prost_build::Config::new();
    prost_build.include_file("_includes.rs");
    prost_build.compile_protos(
        &["../proto/opentelemetry/proto/trace/v1/trace.proto"],
        &["../proto/"],
    )?;
    Ok(())
}
