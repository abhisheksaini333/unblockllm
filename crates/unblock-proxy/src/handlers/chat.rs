//! Handler for POST /v1/chat/completions: mask PII, forward to OpenAI, re-identify response (SSE + non-streaming).
//! Phase 3: policy filter, audit log (metadata only).

use crate::audit::AuditLog;
use crate::masking::{mask_text, reidentify_map};
use crate::ner::NerEngine;
use crate::policy::{filter_spans_by_policy, PolicyEngine};
use crate::state::{StateError, Store};
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use bytes::Bytes;
use futures_util::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use opentelemetry::metrics::{Counter, Histogram, Meter};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use tracing::info;
use unblock_core::EntityType;

pub struct PiiMetrics {
    pub entities_detected: Counter<u64>,
    pub masking_duration: Histogram<f64>,
    pub upstream_duration: Histogram<f64>,
}

pub static PII_METRICS: OnceLock<PiiMetrics> = OnceLock::new();

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

/// Map store errors to HTTP 503 (Block Mode when Redis unavailable).
pub fn map_store_error(e: StateError) -> (StatusCode, String) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        format!("Store unavailable: {}", e),
    )
}

fn entity_type_tag(t: &EntityType) -> String {
    match t {
        EntityType::Person => "PERSON".to_string(),
        EntityType::Organization => "ORGANIZATION".to_string(),
        EntityType::Location => "LOCATION".to_string(),
        EntityType::Email => "EMAIL".to_string(),
        EntityType::Phone => "PHONE".to_string(),
        EntityType::Ssn => "SSN".to_string(),
        EntityType::Date => "DATE".to_string(),
        EntityType::Custom(s) => s.clone(),
    }
}

/// OpenAI chat request: we only need to mutate messages[].content; preserve rest as Value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiChatRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
}

/// Shared app state.
pub struct ChatState {
    pub store: Store,
    pub ner: NerEngine,
    pub client: Client,
    pub openai_base: String,
    pub audit: AuditLog,
    pub policy: PolicyEngine,
}

/// Result of the shared masking pipeline.
pub struct MaskResult {
    pub request_id: String,
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
        mask_elapsed,
    })
}

/// POST /v1/chat/completions
pub async fn chat_completions(
    State(state): State<Arc<ChatState>>,
    req: Request<Body>,
) -> Result<Response, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut body_value: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;
    let mut chat: OpenAiChatRequest = serde_json::from_value(body_value.clone()).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid chat request: {}", e),
        )
    })?;

    let mask_result = mask_messages(&mut chat.messages, &state).await?;
    let request_id = mask_result.request_id;

    // Write masked messages back into body for forwarding
    if let Some(obj) = body_value.as_object_mut() {
        let messages_val = serde_json::to_value(&chat.messages)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        obj.insert("messages".to_string(), messages_val);
    }

    let url = format!(
        "{}/v1/chat/completions",
        state.openai_base.trim_end_matches('/')
    );
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
    if let Some(pii) = PII_METRICS.get() {
        pii.upstream_duration
            .record(upstream_start.elapsed().as_secs_f64(), &[]);
    }

    let status = upstream.status();
    let mut headers = upstream.headers().clone();

    if chat.stream {
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
    if let Some(m) = mapping {
        if let Some(choices) = resp_body.get_mut("choices").and_then(|c| c.as_array_mut()) {
            if let Some(first) = choices.first_mut() {
                if let Some(content) = first.get_mut("message").and_then(|m| m.get_mut("content")) {
                    if let Some(s) = content.as_str() {
                        *content = serde_json::Value::String(reidentify_map(s, &m));
                    }
                }
            }
        }
    }

    info!(
        request_id = %request_id,
        mask_ms = mask_result.mask_elapsed.as_millis(),
        status = %status,
        "chat completion done"
    );

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn chat_returns_400_for_invalid_json() {
        let state = Arc::new(ChatState {
            store: Store::new_memory(),
            ner: NerEngine::from_env().expect("ner"),
            client: reqwest::Client::new(),
            openai_base: "https://api.openai.com".to_string(),
            audit: AuditLog::new(None),
            policy: PolicyEngine::load(std::path::Path::new("/nonexistent")),
        });
        let app = axum::Router::new()
            .route(
                "/v1/chat/completions",
                axum::routing::post(chat_completions),
            )
            .with_state(state);
        let req = Request::builder()
            .uri("/v1/chat/completions")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from("not json"))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
