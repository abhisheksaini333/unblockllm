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
