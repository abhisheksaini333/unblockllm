# Production Readiness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the UnblockLLM proxy production-ready by closing 11 gaps across reliability, observability, multi-provider support, and CI/CD.

**Architecture:** 4 independent workstreams (WS1→WS4), each shippable separately. WS1 adds graceful shutdown, rich health probes, and ONNX model in Docker. WS2 adds OpenTelemetry metrics/traces, structured JSON logging, request ID propagation, and Grafana dashboards. WS3 adds Anthropic provider support with shared masking pipeline. WS4 fixes CI integration tests and adds Docker image push to release pipeline.

**Tech Stack:** Rust/Axum, OpenTelemetry (`opentelemetry` + `opentelemetry-otlp` + `opentelemetry-prometheus`), `tracing-opentelemetry`, `tracing-subscriber` JSON layer, Prometheus, Grafana, Docker, GitHub Actions.

**Design doc:** `docs/plans/2026-03-02-production-readiness-design.md`

---

## WS1: Core Reliability

### Task 1: Graceful Shutdown

**Files:**
- Modify: `crates/unblock-proxy/src/main.rs:93-98`

**Step 1: Add shutdown_signal function**

Add this function at the bottom of `main.rs` (before the `#[cfg(test)]` block):

```rust
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received, draining connections...");
}
```

**Step 2: Wire graceful shutdown into axum::serve**

Replace lines 93-98 in `main.rs`:

```rust
// OLD:
let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
tracing::info!(%addr, "unblock-proxy listening");
let listener = tokio::net::TcpListener::bind(addr).await?;
let make_svc = app.into_make_service_with_connect_info::<SocketAddr>();
axum::serve(listener, make_svc).await?;
Ok(())

// NEW:
let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
tracing::info!(%addr, version = env!("CARGO_PKG_VERSION"), "unblock-proxy listening");
let listener = tokio::net::TcpListener::bind(addr).await?;
let make_svc = app.into_make_service_with_connect_info::<SocketAddr>();
axum::serve(listener, make_svc)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
tracing::info!("unblock-proxy shut down");
Ok(())
```

**Step 3: Verify it compiles**

Run: `cargo build -p unblock-proxy 2>&1 | tail -3`
Expected: `Finished` with no errors.

**Step 4: Run existing tests to verify no regressions**

Run: `cargo test -p unblock-proxy 2>&1 | grep "test result"`
Expected: All tests pass (10 lib + 1 main + 9 integration).

**Step 5: Commit**

```bash
git add crates/unblock-proxy/src/main.rs
git commit -m "feat(proxy): add graceful shutdown on SIGTERM/SIGINT"
```

---

### Task 2: Rich Health Probes — `/health` JSON + `/readyz`

**Files:**
- Create: `crates/unblock-proxy/src/handlers/health.rs`
- Modify: `crates/unblock-proxy/src/handlers/mod.rs`
- Modify: `crates/unblock-proxy/src/main.rs:62-69,89-91,101-104`

**Step 1: Create `health.rs` with health and readiness handlers**

Create file `crates/unblock-proxy/src/handlers/health.rs`:

```rust
//! Health and readiness probe handlers.
//! /health (liveness): lightweight, always returns JSON with version.
//! /readyz (readiness): checks Redis, Postgres, and NER model availability.

use crate::audit::AuditLog;
use crate::state::Store;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;

/// Shared state for health/readiness probes.
pub struct HealthState {
    pub store: Store,
    pub audit: AuditLog,
    pub ner_loaded: bool,
}

/// GET /health — liveness probe. Always 200 if process is alive.
pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// GET /readyz — readiness probe. Checks backing services.
pub async fn readiness_check(
    State(state): State<Arc<HealthState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut checks = serde_json::Map::new();
    let mut all_ok = true;

    // Check Redis/memory store
    let store_ok = state.store.new_request().await.is_ok();
    checks.insert("store".into(), json!(if store_ok { "ok" } else { "fail" }));
    if !store_ok {
        all_ok = false;
    }

    // Check Postgres (audit)
    let audit_ok = state.audit.ping().await;
    checks.insert("audit".into(), json!(if audit_ok { "ok" } else { "skip" }));

    // Check NER model loaded
    checks.insert(
        "ner".into(),
        json!(if state.ner_loaded { "ok" } else { "fail" }),
    );
    if !state.ner_loaded {
        all_ok = false;
    }

    let body = json!({
        "status": if all_ok { "ready" } else { "not_ready" },
        "version": env!("CARGO_PKG_VERSION"),
        "checks": checks,
    });

    if all_ok {
        Ok(Json(body))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, Json(body)))
    }
}
```

**Step 2: Add `ping()` method to `AuditLog`**

In `crates/unblock-proxy/src/audit.rs`, add this method inside the `impl AuditLog` block (after the `log` method):

```rust
    /// Ping the database. Returns true if connected, false if no pool or query fails.
    pub async fn ping(&self) -> bool {
        match &self.0 {
            Some(pool) => sqlx::query("SELECT 1").execute(pool.as_ref()).await.is_ok(),
            None => true, // No pool configured = skip check (not a failure)
        }
    }
```

**Step 3: Register `health` module in handlers/mod.rs**

Change `crates/unblock-proxy/src/handlers/mod.rs` from:

```rust
pub mod chat;
```

to:

```rust
pub mod chat;
pub mod health;
```

