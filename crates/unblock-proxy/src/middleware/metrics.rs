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
