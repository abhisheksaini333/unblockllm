//! Integration tests: spin up the proxy with a mock LLM backend.
//! No real OpenAI key needed. Validates masking, re-identification,
//! policy block enforcement, and audit logging end-to-end.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use std::sync::Arc;
use tower::ServiceExt;

use unblock_proxy::audit::AuditLog;
use unblock_proxy::handlers::anthropic::anthropic_messages;
use unblock_proxy::handlers::chat::{chat_completions, ChatState};
use unblock_proxy::ner::NerEngine;
use unblock_proxy::policy::PolicyEngine;
use unblock_proxy::state::Store;

/// Build a test router pointing at a mock OpenAI base URL.
fn build_app(openai_base: &str) -> axum::Router {
    let state = Arc::new(ChatState {
        store: Store::new_memory(),
        ner: NerEngine::from_env().expect("ner"),
        client: reqwest::Client::new(),
        openai_base: openai_base.to_string(),
        anthropic_base: "http://localhost:1".to_string(),
        audit: AuditLog::new(None),
        policy: PolicyEngine::load(std::path::Path::new("config/policy.yaml")),
    });
    axum::Router::new()
        .route(
            "/v1/chat/completions",
            axum::routing::post(chat_completions),
        )
        .route("/health", axum::routing::get(|| async { "ok" }))
        .with_state(state)
}

/// Build a test router with a custom policy that blocks EMAIL.
fn build_app_with_block(openai_base: &str) -> axum::Router {
    let state = Arc::new(ChatState {
        store: Store::new_memory(),
        ner: NerEngine::from_env().expect("ner"),
        client: reqwest::Client::new(),
        openai_base: openai_base.to_string(),
        anthropic_base: "http://localhost:1".to_string(),
        audit: AuditLog::new(None),
        policy: PolicyEngine::from_config(
            vec![
                "EMAIL".to_string(),
                "PHONE".to_string(),
                "PERSON".to_string(),
            ],
            vec!["EMAIL".to_string()],
        ),
    });
    axum::Router::new()
        .route(
            "/v1/chat/completions",
            axum::routing::post(chat_completions),
        )
        .with_state(state)
}

/// Start a mock OpenAI server that echoes back the masked content it receives.
/// Returns the server URL and a handle to shut it down.
async fn start_mock_openai() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let url = format!("http://127.0.0.1:{}", port);

    let handle = tokio::spawn(async move {
        let app = axum::Router::new().route(
            "/v1/chat/completions",
            axum::routing::post(|body: axum::extract::Json<serde_json::Value>| async move {
                // Echo back the (masked) content from the first message
                let content = body
                    .get("messages")
                    .and_then(|m| m.as_array())
                    .and_then(|a| a.last())
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("no content");

                let response = serde_json::json!({
                    "id": "chatcmpl-test",
                    "object": "chat.completion",
                    "choices": [{
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": format!("Hello! You said: {}", content)
                        },
                        "finish_reason": "stop"
                    }],
                    "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
                });
                axum::Json(response)
            }),
        );
        axum::serve(listener, app).await.ok();
    });

    // Give server time to start
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    (url, handle)
}