**Step 4: Update `main.rs` to use new health handlers and create HealthState**

In `main.rs`, add import:

```rust
use unblock_proxy::handlers::health::{health_check, readiness_check, HealthState};
```

After the `NerEngine::from_env()` call (line ~57), add:

```rust
let ner_loaded = ner.has_onnx_model();
```

After the `ChatState` creation (line ~69), add:

```rust
let health_state = Arc::new(HealthState {
    store: state.store.clone(),
    audit: state.audit.clone(),
    ner_loaded,
});
```

Note: `ChatState` fields `store` and `audit` are already `Clone`. We need to expose `store` and `audit` from `ChatState` or pass them separately. Since `ChatState` already holds them, we can clone from it:

```rust
let health_state = Arc::new(HealthState {
    store: state.store.clone(),
    audit: state.audit.clone(),
    ner_loaded,
});
```

Wait — `state` is `Arc<ChatState>`. So access via `state.store.clone()` works since `Store` is `Clone` and `AuditLog` is `Clone`.

Update the app router (lines 89-91) from:

```rust
let app = Router::new()
    .route("/health", get(health))
    .merge(chat_route);
```

to:

```rust
let readyz_route = Router::new()
    .route("/readyz", get(readiness_check))
    .with_state(health_state);
let app = Router::new()
    .route("/health", get(health_check))
    .merge(readyz_route)
    .merge(chat_route);
```

Remove the old inline `health()` function (lines 101-104):

```rust
// DELETE:
/// Health check for load balancers and CI. No PII.
async fn health() -> &'static str {
    "ok"
}
```

**Step 5: Add `has_onnx_model()` to NerEngine**

In `crates/unblock-proxy/src/ner/engine.rs`, add a public method:

```rust
    /// Returns true if the ONNX model is loaded (not regex-only fallback).
    pub fn has_onnx_model(&self) -> bool {
        self.session.is_some()
    }
```

This requires checking the NerEngine struct to see if it has a `session` field. If the field name differs, adjust accordingly. The key is: does this instance have an ONNX session or is it regex-only?

**Step 6: Update the existing `health_returns_ok` test in main.rs**

The test at the bottom of `main.rs` references the old `super::health` function. Update it:

```rust
#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::Router;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use unblock_proxy::handlers::health::health_check;

    #[tokio::test]
    async fn health_returns_ok() {
        let app = Router::new().route("/health", axum::routing::get(health_check));
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert!(json["version"].is_string());
    }
}
```

**Step 7: Verify compilation and tests**

Run: `cargo build -p unblock-proxy 2>&1 | tail -3`
Expected: `Finished` with no errors.

Run: `cargo test -p unblock-proxy 2>&1 | grep "test result"`
Expected: All tests pass.

**Step 8: Commit**

```bash
git add crates/unblock-proxy/src/handlers/health.rs \
        crates/unblock-proxy/src/handlers/mod.rs \
        crates/unblock-proxy/src/audit.rs \
        crates/unblock-proxy/src/ner/engine.rs \
        crates/unblock-proxy/src/main.rs
git commit -m "feat(proxy): add /readyz readiness probe and JSON /health liveness"
```

---

### Task 3: ONNX Model in Docker Image

**Files:**
- Modify: `Dockerfile.proxy`

**Step 1: Update Dockerfile.proxy to include ONNX model**

Replace the entire `Dockerfile.proxy` with:

```dockerfile
# Production proxy image: multi-stage, distroless, non-root. Target <100MB.
# Build from repo root: docker build -f Dockerfile.proxy .

# Optional: set INCLUDE_ONNX_MODEL=false to skip model download (regex-only mode)
ARG INCLUDE_ONNX_MODEL=true

# Use bookworm (latest stable) for edition2024-capable Cargo (transitive deps)
FROM rust:bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY proto ./proto

# protoc required by unblock-core build.rs (prost)
RUN apt-get update && apt-get install -y --no-install-recommends protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Build release (no debug symbols for smaller image)
RUN cargo build --release --package unblock-proxy && \
    strip /build/target/release/unblock-proxy

# Download ONNX model from HuggingFace (if enabled)
FROM python:3.11-slim AS model-downloader
ARG INCLUDE_ONNX_MODEL
RUN if [ "$INCLUDE_ONNX_MODEL" = "true" ]; then \
        pip install --no-cache-dir huggingface-hub && \
        huggingface-cli download protectai/bert-base-NER-onnx \
            --local-dir /models/bert-base-NER-onnx \
            --local-dir-use-symlinks False; \
    else \
        mkdir -p /models/bert-base-NER-onnx; \
    fi

# Runtime: distroless cc (glibc); non-root
FROM gcr.io/distroless/cc-debian12:nonroot
USER nonroot:nonroot
EXPOSE 8080
COPY --from=builder /build/target/release/unblock-proxy /unblock-proxy
COPY --from=model-downloader /models /models
ENV NER_MODEL_DIR=/models/bert-base-NER-onnx
ENTRYPOINT ["/unblock-proxy"]
```

**Step 2: Verify Dockerfile syntax (dry run)**

Run: `docker build --dry-run -f Dockerfile.proxy . 2>&1 | head -5` (if supported) or just verify the file is valid YAML/Dockerfile syntax by reading it.

**Step 3: Commit**

```bash
git add Dockerfile.proxy
git commit -m "feat(docker): include ONNX NER model in proxy image"
```

