# Production Readiness Design — UnblockLLM Proxy

**Date:** 2026-03-02
**Approach:** Incremental Hardening (4 independent workstreams)
**Decisions:** Deployment-agnostic, OpenTelemetry for observability, Anthropic provider support

---

## 1. Context

UnblockLLM is a zero-trust privacy proxy that redacts PII locally (ONNX NER + regex) before forwarding prompts to public LLMs. Phases 1–7 are complete with 35 passing tests, clean lints, and load testing verified.

This design addresses 11 production readiness gaps identified during a full codebase audit, organized into 4 workstreams that can be shipped independently.

## 2. Gaps Identified

| # | Gap | Severity | Workstream |
|---|-----|----------|------------|
| 1 | No Prometheus `/metrics` endpoint — zero observability | Critical | WS2 |
| 2 | No graceful shutdown — in-flight requests dropped on deploy | Critical | WS1 |
| 3 | ONNX model not in Docker image — no NER in production container | Critical | WS1 |
| 4 | No readiness/liveness probes — `/health` always returns "ok" | High | WS1 |
| 5 | No structured JSON logging — plain text not parseable by log aggregators | High | WS2 |
| 6 | No request tracing headers — no `X-Request-ID` propagation | High | WS2 |
| 7 | CI doesn't run integration tests (ONNX model not present) | High | WS4 |
| 8 | No Grafana dashboards provisioned — empty Grafana after docker compose up | Medium | WS2 |
| 9 | No Helm chart / K8s manifests (deferred — deployment-agnostic focus) | Medium | — |
| 10 | Single-provider lock-in — only OpenAI, Anthropic mentioned but not implemented | Medium | WS3 |
| 11 | No CD pipeline — release.yml only builds binary, no Docker push | Medium | WS4 |

---

## 3. Workstream 1: Core Reliability

### 3.1 Graceful Shutdown

Use `axum::serve(...).with_graceful_shutdown(shutdown_signal())` where `shutdown_signal()` listens for SIGTERM + SIGINT via `tokio::signal`. On signal: stop accepting new connections, drain in-flight requests (up to 30s), log shutdown event, exit cleanly.

**File:** `crates/unblock-proxy/src/main.rs` (~15 lines)

### 3.2 Rich Health Probes

- **`/health`** (liveness): Lightweight, confirms process alive. Returns `{"status":"ok","version":"x.y.z"}`.
- **`/readyz`** (readiness): Checks Redis PING, Postgres SELECT 1, ONNX model loaded. Returns 200 if all pass, 503 with JSON details if any fail.

Pass `Store`, `AuditLog`, and model-loaded flag into readiness handler via shared state.

**Files:**
- `crates/unblock-proxy/src/main.rs` — add `/readyz` route
- `crates/unblock-proxy/src/handlers/health.rs` — **NEW** (~60 lines)

### 3.3 ONNX Model in Docker Image

Add a build stage to `Dockerfile.proxy` that either downloads the model from HuggingFace or COPYs from local `models/` directory. Set `ENV NER_MODEL_DIR=/models/bert-base-NER-onnx`. Optional via build arg `ARG INCLUDE_ONNX_MODEL=true`.

**File:** `Dockerfile.proxy` (~10 lines added)

---

## 4. Workstream 2: Observability (OpenTelemetry)

### 4.1 OpenTelemetry Metrics

Add `opentelemetry`, `opentelemetry-otlp`, `tracing-opentelemetry`, and `opentelemetry-prometheus` crates.

Metrics exported:
- `http_request_duration_seconds` — histogram by method, path, status
- `http_requests_total` — counter by method, path, status
- `pii_entities_detected_total` — counter by entity type
- `pii_masking_duration_seconds` — histogram of NER + masking latency
- `upstream_request_duration_seconds` — histogram of LLM response time

Dual export: OTLP push (via `OTEL_EXPORTER_OTLP_ENDPOINT`) + Prometheus `/metrics` scrape endpoint.

**Files:**
- `crates/unblock-proxy/Cargo.toml` — add otel deps
- `crates/unblock-proxy/src/middleware/metrics.rs` — **NEW** (~80 lines)
- `crates/unblock-proxy/src/middleware/mod.rs` — add metrics module
- `crates/unblock-proxy/src/handlers/chat.rs` — instrument masking duration
- `crates/unblock-proxy/src/main.rs` — init OTLP exporter, add `/metrics` route

### 4.2 Structured JSON Logging

Toggle JSON formatter via `LOG_FORMAT=json` env var (default: `text` for dev). Each log line includes: timestamp, level, target, message, span fields (request_id, method, path), otel.trace_id.

**File:** `crates/unblock-proxy/src/main.rs` (~15 lines, conditional formatter)

### 4.3 Request ID Propagation

Middleware extracts `X-Request-ID` from incoming request (or generates UUID). Stores in tracing span. Forwards to upstream. Returns in response headers.

**Files:**
- `crates/unblock-proxy/src/middleware/request_id.rs` — **NEW** (~40 lines)
- `crates/unblock-proxy/src/handlers/chat.rs` — forward request_id to upstream
- `crates/unblock-proxy/src/main.rs` — add middleware layer

### 4.4 Grafana Dashboard

Pre-built dashboard JSON with panels for: request rate, latency percentiles, error rate, PII entities by type, upstream LLM latency. Auto-provisioned on Grafana start.

