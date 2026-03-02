# UnblockLLM Baseline Validation & Gap Remediation Design

**Date:** 2026-03-02
**Author:** Cascade + Abhishek
**Status:** Approved

## Overview

Full-scope validation and improvement of the unblockllm monorepo across three phases.

## Phase 1: Validate Current State (this session)

### 1.1 Service Startup
- Docker infra (Redis + Postgres)
- Rust proxy (`cargo run -p unblock-proxy`)
- Next.js Dashboard (`apps/dashboard`, port 3000)
- Next.js Landing (`apps/landing`, port 3001)

### 1.2 Existing Test Suites
- Rust: `cargo test`, `cargo fmt --check`, `cargo clippy`, `cargo bench --bench latency`
- Python: `pip install -e ".[dev]"`, `ruff check`, `pytest -v`, NER benchmark
- Dashboard: `npm run lint`, `npm run build`
- Landing: `npm run build`

### 1.3 Live E2E Validation
- Non-streaming PII redaction through proxy
- Streaming PII redaction
- Policy block enforcement
- Rate limiting
- Audit log verification (Postgres)
- Dashboard auth flow

### 1.4 Baseline Report
Produce `docs/plans/2026-03-02-baseline-test-report.md` with pass/fail evidence.

## Phase 2: Fix Gaps (next session)

### Gap 1: ONNX NER not wired into Rust proxy
- Integrate `ort` crate into `unblock-proxy`
- Load `protectai/bert-base-NER-onnx` model when `NER_MODEL_DIR` is set
- Tokenize input, run ONNX inference, map NER labels to `EntityType`
- Fallback to regex when model not available

### Gap 2: Single commit history
- Establish branch strategy (main + feature branches)
- Create proper PRs for gap fixes

### Gap 3: No real battle-testing evidence
- Execute k6 load tests from `security/load-test/`
- Capture and commit results

### Gap 4: Thin Python SDK
- Add client-side PII scanning before sending to proxy
- Add retry logic, better error handling

### Gap 5: Protobuf not used
- Evaluate whether to wire proto for inter-service comms or remove dead code

## Phase 3: Comprehensive Testing (after fixes)

- Unit tests for ONNX NER integration
- Integration tests for full proxy pipeline
- E2E tests with real API calls
- Load tests (k6, target: <20ms overhead, 1000 RPS)
- Security pen-test script execution
- Final production readiness report with evidence

## Approach Selected

**Approach A: Sequential Phases** — validate before changing, fix with confidence, test comprehensively.

## Constraints

- No PII in logs (ever)
- No `unwrap()` in production Rust code
- Target latency: <20ms mask+unmask, <5ms NER inference
- OpenAI API key available for real E2E testing
