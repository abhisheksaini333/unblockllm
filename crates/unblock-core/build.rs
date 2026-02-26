//! Generate Rust types from Protobuf schemas for internal comms.
//! No PII in generated code or at runtime.

use std::io::Result;

fn main() -> Result<()> {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_dir = manifest_dir.join("../../proto");
    let proto_file = proto_dir.join("redaction.proto");
    if !proto_file.exists() {
        return Ok(());
    }
    prost_build::compile_protos(&[proto_file], &[proto_dir])?;
    Ok(())
}
