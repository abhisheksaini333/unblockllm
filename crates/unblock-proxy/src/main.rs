//! unblock-proxy: HTTP proxy that redacts PII before forwarding to LLM APIs.
//! Phase 3: Redis state, Postgres audit, policy engine.
//! Sentry: set SENTRY_DSN to enable error tracking.

use axum::routing::{get, post};
use axum::Router;
use opentelemetry::metrics::MeterProvider;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use unblock_proxy::audit::{pool_from_env, AuditLog};
use unblock_proxy::handlers::chat::{chat_completions, init_pii_metrics, ChatState};
use unblock_proxy::handlers::health::{health_check, readiness_check, HealthState};
use unblock_proxy::middleware::metrics::{init_http_metrics, metrics_layer};
use unblock_proxy::middleware::request_id::request_id_layer;
use unblock_proxy::ner::NerEngine;
use unblock_proxy::policy::PolicyEngine;
use unblock_proxy::state::Store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    let _sentry = sentry::init(sentry::ClientOptions {
        attach_stacktrace: true,
        ..Default::default()
    });
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

    // Prometheus + OpenTelemetry metrics
    let prometheus_registry = prometheus::Registry::new();
    let prometheus_exporter = opentelemetry_prometheus::exporter()
        .with_registry(prometheus_registry.clone())
        .build()
        .map_err(|e| format!("prometheus exporter: {}", e))?;
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(prometheus_exporter)
        .build();
    let meter = meter_provider.meter("unblock-proxy");
    init_http_metrics(&meter);
    init_pii_metrics(&meter);

    let openai_base =
        std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string());

    let store = Store::from_env()
        .await
        .map_err(|e| format!("store: {}", e))?;
    let audit_pool = pool_from_env()
        .await
        .map_err(|e| format!("audit pool: {}", e))?;
    let audit = AuditLog::new(audit_pool);

    let policy_path =
        std::env::var("POLICY_CONFIG_PATH").unwrap_or_else(|_| "config/policy.yaml".to_string());
    let policy_path = PathBuf::from(&policy_path);
    let policy = PolicyEngine::load(&policy_path);
    if let Some(dashboard_url) = std::env::var("DASHBOARD_URL")
        .ok()
        .filter(|u| !u.is_empty())
    {
        let proxy_api_key = std::env::var("PROXY_API_KEY").unwrap_or_default();
        tracing::info!(dashboard_url = %dashboard_url, "policy: remote poll from dashboard");
        unblock_proxy::policy::spawn_remote_poll(policy.clone(), dashboard_url, proxy_api_key);
    } else {
        tracing::info!(path = %policy_path.display(), "policy: local file watch (reload every 5s)");
        policy.clone().spawn_reload(policy_path.clone(), 5);
    }

    let ner = NerEngine::from_env().map_err(|e| format!("NER engine: {}", e))?;
    let ner_loaded = ner.has_onnx_model();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let state = Arc::new(ChatState {
        store,
        ner,
        client,
        openai_base,
        audit,
        policy,
    });

    let health_state = Arc::new(HealthState {
        store: state.store.clone(),
        audit: state.audit.clone(),
        ner_loaded,
    });

    let chat_route =
        if let Some(redis_url) = std::env::var("REDIS_URL").ok().filter(|u| !u.is_empty()) {
            let layer = unblock_proxy::middleware::rate_limit::rate_limit_layer_redis(&redis_url)
                .await
                .map_err(|e| format!("rate limit (Redis): {}", e))?;
            Router::new()
                .route("/v1/chat/completions", post(chat_completions))
                .layer(layer)
                .with_state(state)
        } else {
            let layer = unblock_proxy::middleware::rate_limit::rate_limit_layer()
                .map_err(|e| format!("rate limit layer: {}", e))?;
            Router::new()
                .route("/v1/chat/completions", post(chat_completions))
                .layer(layer)
                .with_state(state)
        };

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
                [(
                    axum::http::header::CONTENT_TYPE,
                    "text/plain; charset=utf-8",
                )],
                buffer,
            )
        }
    };

    let readyz_route = Router::new()
        .route("/readyz", get(readiness_check))
        .with_state(health_state);
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .merge(readyz_route)
        .merge(chat_route)
        .layer(axum::middleware::from_fn(metrics_layer))
        .layer(axum::middleware::from_fn(request_id_layer));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!(%addr, version = env!("CARGO_PKG_VERSION"), "unblock-proxy listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let make_svc = app.into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, make_svc)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    tracing::info!("unblock-proxy shut down");
    Ok(())
}

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