---

## WS2: Observability

### Task 4: Add OpenTelemetry Dependencies

**Files:**
- Modify: `crates/unblock-proxy/Cargo.toml`

**Step 1: Add otel crates to dependencies**

Add these lines to the `[dependencies]` section of `crates/unblock-proxy/Cargo.toml`:

```toml
opentelemetry = { version = "0.27", features = ["metrics"] }
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio", "metrics"] }
opentelemetry-otlp = { version = "0.27", features = ["metrics", "grpc-tonic"] }
opentelemetry-prometheus = "0.27"
prometheus = "0.13"
tracing-opentelemetry = "0.28"
```

Also add `"json"` feature to tracing-subscriber:

Change:
```toml
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```
To:
```toml
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

**Step 2: Verify deps resolve**

Run: `cargo check -p unblock-proxy 2>&1 | tail -5`
Expected: Finishes without errors. If version conflicts occur, adjust versions to compatible ones.

**Step 3: Commit**

```bash
git add crates/unblock-proxy/Cargo.toml Cargo.lock
git commit -m "deps(proxy): add opentelemetry, prometheus, tracing-opentelemetry"
```

---

### Task 5: Metrics Middleware

**Files:**
- Create: `crates/unblock-proxy/src/middleware/metrics.rs`
- Modify: `crates/unblock-proxy/src/middleware/mod.rs`

**Step 1: Create the metrics middleware**

Create `crates/unblock-proxy/src/middleware/metrics.rs`:

```rust
//! Prometheus + OpenTelemetry metrics middleware for HTTP requests.
//! Records request count and duration histogram by method, path, and status.

use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use opentelemetry::metrics::{Counter, Histogram, Meter};
use std::sync::OnceLock;
use std::time::Instant;

struct HttpMetrics {
    request_count: Counter<u64>,
    request_duration: Histogram<f64>,
}

static HTTP_METRICS: OnceLock<HttpMetrics> = OnceLock::new();

/// Initialize HTTP metrics with the given meter. Call once at startup.
pub fn init_http_metrics(meter: &Meter) {
    let _ = HTTP_METRICS.get_or_init(|| HttpMetrics {
        request_count: meter
            .u64_counter("http_requests_total")
            .with_description("Total HTTP requests")
            .build(),
        request_duration: meter
            .f64_histogram("http_request_duration_seconds")
            .with_description("HTTP request duration in seconds")
            .build(),
    });
}

/// Axum middleware that records request metrics.
pub async fn metrics_layer(req: Request<Body>, next: Next) -> Response {
    let method = req.method().to_string();
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());

    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    if let Some(metrics) = HTTP_METRICS.get() {
        let attrs = [
            opentelemetry::KeyValue::new("method", method),
            opentelemetry::KeyValue::new("path", path),
            opentelemetry::KeyValue::new("status", status),
        ];
        metrics.request_count.add(1, &attrs);
        metrics.request_duration.record(duration, &attrs);
    }

    response
}
```

**Step 2: Register metrics module**

Change `crates/unblock-proxy/src/middleware/mod.rs` to:

```rust
//! Middleware: rate limiting, metrics, request ID.

pub mod metrics;
pub mod rate_limit;
```

**Step 3: Verify compilation**

Run: `cargo check -p unblock-proxy 2>&1 | tail -5`
Expected: Compiles.

**Step 4: Commit**

```bash
git add crates/unblock-proxy/src/middleware/metrics.rs \
        crates/unblock-proxy/src/middleware/mod.rs
git commit -m "feat(proxy): add HTTP metrics middleware (otel + prometheus)"
```

---

### Task 6: PII Metrics in Chat Handler

**Files:**
- Modify: `crates/unblock-proxy/src/handlers/chat.rs`

**Step 1: Add PII metrics counters**

Add these imports at the top of `chat.rs`:

```rust
use opentelemetry::metrics::{Counter, Histogram, Meter};
use std::sync::OnceLock;
```

Add static metrics after the imports:

```rust
struct PiiMetrics {
    entities_detected: Counter<u64>,
    masking_duration: Histogram<f64>,
    upstream_duration: Histogram<f64>,
}

static PII_METRICS: OnceLock<PiiMetrics> = OnceLock::new();

/// Initialize PII-specific metrics. Call once at startup.
pub fn init_pii_metrics(meter: &Meter) {
    let _ = PII_METRICS.get_or_init(|| PiiMetrics {
        entities_detected: meter
            .u64_counter("pii_entities_detected_total")
            .with_description("Total PII entities detected by type")
            .build(),
        masking_duration: meter
            .f64_histogram("pii_masking_duration_seconds")
            .with_description("Time spent on NER + masking per request")
            .build(),
        upstream_duration: meter
            .f64_histogram("upstream_request_duration_seconds")
            .with_description("Time waiting for upstream LLM response")
            .build(),
    });
}
```

**Step 2: Instrument the chat handler**

In the `chat_completions` function, after the masking loop and before the `entity_type_tags` line (~line 128), add:

```rust
    // Record PII metrics
    if let Some(pii) = PII_METRICS.get() {
        pii.masking_duration
            .record(mask_elapsed.as_secs_f64(), &[]);
        for tag in &entity_type_tags {
            pii.entities_detected.add(
                1,
                &[opentelemetry::KeyValue::new("entity_type", tag.clone())],
            );
        }
    }
