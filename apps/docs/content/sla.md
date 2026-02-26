# Service Level Agreement (SLA) — Overhead Targets

unblockllm proxy adds redaction overhead (mask + reidentify). Targets depend on the detection path.

## Detection Modes

| Mode | Description | Overhead SLA |
|------|-------------|--------------|
| **Regex** | Pattern-based only (no ONNX). Used when `NER_MODEL_DIR` is unset or regex-only config. | **<5ms** (p95) |
| **ONNX** | NER model inference (e.g. `protectai/bert-base-NER-onnx`) plus regex. | **<50ms** (p95) |

## Rationale

- **Regex:** Rust bench `benches/latency.rs` measures mask + reidentify only; typical mean ~0.4ms. SLA <5ms allows headroom for I/O and variance.
- **ONNX:** Phase 2 benchmark (Python `ner_benchmark`) reported ~14.28ms mean for NER inference. SLA <50ms accounts for model load, batch size, and hardware variance. **We do not promise <20ms for ONNX mode.**

## Total Request Overhead

End-to-end latency also includes:

- Redis (mapping store) and/or Postgres (audit) when enabled
- Network to upstream LLM

These are outside the redaction-overhead SLA above.

## Verification

- Regex: `cargo bench --bench latency` (mask+reid only).
- ONNX: `python -m unblockllm.benchmark.ner_benchmark --runs 50 --output benchmark_report.json`.