/// Start a mock OpenAI SSE streaming server.
/// Echoes back the masked content across multiple SSE chunks.
async fn start_mock_openai_streaming() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let url = format!("http://127.0.0.1:{}", port);

    let handle = tokio::spawn(async move {
        let app = axum::Router::new().route(
            "/v1/chat/completions",
            axum::routing::post(|body: axum::extract::Json<serde_json::Value>| async move {
                let content = body
                    .get("messages")
                    .and_then(|m| m.as_array())
                    .and_then(|a| a.last())
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("no content")
                    .to_string();

                // Split content into words and stream each as an SSE chunk
                let words: Vec<String> =
                    content.split_whitespace().map(|w| w.to_string()).collect();
                let mut chunks = Vec::new();
                for (i, word) in words.iter().enumerate() {
                    let delta_content = if i == 0 {
                        word.clone()
                    } else {
                        format!(" {}", word)
                    };
                    let chunk = format!(
                        "data: {}\n\n",
                        serde_json::json!({
                            "id": "chatcmpl-stream",
                            "object": "chat.completion.chunk",
                            "choices": [{
                                "index": 0,
                                "delta": {"content": delta_content},
                                "finish_reason": null
                            }]
                        })
                    );
                    chunks.push(chunk);
                }
                chunks.push("data: [DONE]\n\n".to_string());

                let body = chunks.join("");
                (
                    [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
                    body,
                )
                    .into_response()
            }),
        );
        axum::serve(listener, app).await.ok();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    (url, handle)
}

// ── Tests ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn health_returns_ok() {
    let app = build_app("http://localhost:1");
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn invalid_json_returns_400() {
    let app = build_app("http://localhost:1");
    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from("not json"))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn non_streaming_masks_and_reidentifies_email() {
    let (mock_url, _handle) = start_mock_openai().await;
    let app = build_app(&mock_url);

    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", "Bearer sk-test")
        .body(Body::from(
            serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": "Email me at user@example.com please."}],
                "stream": false
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .expect("content should be a string");

    // The mock echoes back the masked input — proxy should re-identify it
    // Re-identified response should contain the original email
    assert!(
        content.contains("user@example.com"),
        "Expected re-identified email in response, got: {}",
        content
    );
}

#[tokio::test]
async fn non_streaming_masks_phone_and_ssn() {
    let (mock_url, _handle) = start_mock_openai().await;
    let app = build_app(&mock_url);

    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", "Bearer sk-test")
        .body(Body::from(
            serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": "Call 555-123-4567 or SSN 123-45-6789."}],
                "stream": false
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .expect("content");

    // Re-identified response should contain the original PII
    assert!(
        content.contains("555-123-4567"),
        "Expected re-identified phone, got: {}",
        content
    );
    assert!(
        content.contains("123-45-6789"),
        "Expected re-identified SSN, got: {}",
        content
    );
}

#[tokio::test]
async fn policy_block_rejects_email() {
    let (mock_url, _handle) = start_mock_openai().await;
    let app = build_app_with_block(&mock_url);

    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", "Bearer sk-test")
        .body(Body::from(
            serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": "Email me at user@example.com please."}],
                "stream": false
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // Policy block should return 400 Bad Request
    assert_eq!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "Policy block should reject requests containing EMAIL"
    );
}

#[tokio::test]
async fn no_pii_passes_through_cleanly() {
    let (mock_url, _handle) = start_mock_openai().await;
    let app = build_app(&mock_url);

    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", "Bearer sk-test")
        .body(Body::from(
            serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": "What is the capital of France?"}],
                "stream": false
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .expect("content");

    // No PII → content passes through unchanged
    assert!(
        content.contains("What is the capital of France?"),
        "Expected clean passthrough, got: {}",
        content
    );
}

#[tokio::test]
async fn upstream_error_propagates_status() {
    // Point at a server that doesn't exist
    let app = build_app("http://127.0.0.1:1");

    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", "Bearer sk-test")
        .body(Body::from(
            serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": "hello"}],
                "stream": false
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // Should get 502 Bad Gateway when upstream is unreachable
    assert_eq!(res.status(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn streaming_masks_and_reidentifies_email() {
    let (mock_url, _handle) = start_mock_openai_streaming().await;
    let app = build_app(&mock_url);

    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", "Bearer sk-test")
        .body(Body::from(
            serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": "Email me at user@example.com please."}],
                "stream": true
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Collect the full streaming body
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body);

    // The SSE stream should contain re-identified email (not masked placeholder)
    assert!(
        text.contains("user@example.com"),
        "Expected re-identified email in SSE stream, got: {}",
        text
    );
    // Should not contain any masked placeholder pattern like <<EMAIL_...>>
    assert!(
        !text.contains("<<EMAIL_"),
        "SSE stream should not contain masked placeholders, got: {}",
        text
    );
}

#[tokio::test]
async fn streaming_no_pii_passes_through() {
    let (mock_url, _handle) = start_mock_openai_streaming().await;
    let app = build_app(&mock_url);

    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("content-type", "application/json")
        .header("authorization", "Bearer sk-test")
        .body(Body::from(
            serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": "What is 2+2?"}],
                "stream": true
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body);

    // Clean text passes through unchanged in SSE
    assert!(
        text.contains("What") && text.contains("2+2?"),
        "Expected clean passthrough in stream, got: {}",
        text
    );
}

// ── Anthropic Integration Tests ──────────────────────────────────

/// Build a test router for Anthropic Messages API.
fn build_app_anthropic(anthropic_base: &str) -> axum::Router {
    let state = Arc::new(ChatState {
        store: Store::new_memory(),
        ner: NerEngine::from_env().expect("ner"),
        client: reqwest::Client::new(),
        openai_base: "http://localhost:1".to_string(),
        anthropic_base: anthropic_base.to_string(),
        audit: AuditLog::new(None),
        policy: PolicyEngine::load(std::path::Path::new("config/policy.yaml")),
    });
    axum::Router::new()
        .route("/v1/messages", axum::routing::post(anthropic_messages))
        .with_state(state)
}

/// Start a mock Anthropic Messages API server (non-streaming).
/// Echoes back the masked content it receives in Anthropic response format.
async fn start_mock_anthropic() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let url = format!("http://127.0.0.1:{}", port);

    let handle = tokio::spawn(async move {
        let app = axum::Router::new().route(
            "/v1/messages",
            axum::routing::post(|body: axum::extract::Json<serde_json::Value>| async move {
                let content = body
                    .get("messages")
                    .and_then(|m| m.as_array())
                    .and_then(|a| a.last())
                    .and_then(|m| m.get("content"))
                    .map(|c| {
                        if let Some(s) = c.as_str() {
                            s.to_string()
                        } else if let Some(blocks) = c.as_array() {
                            blocks
                                .iter()
                                .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                                .collect::<Vec<_>>()
                                .join("\n")
                        } else {
                            "no content".to_string()
                        }
                    })
                    .unwrap_or_else(|| "no content".to_string());

                let response = serde_json::json!({
                    "id": "msg_test",
                    "type": "message",
                    "role": "assistant",
                    "content": [{
                        "type": "text",
                        "text": format!("Hello! You said: {}", content)
                    }],
                    "model": "claude-3-haiku-20240307",
                    "stop_reason": "end_turn",
                    "usage": {"input_tokens": 10, "output_tokens": 5}
                });
                axum::Json(response)
            }),
        );
        axum::serve(listener, app).await.ok();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    (url, handle)
}

/// Start a mock Anthropic SSE streaming server.
async fn start_mock_anthropic_streaming() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let url = format!("http://127.0.0.1:{}", port);

    let handle = tokio::spawn(async move {
        let app = axum::Router::new().route(
            "/v1/messages",
            axum::routing::post(|body: axum::extract::Json<serde_json::Value>| async move {
                let content = body
                    .get("messages")
                    .and_then(|m| m.as_array())
                    .and_then(|a| a.last())
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("no content")
                    .to_string();

                let words: Vec<String> =
                    content.split_whitespace().map(|w| w.to_string()).collect();
                let mut chunks = Vec::new();

                // message_start event
                chunks.push("event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_stream\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-3-haiku-20240307\"}}\n\n".to_string());
                // content_block_start
                chunks.push("event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n".to_string());

                for (i, word) in words.iter().enumerate() {
                    let delta = if i == 0 {
                        word.clone()
                    } else {
                        format!(" {}", word)
                    };
                    chunks.push(format!(
                        "event: content_block_delta\ndata: {}\n\n",
                        serde_json::json!({
                            "type": "content_block_delta",
                            "index": 0,
                            "delta": {"type": "text_delta", "text": delta}
                        })
                    ));
                }

                chunks.push("event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\n".to_string());
                chunks.push("event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string());

                let body = chunks.join("");
                (
                    [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
                    body,
                )
                    .into_response()
            }),
        );
        axum::serve(listener, app).await.ok();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    (url, handle)
}

#[tokio::test]
async fn anthropic_non_streaming_masks_email() {
    let (mock_url, _handle) = start_mock_anthropic().await;
    let app = build_app_anthropic(&mock_url);

    let req = Request::builder()
        .uri("/v1/messages")
        .method("POST")
        .header("content-type", "application/json")
        .header("x-api-key", "sk-ant-test")
        .header("anthropic-version", "2023-06-01")
        .body(Body::from(
            serde_json::json!({
                "model": "claude-3-haiku-20240307",
                "max_tokens": 1024,
                "messages": [{"role": "user", "content": "Email me at user@example.com please."}],
                "stream": false
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let text = json["content"][0]["text"]
        .as_str()
        .expect("text should be a string");

    assert!(
        text.contains("user@example.com"),
        "Expected re-identified email in Anthropic response, got: {}",
        text
    );
}

#[tokio::test]
async fn anthropic_content_blocks_masks_email() {
    let (mock_url, _handle) = start_mock_anthropic().await;
    let app = build_app_anthropic(&mock_url);

    let req = Request::builder()
        .uri("/v1/messages")
        .method("POST")
        .header("content-type", "application/json")
        .header("x-api-key", "sk-ant-test")
        .header("anthropic-version", "2023-06-01")
        .body(Body::from(
            serde_json::json!({
                "model": "claude-3-haiku-20240307",
                "max_tokens": 1024,
                "messages": [{
                    "role": "user",
                    "content": [{"type": "text", "text": "Email me at user@example.com please."}]
                }],
                "stream": false
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let text = json["content"][0]["text"]
        .as_str()
        .expect("text should be a string");

    assert!(
        text.contains("user@example.com"),
        "Expected re-identified email with content blocks, got: {}",
        text
    );
}

#[tokio::test]
async fn anthropic_streaming_masks_email() {
    let (mock_url, _handle) = start_mock_anthropic_streaming().await;
    let app = build_app_anthropic(&mock_url);

    let req = Request::builder()
        .uri("/v1/messages")
        .method("POST")
        .header("content-type", "application/json")
        .header("x-api-key", "sk-ant-test")
        .header("anthropic-version", "2023-06-01")
        .body(Body::from(
            serde_json::json!({
                "model": "claude-3-haiku-20240307",
                "max_tokens": 1024,
                "messages": [{"role": "user", "content": "Email me at user@example.com please."}],
                "stream": true
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body);

    assert!(
        text.contains("user@example.com"),
        "Expected re-identified email in Anthropic SSE stream, got: {}",
        text
    );
    assert!(
        !text.contains("<<EMAIL_"),
        "SSE stream should not contain masked placeholders, got: {}",
        text
    );
}

#[tokio::test]
async fn anthropic_no_pii_passes_through() {
    let (mock_url, _handle) = start_mock_anthropic().await;
    let app = build_app_anthropic(&mock_url);

    let req = Request::builder()
        .uri("/v1/messages")
        .method("POST")
        .header("content-type", "application/json")
        .header("x-api-key", "sk-ant-test")
        .header("anthropic-version", "2023-06-01")
        .body(Body::from(
            serde_json::json!({
                "model": "claude-3-haiku-20240307",
                "max_tokens": 1024,
                "messages": [{"role": "user", "content": "What is the capital of France?"}],
                "stream": false
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let text = json["content"][0]["text"]
        .as_str()
        .expect("text should be a string");

    assert!(
        text.contains("What is the capital of France?"),
        "Expected clean passthrough in Anthropic response, got: {}",
        text
    );
}