```

Wait — `entity_type_tags` is built after this point. Move the metric recording to after line 128 (`let entity_type_tags: Vec<String> = ...`). Place it after the `tracing::debug!` line (~135):

```rust
    if let Some(pii) = PII_METRICS.get() {
        pii.masking_duration
            .record(mask_elapsed.as_secs_f64(), &[]);
        for tag in &entity_type_tags {
            pii.entities_detected.add(
                1,
                &[opentelemetry::KeyValue::new("entity_type", tag.clone())],
            );
        }
    }
```

Before the upstream request (after the `let upstream = state.client.post...` section), wrap the upstream call to measure duration:

```rust
    let upstream_start = Instant::now();
    // ... existing upstream call ...
    let upstream_elapsed = upstream_start.elapsed();
    if let Some(pii) = PII_METRICS.get() {
        pii.upstream_duration
            .record(upstream_elapsed.as_secs_f64(), &[]);
    }
```

**Step 3: Verify compilation**

Run: `cargo check -p unblock-proxy 2>&1 | tail -5`

**Step 4: Run tests**

Run: `cargo test -p unblock-proxy 2>&1 | grep "test result"`
Expected: All pass.

**Step 5: Commit**

```bash
git add crates/unblock-proxy/src/handlers/chat.rs
git commit -m "feat(proxy): instrument chat handler with PII and upstream metrics"
```

---

### Task 7: Prometheus `/metrics` Endpoint + OTLP Init

**Files:**
- Modify: `crates/unblock-proxy/src/main.rs`

**Step 1: Add OTLP + Prometheus initialization in main()**

Add these imports to `main.rs`:

```rust
use opentelemetry::metrics::MeterProvider;
use opentelemetry_prometheus::exporter::PrometheusExporter;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use unblock_proxy::handlers::chat::init_pii_metrics;
use unblock_proxy::middleware::metrics::{init_http_metrics, metrics_layer};
```

After the tracing init block in `main()`, add Prometheus exporter setup:

```rust
    // Prometheus exporter for /metrics endpoint
    let prometheus_registry = prometheus::Registry::new();
    let prometheus_exporter = opentelemetry_prometheus::exporter()
        .with_registry(prometheus_registry.clone())
        .build()
        .map_err(|e| format!("prometheus exporter: {}", e))?;
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(prometheus_exporter)
        .build();
    let meter = meter_provider.meter("unblock-proxy");

    // Initialize metrics
    init_http_metrics(&meter);
    init_pii_metrics(&meter);
