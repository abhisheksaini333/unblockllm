---
name: unblockllm
description: Zero-Trust Privacy Proxy for LLMs. Use when editing or extending unblockllm (Rust proxy, Python SDK, ONNX NER, Next.js dashboard), implementing PII redaction, adding compliance features, or following the project's phase plan. Apply security and latency constraints: no PII in logs, no unwrap in production, &lt;20ms overhead.
---

# unblockllm — Cursor Agent Skill

Context and rules for working in the **unblockllm** monorepo: Zero-Trust Privacy Proxy that redacts PII locally (ONNX NER) before sending prompts to public LLMs.

## When to Use This Skill

- Editing or adding code in `unblockllm` (any crate, Python package, or dashboard).
- Implementing or changing PII redaction, masking, or re-identification.
- Adding compliance-related behavior (audit logs, policy engine, encryption).
- Running or modifying tests, benchmarks, or CI.
- Answering questions about repo layout, phases, or constraints.

## Repo Layout

| Area | Path | Purpose |
|------|------|---------|
| Rust workspace | `Cargo.toml`, `crates/` | Proxy (`unblock-proxy`), core types/proto (`unblock-core`) |
| Protobuf | `proto/redaction.proto` | RedactionRequest, RedactionResult, EntitySpan; Rust codegen in unblock-core `build.rs` |
| Python | `python/`, `python/unblockllm/` | SDK, `unblockllm.benchmark.ner_benchmark`, CLI `unblockllm-scan` |
| Dashboard | `apps/dashboard/` | Next.js (Phase 5) |
| CI | `.github/workflows/ci.yml` | Rust, Python, Dashboard jobs |
| Docs | `docs/`, `README.md` | MODEL_BENCHMARK_REPORT.md, architecture |

## Non-Negotiable Constraints

1. **Privacy:** Never log raw PII to stdout, stderr, or DB. Log hashes or counts only.
2. **Security:** No hardcoded secrets; use environment variables. TLS enforced where applicable.
3. **Rust:** No `unwrap()` in production code; use `?`, `expect()` with clear messages, or explicit handling.
4. **Latency:** Total mask + unmask overhead target **&lt;20ms**; NER inference target **&lt;5ms**.
5. **Types:** Explicit error handling; no placeholders or mocks in production paths.

## Key Conventions

- **Rust:** `unblock-core` holds types, errors, proto; `unblock-proxy` is the HTTP proxy (Axum). Build from repo root: `cargo build --release`, `cargo test`.
- **Python:** Editable install `pip install -e ".[dev]"` from `python/`. Benchmark: `python -m unblockllm.benchmark.ner_benchmark --runs 50 --output benchmark_report.json`. Use synthetic data only in benchmarks/tests.
- **ONNX NER:** Primary model `protectai/bert-base-NER-onnx` (HuggingFace). Fallback `dslim/bert-base-NER` via Optimum.
- **Proto:** Only request_id, offsets, entity types, and masked text in messages—no raw PII in serialized form.

## Verification Commands (from repo root)

```bash
# Rust
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo build --release && cargo test

# Python
cd python && pip install -e ".[dev]" && ruff check . && ruff format --check . && pytest -v

# NER benchmark (<5ms target)
cd python && python -m unblockllm.benchmark.ner_benchmark --runs 50 --output benchmark_report.json
# Rust benches/latency.rs measures mask+reidentify only (regex path); total <20ms = NER + mask + reid + network.

# Dashboard
cd apps/dashboard && npm install && npm run lint && npm run build

# Proxy health (run proxy in background, then curl)
./target/release/unblock-proxy &
curl -s http://127.0.0.1:8080/health
```

## Phase Plan (reference)

| Phase | Focus |
|-------|--------|
| 1 | Foundation: monorepo, proto, ONNX benchmark, CI |
| 2 | Core proxy: intercept OpenAI API, streaming SSE, ONNX NER, masking |
| 3 | State & compliance: Redis mapping, PostgreSQL audit (metadata only), policy engine, encryption at rest |
| 4 | DX: Python/Node SDKs, CLI `unblockllm-scan`, docs |
| 5 | Control plane: Next.js dashboard, Stripe, remote config |
| 6 | Hardening: pen test, k6 load (&lt;20ms overhead, 1000 RPS), cargo-audit / safety |
| 7 | Launch: Docker, landing page, SOC2 readiness |

## Instructions for Common Tasks

### Adding or changing Rust code

- Prefer `Result` and `?`; avoid `unwrap()` in library or long-lived code.
- Run `cargo clippy --all-targets -- -D warnings` and fix lints.
- Add or update unit tests in the same crate; integration tests for proxy flow in `unblock-proxy`.

### Adding or changing Python code

- Use type hints; no PII in log messages or test data.
- Run `ruff check .` and `ruff format --check .` in `python/`.
- Benchmark script must only use synthetic sentences and aggregate stats in the report.

### Changing Protobuf

- Edit `proto/redaction.proto`; run `cargo build` so `unblock-core` regenerates via `build.rs`. Do not add fields that could carry raw PII.

### Adding a new phase or deliverable

- Align with the phase list above; update `docs/` and README if the repo structure or verification steps change.

## Examples

**User:** "Add a new entity type for credit card in the proto."
- Edit `proto/redaction.proto` (add enum value and any doc).
- Update `crates/unblock-core/src/types.rs` to match (e.g. `EntityType` and any mapping).
- Re-run `cargo build` and tests. Do not log or serialize raw card numbers.

**User:** "Run the NER benchmark and check latency."
- From repo root: `cd python && python -m unblockllm.benchmark.ner_benchmark --runs 50 --output benchmark_report.json`. Interpret mean/p95 vs &lt;5ms target; no PII in report.

**User:** "Where do we mask PII before sending to the LLM?"
- Masking is implemented in Phase 2 in the proxy: intercept request → run NER (ONNX) + regex → replace with placeholders → forward. Re-identification uses ephemeral token mapping (Phase 3). Point to `crates/unblock-proxy` and `unblock-core` types.

## Additional Resources

- Full benchmark and Phase 1 layout: [docs/MODEL_BENCHMARK_REPORT.md](docs/MODEL_BENCHMARK_REPORT.md)
- Proto usage: [proto/README.md](proto/README.md)
- Python usage: [python/README.md](python/README.md)