**Files:**
- `monitoring/grafana/dashboards/proxy.json` — **NEW**
- `monitoring/grafana/provisioning/dashboards.yml` — **NEW**
- `monitoring/grafana/provisioning/datasources.yml` — **NEW**
- `docker-compose.monitoring.yml` — add volume mounts
- `monitoring/prometheus.yml` — uncomment proxy scrape target

---

## 5. Workstream 3: Anthropic Provider Support

### 5.1 Provider Abstraction

Route by path:
- `POST /v1/chat/completions` → OpenAI provider (existing)
- `POST /v1/messages` → Anthropic provider (new)

Both share the same masking pipeline: NER detect → policy filter → mask_text → store mappings.

### 5.2 Anthropic Messages API Handler

New handler at `/v1/messages`:
- Parses Anthropic request format (`model`, `messages`, `max_tokens`, content blocks)
- Auth: forwards `X-API-Key` and `Anthropic-Version` headers
- Upstream URL: `ANTHROPIC_BASE_URL` env var (default: `https://api.anthropic.com`)
- Streaming: parses Anthropic SSE events (`content_block_delta` → `delta.text`) for re-identification
- Non-streaming: extracts `content[].text` from response for re-identification

**Files:**
- `crates/unblock-proxy/src/handlers/anthropic.rs` — **NEW** (~200 lines)
- `crates/unblock-proxy/src/handlers/mod.rs` — add anthropic module

### 5.3 Shared Masking Pipeline Refactor

Extract masking logic from `chat_completions` into a reusable `mask_messages()` function:
- Takes messages, NerEngine, PolicyEngine, Store, AuditLog
- Returns masked messages + request_id + entity metadata
- Both OpenAI and Anthropic handlers call this

**File:** `crates/unblock-proxy/src/handlers/chat.rs` — extract ~40 lines

### 5.4 Integration Tests

Add ~4 new tests for Anthropic: non-streaming masking, streaming re-identification, content blocks format, clean passthrough. Reuse existing mock server pattern.

**File:** `crates/unblock-proxy/tests/integration.rs`

---

## 6. Workstream 4: CI/CD & Ops

### 6.1 Integration Tests in CI

Add CI step to download ONNX model before tests. Run `cargo test --test integration` explicitly with `NER_MODEL_DIR` set. Cache model between runs.

**File:** `.github/workflows/ci.yml` (~15 lines)

### 6.2 Docker Image Push in Release

Add Docker build + push to `release.yml`: build both `Dockerfile.proxy` and `Dockerfile.dashboard`, push to `ghcr.io`. Tag with git tag + `latest`.

**File:** `.github/workflows/release.yml` (~30 lines)

### 6.3 Version Embedding

Use `env!("CARGO_PKG_VERSION")` at compile time. Return in `/health` JSON response. Include in startup log. Add `X-Proxy-Version` response header.

**Files:** `crates/unblock-proxy/src/main.rs`, health handler

### 6.4 Documentation Updates

Update README.md, DEPLOYMENT.md, ARCHITECTURE.md, and SKILL.md to reflect: Anthropic support, `/metrics` + `/readyz` endpoints, `LOG_FORMAT`, OTLP env vars, graceful shutdown behavior, readiness probes.

---

## 7. Execution Order

```
WS1 (Core Reliability)  →  WS2 (Observability)  →  WS3 (Anthropic)  →  WS4 (CI/CD)
     ~1 day                    ~2 days                 ~2 days              ~1 day
```

Each workstream is independently shippable. The proxy is production-deployable after WS1+WS2.

## 8. New Environment Variables

| Variable | Default | WS | Purpose |
|----------|---------|-----|---------|
| `LOG_FORMAT` | `text` | WS2 | `json` for structured logging |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | (unset) | WS2 | OTLP collector endpoint |
| `OTEL_SERVICE_NAME` | `unblock-proxy` | WS2 | Service name in traces/metrics |
| `ANTHROPIC_BASE_URL` | `https://api.anthropic.com` | WS3 | Anthropic API base URL |

## 9. New Files Summary

| File | WS | Lines (est.) |
|------|-----|-------------|
| `crates/unblock-proxy/src/handlers/health.rs` | WS1 | ~60 |
| `crates/unblock-proxy/src/middleware/metrics.rs` | WS2 | ~80 |
| `crates/unblock-proxy/src/middleware/request_id.rs` | WS2 | ~40 |
| `monitoring/grafana/dashboards/proxy.json` | WS2 | ~200 |
| `monitoring/grafana/provisioning/dashboards.yml` | WS2 | ~10 |
| `monitoring/grafana/provisioning/datasources.yml` | WS2 | ~10 |
| `crates/unblock-proxy/src/handlers/anthropic.rs` | WS3 | ~200 |

## 10. Success Criteria

- All existing 35 tests continue to pass
- New integration tests for Anthropic (~4 tests)
- `cargo fmt --check` + `cargo clippy -D warnings` clean
- `/readyz` returns 503 when Redis is down
- `/metrics` exports Prometheus-format metrics
- Graceful shutdown drains requests on SIGTERM
- Docker image includes ONNX model and starts correctly
- CI runs integration tests with ONNX model
- Release pipeline pushes Docker images to GHCR

---

*Design approved and documented on 2026-03-02.*
