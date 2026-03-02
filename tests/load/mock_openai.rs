//! Standalone mock OpenAI server for load testing.
//! Responds to /v1/chat/completions with a canned response (non-streaming)
//! or SSE chunks (streaming). Echoes back the masked content it receives.
//!
//! Usage: cargo run --release --example mock_openai
//! Listens on 127.0.0.1:9090

use axum::response::IntoResponse;

#[tokio::main]
async fn main() {
    let app = axum::Router::new().route("/v1/chat/completions", axum::routing::post(handler));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9090")
        .await
        .expect("bind 9090");
    println!("Mock OpenAI listening on http://127.0.0.1:9090");
    axum::serve(listener, app).await.unwrap();
}

async fn handler(body: axum::extract::Json<serde_json::Value>) -> impl IntoResponse {
    let content = body
        .get("messages")
        .and_then(|m| m.as_array())
        .and_then(|a| a.last())
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("no content");

    let is_stream = body
        .get("stream")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);

    if is_stream {
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut chunks = Vec::new();
        for (i, word) in words.iter().enumerate() {
            let delta = if i == 0 {
                word.to_string()
            } else {
                format!(" {}", word)
            };
            chunks.push(format!(
                "data: {}\n\n",
                serde_json::json!({
                    "id": "chatcmpl-mock",
                    "object": "chat.completion.chunk",
                    "choices": [{"index": 0, "delta": {"content": delta}, "finish_reason": null}]
                })
            ));
        }
        chunks.push("data: [DONE]\n\n".to_string());
        (
            [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
            chunks.join(""),
        )
            .into_response()
    } else {
        let response = serde_json::json!({
            "id": "chatcmpl-mock",
            "object": "chat.completion",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": format!("Echo: {}", content)},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
        });
        axum::Json(response).into_response()
    }
}
