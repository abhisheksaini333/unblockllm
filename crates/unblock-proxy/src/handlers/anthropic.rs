//! Handler for POST /v1/messages: Anthropic Messages API with PII masking.
//! Mirrors the OpenAI handler pattern but with Anthropic-specific request/response formats.

use crate::handlers::chat::{map_store_error, mask_messages, ChatMessage, ChatState, PII_METRICS};
use crate::masking::reidentify_map;
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

    let url = format!("{}/v1/messages", state.anthropic_base.trim_end_matches('/'));

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
                    if let Some(text) = block
                        .get("text")
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string())
                    {
                        block["text"] = serde_json::Value::String(reidentify_map(&text, &m));
                    }
                }
            }
        }
    }

    tracing::info!(
        request_id = %request_id,
        mask_ms = mask_result.mask_elapsed.as_millis(),
        status = %status,
        "anthropic messages done"
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
