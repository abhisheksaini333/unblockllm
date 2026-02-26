//! unblock-proxy: HTTP proxy that redacts PII before forwarding to LLM APIs.
//! Phase 3: Redis state, Postgres audit, policy engine.
//! Sentry: set SENTRY_DSN to enable error tracking.

use unblock_proxy::audit::{pool_from_env, AuditLog};
use unblock_proxy::handlers::chat::{chat_completions, ChatState};
use unblock_proxy::ner::NerEngine;
use unblock_proxy::policy::PolicyEngine;
use unblock_proxy::state::Store;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _sentry = sentry::init(sentry::ClientOptions {
    attach_stacktrace: true,
    ..Default::default()
});
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let openai_base =
        std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string());

    let store = Store::from_env().await.map_err(|e| format!("store: {}", e))?;
    let audit_pool = pool_from_env().await.map_err(|e| format!("audit pool: {}", e))?;
    let audit = AuditLog::new(audit_pool);

    let policy_path = std::env::var("POLICY_CONFIG_PATH")
        .unwrap_or_else(|_| "config/policy.yaml".to_string());
    let policy_path = PathBuf::from(&policy_path);
    let policy = PolicyEngine::load(&policy_path);
    if let Ok(dashboard_url) = std::env::var("DASHBOARD_URL") {
        let proxy_api_key = std::env::var("PROXY_API_KEY").unwrap_or_default();
        unblock_proxy::policy::spawn_remote_poll(policy.clone(), dashboard_url, proxy_api_key);
    } else {
        policy.clone().spawn_reload(policy_path.clone(), 5);
    }

    let ner = NerEngine::from_env().map_err(|e| format!("NER engine: {}", e))?;
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

    let chat_route = if let Some(redis_url) = std::env::var("REDIS_URL").ok().filter(|u| !u.is_empty()) {
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

    let app = Router::new()
        .route("/health", get(health))
        .merge(chat_route);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!(%addr, "unblock-proxy listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let make_svc = app.into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, make_svc).await?;
    Ok(())
}

/// Health check for load balancers and CI. No PII.
async fn health() -> &'static str {
    "ok"
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::Router;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_ok() {
        let app = Router::new().route("/health", axum::routing::get(super::health));
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), b"ok");
    }
}