```

Add a `/metrics` handler:

```rust
    let prom_registry = prometheus_registry.clone();
    let metrics_handler = move || {
        let prom_registry = prom_registry.clone();
        async move {
            use prometheus::Encoder;
            let encoder = prometheus::TextEncoder::new();
            let metric_families = prom_registry.gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            (
                [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                buffer,
            )
        }
    };
```

Update the app router to include `/metrics` and the metrics middleware:

```rust
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .merge(readyz_route)
        .merge(chat_route)
        .layer(axum::middleware::from_fn(metrics_layer));
```

**Step 2: Verify compilation and tests**

Run: `cargo check -p unblock-proxy 2>&1 | tail -5`
Run: `cargo test -p unblock-proxy 2>&1 | grep "test result"`

**Step 3: Commit**

```bash
git add crates/unblock-proxy/src/main.rs
git commit -m "feat(proxy): add /metrics prometheus endpoint and OTLP meter init"
```

---

### Task 8: Structured JSON Logging

**Files:**
- Modify: `crates/unblock-proxy/src/main.rs:24-28`

**Step 1: Replace tracing init with conditional JSON/text formatter**

Replace the tracing init block:

```rust
// OLD:
let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
tracing_subscriber::registry()
    .with(filter)
    .with(tracing_subscriber::fmt::layer())
    .init();

// NEW:
let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());
if log_format == "json" {
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
} else {
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

**Step 2: Verify compilation**

Run: `cargo check -p unblock-proxy 2>&1 | tail -5`

**Step 3: Manual test (optional)**

Run: `LOG_FORMAT=json cargo run -p unblock-proxy 2>&1 | head -3`
Expected: JSON-formatted log lines.

**Step 4: Commit**

```bash
git add crates/unblock-proxy/src/main.rs
git commit -m "feat(proxy): structured JSON logging via LOG_FORMAT=json"
```

---

### Task 9: Request ID Middleware

**Files:**
- Create: `crates/unblock-proxy/src/middleware/request_id.rs`
- Modify: `crates/unblock-proxy/src/middleware/mod.rs`
- Modify: `crates/unblock-proxy/src/main.rs`

**Step 1: Create request_id middleware**

Create `crates/unblock-proxy/src/middleware/request_id.rs`:

```rust
//! X-Request-ID propagation middleware.
//! Extracts or generates a request ID, adds it to a tracing span,
//! and returns it in the response headers.

use axum::body::Body;
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";

/// Axum middleware: extract or generate X-Request-ID, add to span and response.
pub async fn request_id_layer(req: Request<Body>, next: Next) -> Response {
    let request_id = req
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let span = tracing::info_span!("request", request_id = %request_id);
    let _guard = span.enter();

    let mut response = next.run(req).await;

    if let Ok(val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, val);
    }

    response
}
```

**Step 2: Register module**

Add to `crates/unblock-proxy/src/middleware/mod.rs`:

```rust
pub mod request_id;
```

**Step 3: Wire into main.rs app**

Add import:

```rust
use unblock_proxy::middleware::request_id::request_id_layer;
```

Update the app router to include request_id middleware (before metrics_layer):

```rust
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .merge(readyz_route)
        .merge(chat_route)
        .layer(axum::middleware::from_fn(request_id_layer))
        .layer(axum::middleware::from_fn(metrics_layer));
```

Note: Layer order in Axum is "last added = outermost". So metrics_layer wraps request_id_layer wraps the routes. This means metrics sees the request first, then request_id adds the ID.

Actually, we want request_id to run first (outermost) so the ID is available for logging within metrics. Reverse the order:

```rust
        .layer(axum::middleware::from_fn(metrics_layer))
        .layer(axum::middleware::from_fn(request_id_layer));
```

In Axum, `.layer()` wraps outer-to-inner in reverse order of declaration. The last `.layer()` is the outermost. So `request_id_layer` (last) runs first → then `metrics_layer`.

**Step 4: Verify compilation and tests**

Run: `cargo check -p unblock-proxy 2>&1 | tail -5`
Run: `cargo test -p unblock-proxy 2>&1 | grep "test result"`

**Step 5: Commit**

```bash
git add crates/unblock-proxy/src/middleware/request_id.rs \
        crates/unblock-proxy/src/middleware/mod.rs \
        crates/unblock-proxy/src/main.rs
git commit -m "feat(proxy): add X-Request-ID propagation middleware"
```

---

### Task 10: Grafana Dashboard + Prometheus Config

**Files:**
- Create: `monitoring/grafana/provisioning/datasources.yml`
- Create: `monitoring/grafana/provisioning/dashboards.yml`
- Create: `monitoring/grafana/dashboards/proxy.json`
- Modify: `monitoring/prometheus.yml`
- Modify: `docker-compose.monitoring.yml`

**Step 1: Create datasource provisioning**

Create `monitoring/grafana/provisioning/datasources.yml`:

```yaml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
```

**Step 2: Create dashboard provisioning**

Create `monitoring/grafana/provisioning/dashboards.yml`:

```yaml
apiVersion: 1
providers:
  - name: default
    orgId: 1
    folder: ""
    type: file
    disableDeletion: false
    editable: true
    options:
      path: /var/lib/grafana/dashboards
      foldersFromFilesStructure: false
```

**Step 3: Create proxy dashboard JSON**

Create `monitoring/grafana/dashboards/proxy.json` — a standard Grafana dashboard with panels for the key metrics (request rate, latency, errors, PII entities, upstream latency). This is a large JSON file; generate it with appropriate Prometheus queries targeting the metric names from Task 5-7.

Key panels:
- Request Rate: `rate(http_requests_total[1m])`
- Latency p50/p95/p99: `histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))`
- Error Rate: `rate(http_requests_total{status=~"5.."}[1m])`
- PII Entities: `rate(pii_entities_detected_total[1m])` by `entity_type`
- Upstream Latency: `histogram_quantile(0.95, rate(upstream_request_duration_seconds_bucket[5m]))`

**Step 4: Uncomment proxy scrape target in prometheus.yml**

Change `monitoring/prometheus.yml`:

```yaml
  # Example: when proxy exposes /metrics (future)
  # - job_name: proxy
  #   static_configs:
  #     - targets: ["proxy:8080"]
  #   metrics_path: /metrics
```

to:

```yaml
  - job_name: proxy
    static_configs:
      - targets: ["proxy:8080"]
    metrics_path: /metrics
    scrape_interval: 5s
```

**Step 5: Update docker-compose.monitoring.yml**

Add volume mounts to the grafana service:

```yaml
  grafana:
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/provisioning:/etc/grafana/provisioning:ro
      - ./monitoring/grafana/dashboards:/var/lib/grafana/dashboards:ro
```

**Step 6: Commit**

```bash
git add monitoring/
git add docker-compose.monitoring.yml
git commit -m "feat(monitoring): add Grafana dashboards and Prometheus proxy scrape"
```

---

## WS3: Anthropic Provider

### Task 11: Shared Masking Pipeline Refactor

**Files:**
- Modify: `crates/unblock-proxy/src/handlers/chat.rs`

**Step 1: Extract masking logic into a reusable function**

Add this struct and function in `chat.rs` (before the `chat_completions` handler):

```rust
/// Result of masking a set of messages.
pub struct MaskResult {
    pub request_id: String,
    pub total_entity_count: u32,
    pub entity_type_tags: Vec<String>,
    pub mask_elapsed: std::time::Duration,
}

/// Shared masking pipeline: NER detect → policy filter → mask → store mappings → audit.
/// Used by both OpenAI and Anthropic handlers.
pub async fn mask_messages(
    messages: &mut [ChatMessage],
    state: &ChatState,
) -> Result<MaskResult, (StatusCode, String)> {
    let request_id = state.store.new_request().await.map_err(map_store_error)?;
    let start = Instant::now();
    let mut total_entity_count: u32 = 0;
    let mut entity_type_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    for msg in messages.iter_mut() {
        let Some(ref content) = msg.content else {
            continue;
        };
        if content.is_empty() {
            continue;
        }
        let spans = state
            .ner
            .detect_async(content.clone())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let filtered = filter_spans_by_policy(&state.policy, &spans)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        for s in &filtered {
            total_entity_count += 1;
            entity_type_set.insert(entity_type_tag(&s.entity_type));
        }
        let (masked, mapping_pairs) = mask_text(content, &filtered);
        for (placeholder, value) in mapping_pairs {
            state
                .store
                .insert(&request_id, placeholder, value)
                .await
                .map_err(map_store_error)?;
        }
        msg.content = Some(masked);
    }

    let entity_type_tags: Vec<String> = entity_type_set.into_iter().collect();
    state
        .audit
        .log(&request_id, None, total_entity_count, &entity_type_tags)
        .await;

    let mask_elapsed = start.elapsed();
    tracing::debug!(request_id = %request_id, mask_ms = mask_elapsed.as_millis(), "masked request");

    // Record PII metrics
    if let Some(pii) = PII_METRICS.get() {
        pii.masking_duration
            .record(mask_elapsed.as_secs_f64(), &[]);
        for tag in &entity_type_tags {
            pii.entities_detected.add(
                1,
                &[opentelemetry::KeyValue::new("entity_type", tag.clone())],
            );
        }
    }

    Ok(MaskResult {
        request_id,
        total_entity_count,
        entity_type_tags,
        mask_elapsed,
    })
}
```

**Step 2: Refactor chat_completions to use mask_messages**

Replace the masking section of `chat_completions` (from `let start = Instant::now()` through the `tracing::debug!` line) with:

```rust
    let mask_result = mask_messages(&mut chat.messages, &state).await?;
    let request_id = mask_result.request_id;
```

Keep the rest of the handler the same (forwarding, re-identification, etc.). Remove the now-redundant variables (`start`, `total_entity_count`, `entity_type_set`, `entity_type_tags`, `mask_elapsed`).

**Step 3: Verify compilation and all tests**

Run: `cargo test -p unblock-proxy 2>&1 | grep "test result"`
Expected: All pass (no behavior change).

**Step 4: Commit**

```bash
git add crates/unblock-proxy/src/handlers/chat.rs
git commit -m "refactor(proxy): extract shared mask_messages pipeline for multi-provider"
```

---

### Task 12: Anthropic Handler

**Files:**
- Create: `crates/unblock-proxy/src/handlers/anthropic.rs`
- Modify: `crates/unblock-proxy/src/handlers/mod.rs`
- Modify: `crates/unblock-proxy/src/handlers/chat.rs` (make ChatState fields + types public if needed)
- Modify: `crates/unblock-proxy/src/main.rs`

**Step 1: Create the Anthropic handler**

Create `crates/unblock-proxy/src/handlers/anthropic.rs`:

```rust
//! Handler for POST /v1/messages: Anthropic Messages API with PII masking.
//! Mirrors the OpenAI handler pattern but with Anthropic-specific request/response formats.

use crate::handlers::chat::{mask_messages, map_store_error, ChatMessage, ChatState, PII_METRICS};
use crate::masking::reidentify_map;
use crate::state::Store;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use bytes::Bytes;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

/// Anthropic Messages API request format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    pub max_tokens: u32,
    #[serde(default)]
    pub stream: bool,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: AnthropicContent,
}

/// Anthropic content can be a string or array of content blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnthropicContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl AnthropicMessage {
    /// Convert to ChatMessage for the shared masking pipeline.
    fn to_chat_message(&self) -> ChatMessage {
        let content = match &self.content {
            AnthropicContent::Text(s) => Some(s.clone()),
            AnthropicContent::Blocks(blocks) => {
                let texts: Vec<&str> = blocks
                    .iter()
                    .filter(|b| b.block_type == "text")
                    .filter_map(|b| b.text.as_deref())
                    .collect();
                if texts.is_empty() {
                    None
                } else {
                    Some(texts.join("\n"))
                }
            }
        };
        ChatMessage {
            role: self.role.clone(),
            content,
        }
    }

    /// Update content from a masked ChatMessage.
    fn apply_masked(&mut self, masked: &ChatMessage) {
        let Some(ref masked_text) = masked.content else {
            return;
        };
        match &mut self.content {
            AnthropicContent::Text(ref mut s) => *s = masked_text.clone(),
            AnthropicContent::Blocks(ref mut blocks) => {
                // Apply masked text back to the first text block
                if let Some(block) = blocks.iter_mut().find(|b| b.block_type == "text") {
                    block.text = Some(masked_text.clone());
                }
            }
        }
    }
}

/// POST /v1/messages — Anthropic Messages API
pub async fn anthropic_messages(
    State(state): State<Arc<ChatState>>,
    req: Request<Body>,
) -> Result<Response, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut body_value: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;
    let mut anthropic_req: AnthropicRequest =
        serde_json::from_value(body_value.clone()).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid Anthropic request: {}", e),
            )
        })?;

    // Convert to ChatMessages for shared masking pipeline
    let mut chat_messages: Vec<ChatMessage> = anthropic_req
        .messages
        .iter()
        .map(|m| m.to_chat_message())
        .collect();

    let mask_result = mask_messages(&mut chat_messages, &state).await?;
    let request_id = mask_result.request_id;

    // Apply masked content back to Anthropic messages
    for (anthro_msg, chat_msg) in anthropic_req.messages.iter_mut().zip(chat_messages.iter()) {
        anthro_msg.apply_masked(chat_msg);
    }

    // Write masked messages back into body
    if let Some(obj) = body_value.as_object_mut() {
        let messages_val = serde_json::to_value(&anthropic_req.messages)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        obj.insert("messages".to_string(), messages_val);
    }

    let anthropic_base = std::env::var("ANTHROPIC_BASE_URL")
        .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
    let url = format!("{}/v1/messages", anthropic_base.trim_end_matches('/'));

    let mut fwd_headers = parts.headers.clone();
    fwd_headers.remove(axum::http::header::CONTENT_LENGTH);

    let upstream_start = Instant::now();
    let upstream = state
        .client
        .post(&url)
        .headers(fwd_headers)
        .json(&body_value)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let upstream_elapsed = upstream_start.elapsed();

    if let Some(pii) = PII_METRICS.get() {
        pii.upstream_duration
            .record(upstream_elapsed.as_secs_f64(), &[]);
    }

    let status = upstream.status();
    let mut headers = upstream.headers().clone();

    if anthropic_req.stream {
        let mapping = state
            .store
            .get_mapping(&request_id)
            .await
            .map_err(map_store_error)?
            .unwrap_or_default();
        let store = state.store.clone();
        let rid = request_id.clone();
        let stream = upstream
            .bytes_stream()
            .map(move |r: Result<Bytes, reqwest::Error>| {
                let r = r.map_err(|e| e.to_string())?;
                let text = String::from_utf8_lossy(&r);
                let out = reidentify_map(&text, &mapping);
                Ok::<_, String>(Bytes::from(out))
            });
        tokio::spawn(async move {
            let _ = store.remove(&rid).await;
        });
        let body = Body::from_stream(stream);
        let mut res = Response::new(body);
        *res.status_mut() = status;
        *res.headers_mut() = headers;
        return Ok(res);
    }

    // Non-streaming: re-identify content in response
    let body = upstream
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let mapping = state
        .store
        .get_mapping(&request_id)
        .await
        .map_err(map_store_error)?;
    state
        .store
        .remove(&request_id)
        .await
        .map_err(map_store_error)?;

    let mut resp_body: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);

    // Anthropic response: { "content": [{"type":"text","text":"..."}], ... }
    if let Some(m) = mapping {
        if let Some(content) = resp_body.get_mut("content").and_then(|c| c.as_array_mut()) {
            for block in content.iter_mut() {
                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                    if let Some(text) = block.get_mut("text").and_then(|t| t.as_str().map(|s| s.to_string())) {
                        block["text"] = serde_json::Value::String(reidentify_map(&text, &m));
                    }
                }
            }
        }
    }

    let json = serde_json::to_vec(&resp_body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut res = Response::new(Body::from(json));
    *res.status_mut() = status;
    headers.remove(axum::http::header::CONTENT_LENGTH);
    headers.remove(axum::http::header::TRANSFER_ENCODING);
    *res.headers_mut() = headers;
    res.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/json"),
    );
    Ok(res)
}
```

**Step 2: Register the anthropic module**

Update `crates/unblock-proxy/src/handlers/mod.rs`:

```rust
pub mod anthropic;
pub mod chat;
pub mod health;
```

**Step 3: Make necessary items public in chat.rs**

In `chat.rs`, ensure these are `pub`:
- `map_store_error` function
- `PII_METRICS` static
- `ChatMessage` struct (already pub)
- `mask_messages` function (already pub from Task 11)

**Step 4: Add `/v1/messages` route in main.rs**

Add import:

```rust
use unblock_proxy::handlers::anthropic::anthropic_messages;
```

In the chat_route builder, add the Anthropic route alongside the OpenAI one. Both routes share the same state and rate limiter:

```rust
    // In the if/else for Redis/in-memory rate limiting:
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/messages", post(anthropic_messages))
        .layer(layer)
        .with_state(state)
```

**Step 5: Verify compilation**

Run: `cargo check -p unblock-proxy 2>&1 | tail -5`

**Step 6: Commit**

```bash
git add crates/unblock-proxy/src/handlers/anthropic.rs \
        crates/unblock-proxy/src/handlers/mod.rs \
        crates/unblock-proxy/src/handlers/chat.rs \
        crates/unblock-proxy/src/main.rs
git commit -m "feat(proxy): add Anthropic /v1/messages handler with PII masking"
```

---

### Task 13: Anthropic Integration Tests

**Files:**
- Modify: `crates/unblock-proxy/tests/integration.rs`

**Step 1: Add mock Anthropic server and build_app helper for Anthropic**

Add a mock Anthropic server function that returns Anthropic-format responses (content blocks). Add a `build_app_anthropic()` helper. Add 4 tests:

1. `anthropic_non_streaming_masks_email` — email in content → masked → re-identified in response
2. `anthropic_content_blocks_masks_email` — content as array of blocks → masked
3. `anthropic_streaming_masks_email` — SSE stream with Anthropic events
4. `anthropic_no_pii_passes_through` — clean text → passthrough

Reuse the existing mock server pattern from the OpenAI tests.

**Step 2: Run all tests**

Run: `cargo test -p unblock-proxy 2>&1 | grep "test result"`
Expected: All existing tests + 4 new tests pass.

**Step 3: Commit**

```bash
git add crates/unblock-proxy/tests/integration.rs
git commit -m "test(proxy): add Anthropic integration tests (4 tests)"
```

---

## WS4: CI/CD & Ops

### Task 14: Integration Tests in CI

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Add ONNX model download and integration test step**

In the `rust` job, after the `Test` step, add:

```yaml
      - name: Download ONNX NER model
        run: |
          pip install huggingface-hub
          mkdir -p models/bert-base-NER-onnx
          huggingface-cli download protectai/bert-base-NER-onnx \
            --local-dir models/bert-base-NER-onnx \
            --local-dir-use-symlinks False

      - name: Integration tests
        env:
          NER_MODEL_DIR: models/bert-base-NER-onnx
        run: cargo test --test integration -p unblock-proxy -v
```

Optionally cache the model directory:

```yaml
      - name: Cache ONNX model
        uses: actions/cache@v4
        with:
          path: models/bert-base-NER-onnx
          key: onnx-ner-model-v1
```

Place the cache step before the download step. The download step should check if the model already exists.

**Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add ONNX model download and integration tests"
```

---

### Task 15: Docker Image Push in Release

**Files:**
- Modify: `.github/workflows/release.yml`

**Step 1: Add Docker build + push steps**

Add after the existing `Create Release` step:

```yaml
      - name: Log in to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build and push proxy image
        uses: docker/build-push-action@v5
        with:
          context: .
          file: Dockerfile.proxy
          push: true
          tags: |
            ghcr.io/${{ github.repository_owner }}/unblock-proxy:${{ github.ref_name }}
            ghcr.io/${{ github.repository_owner }}/unblock-proxy:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Build and push dashboard image
        uses: docker/build-push-action@v5
        with:
          context: .
          file: Dockerfile.dashboard
          push: true
          tags: |
            ghcr.io/${{ github.repository_owner }}/unblock-dashboard:${{ github.ref_name }}
            ghcr.io/${{ github.repository_owner }}/unblock-dashboard:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

Also add `packages: write` to the permissions block:

```yaml
permissions:
  contents: write
  packages: write
```

**Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add Docker image build + push to GHCR on release"
```

---

### Task 16: Documentation Updates

**Files:**
- Modify: `README.md`
- Modify: `docs/DEPLOYMENT.md`
- Modify: `docs/ARCHITECTURE.md`
- Modify: `SKILL.md`
- Modify: `.env.example`

**Step 1: Update README.md**

Add Anthropic support, new endpoints, and env vars.

**Step 2: Update DEPLOYMENT.md**

Add sections on: graceful shutdown (30s drain), `/readyz` for K8s readiness probes, `/metrics` for Prometheus, `LOG_FORMAT=json`, OTLP env vars, `ANTHROPIC_BASE_URL`.

**Step 3: Update ARCHITECTURE.md**

Add Anthropic provider path to the data flow diagram. Update repository structure section.

**Step 4: Update SKILL.md**

Update phase plan to include Phase 8 (production readiness). Add new verification commands.

**Step 5: Update .env.example**

Add new env vars: `LOG_FORMAT`, `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_SERVICE_NAME`, `ANTHROPIC_BASE_URL`.

**Step 6: Commit**

```bash
git add README.md docs/DEPLOYMENT.md docs/ARCHITECTURE.md SKILL.md .env.example
git commit -m "docs: update for production readiness (Anthropic, observability, probes)"
```

---

### Task 17: Final Verification

**Step 1: Run full test suite**

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
cd python && ruff check . && pytest -v && cd ..
```

Expected: All pass, zero warnings.

**Step 2: Run integration tests explicitly**

```bash
NER_MODEL_DIR=models/bert-base-NER-onnx cargo test --test integration -p unblock-proxy -v
```

Expected: All tests pass (9 existing + 4 Anthropic = 13).

**Step 3: Smoke test the proxy with metrics**

```bash
# Terminal 1: start proxy
cargo run -p unblock-proxy

# Terminal 2: check endpoints
curl -s http://127.0.0.1:8080/health | jq .
curl -s http://127.0.0.1:8080/readyz | jq .
curl -s http://127.0.0.1:8080/metrics | head -20

# Verify graceful shutdown
kill -SIGTERM <proxy-pid>
# Should log "shutdown signal received, draining connections..." and exit
```

**Step 4: Final commit and tag**

```bash
git add -A
git commit -m "chore: production readiness complete"
```

---

## Summary

| Task | WS | Description | Est. |
|------|----|-------------|------|
| 1 | WS1 | Graceful shutdown | 5 min |
| 2 | WS1 | Rich health probes (/health JSON, /readyz) | 20 min |
| 3 | WS1 | ONNX model in Docker image | 10 min |
| 4 | WS2 | Add OpenTelemetry dependencies | 5 min |
| 5 | WS2 | HTTP metrics middleware | 15 min |
| 6 | WS2 | PII metrics in chat handler | 15 min |
| 7 | WS2 | Prometheus /metrics endpoint + OTLP init | 20 min |
| 8 | WS2 | Structured JSON logging | 10 min |
| 9 | WS2 | Request ID middleware | 15 min |
| 10 | WS2 | Grafana dashboard + Prometheus config | 20 min |
| 11 | WS3 | Shared masking pipeline refactor | 15 min |
| 12 | WS3 | Anthropic handler | 30 min |
| 13 | WS3 | Anthropic integration tests | 20 min |
| 14 | WS4 | Integration tests in CI | 10 min |
| 15 | WS4 | Docker image push in release | 10 min |
| 16 | WS4 | Documentation updates | 15 min |
| 17 | WS4 | Final verification | 10 min |
