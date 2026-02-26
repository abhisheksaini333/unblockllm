# ONNX NER Model Benchmark Report (Phase 1)

## Plan Summary (Phase 1: Foundation & Architecture)

- **Monorepo:** Cargo workspace (Rust), Python package (`unblockllm`), Next.js app (`apps/dashboard`), shared `proto/` and `docs/`.
- **Protobuf:** `proto/redaction.proto` for RedactionRequest, RedactionResult, EntitySpan, MaskStrategy, EntityType; Rust codegen via `prost` in `unblock-core`.
- **ONNX NER:** Primary model **protectai/bert-base-NER-onnx** (HuggingFace link below); benchmark script in Python with &lt;5ms target.
- **CI/CD:** GitHub Actions workflow (Rust: fmt, clippy, build, test, cargo-audit; Python: install, ruff, pytest, NER benchmark; Dashboard: npm lint/build).

## File Structure (Phase 1)

```
unblockllm/
├── Cargo.toml                 # Workspace: unblock-proxy, unblock-core
├── LICENSE
├── README.md
├── .github/
│   └── workflows/
│       └── ci.yml              # CI: Rust, Python, Dashboard
├── apps/
│   └── dashboard/              # Next.js (Phase 5)
│       ├── package.json
│       ├── src/
│       │   ├── app/
│       │   │   ├── globals.css
│       │   │   └── layout.tsx
│       │   └── ...
│       └── ...
├── crates/
│   ├── unblock-core/          # Types, errors, proto codegen
│   │   ├── Cargo.toml
│   │   ├── build.rs            # prost-build: proto/redaction.proto
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       └── types.rs
│   └── unblock-proxy/         # HTTP proxy (health in Phase 1)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs        # /health endpoint
├── docs/
│   └── MODEL_BENCHMARK_REPORT.md
├── proto/
│   ├── README.md
│   └── redaction.proto
└── python/
    ├── pyproject.toml
    ├── README.md
    ├── unblockllm/
    │   ├── __init__.py
    │   ├── benchmark/
    │   │   ├── __init__.py
    │   │   └── ner_benchmark.py
    │   └── cli/
    │       ├── __init__.py
    │       └── scan.py
    └── tests/
        └── test_benchmark.py
```

## Objective

Select an ONNX NER model and verify **inference latency <5ms** per request on typical hardware, so that total proxy overhead (mask + unmask) stays **<20ms**.

## Model selection

| Model | HuggingFace | Notes |
|-------|-------------|--------|
| **Primary** | [protectai/bert-base-NER-onnx](https://huggingface.co/protectai/bert-base-NER-onnx) | Pre-exported ONNX; CoNLL-2003 (PER, ORG, LOC, MISC). |
| **Fallback** | [dslim/bert-base-NER](https://huggingface.co/dslim/bert-base-NER) | Same base; loaded via Optimum `ORTModelForTokenClassification`. |

- **Entity types:** PERSON, ORGANIZATION, LOCATION, MISC (plus regex for EMAIL, PHONE, SSN in Phase 2).
- **Runtime:** ONNX Runtime (Python: `optimum[onnxruntime]`, Rust: `ort` in Phase 2).

## Benchmark script

- **Path:** `python/unblockllm/benchmark/ner_benchmark.py`
- **Usage:**
  ```bash
  cd python && pip install -e ".[dev]" && python -m unblockllm.benchmark.ner_benchmark --runs 100 --output benchmark_report.json
  ```
- **Output:** `benchmark_report.json` with mean/median/p95/p99 latency (ms); **no PII** in report or logs.
- **Target:** Mean latency <5ms. If exceeded, Phase 2 will consider quantized (INT8) or smaller models (e.g. DistilBERT NER).

## Verification commands (Phase 1)

Run from repository root (`unblockllm/`):

```bash
# 1. Rust
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo build --release && cargo test

# 2. Python
cd python && pip install -e ".[dev]" && ruff check . && ruff format --check . && pytest -v

# 3. Benchmark (after pip install)
cd python && python -m unblockllm.benchmark.ner_benchmark --runs 50 --output benchmark_report.json

# 4. Dashboard
cd apps/dashboard && npm install && npm run lint && npm run build

# 5. Proxy health (after cargo build, from repo root)
./target/release/unblock-proxy &
curl -s http://127.0.0.1:8080/health
# expect: ok
kill %1
```

## Security check (Phase 1)

- **Benchmark:** Uses **synthetic sentences only**; no real PII in repo or logs.
- **Report:** Contains **aggregate latency stats** only (mean/median/p95/p99); no user input or model outputs.
- **CI:** Does **not** log raw request/response bodies; only pass/fail and counts.
- **Secrets:** No keys or credentials in repo; all via environment variables (enforced in later phases).
- **Proto:** Redaction messages carry only request_id, offsets, types, and masked text—no raw PII in serialized form.

---

**Phase 1 deliverable:** Repo structure, Protobuf, CI, benchmark script, and this report.  
**Next:** Phase 2 — Core Proxy Development (intercept, mask, ONNX integration).
