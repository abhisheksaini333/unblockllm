# Protobuf schemas

Internal communication between proxy, state store, and audit components.

- `redaction.proto` – redaction request/response and entity spans.
- Generated code: Rust via `prost` (in crates), Python via `grpc_tools` (in python package).

Generate (from repo root):

```bash
# Rust: prost-build in build.rs of crates that need it
cargo build

# Python (optional, for gRPC clients later)
cd python && python -m grpc_tools.protoc -I../proto --python_out=. --pyi_out=. ../proto/*.proto
```
