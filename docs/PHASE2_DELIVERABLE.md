# Phase 2 Deliverable: Core Proxy Development

## File structure

```
crates/unblock-proxy/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Re-exports for bench
│   ├── main.rs             # Entry: axum server, routes, state
│   ├── handlers/
│   │   ├── mod.rs
│   │   └── chat.rs         # POST /v1/chat/completions, mask, forward, SSE + non-streaming
│   ├── ner/
│   │   ├── mod.rs
│   │   └── engine.rs       # Regex NER + spawn_blocking; ONNX optional via NER_MODEL_DIR
│   ├── masking/
│   │   └── mod.rs          # Regex patterns, mask_text, reidentify_map
│   └── state.rs            # In-memory mapping store (Phase 3: Redis)
└── benches/
    └── latency.rs          # Mask + reidentify overhead (<20ms target)
```

## Verification

### 1. Build and test

```bash
cd unblockllm
cargo build --release
cargo test
```

### 2. Latency benchmark (<20ms overhead)

```bash
cargo bench --bench latency
```

**Recorded results (release build, 1000 runs):**

| Metric | Value | Target |
|--------|--------|--------|
| mask_mean_us | 395.45 | — |
| reidentify_mean_us | 0.30 | — |
| **total_overhead_ms** | **0.3958** | **&lt;20** |

The benchmark measures regex mask + reidentify only (no network). Total overhead is well under 20ms.

### 3. Run proxy and test streaming (curl)

Set your OpenAI API key and start the proxy:

```bash
export OPENAI_API_KEY=sk-...
./target/release/unblock-proxy
```

In another terminal, non-streaming:

```bash
curl -s http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Reply with one sentence. My email is user@example.com."}],
    "stream": false
  }'
```

Expected: response content may contain `[EMAIL_1]` replaced back with `user@example.com` (re-identification). The request sent to OpenAI has `[EMAIL_1]` instead of the raw email.

Streaming:

```bash
curl -s -N http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Say hello. Contact me at test@foo.com."}],
    "stream": true
  }'
```

Expected: SSE stream; placeholders in the stream are re-identified before being sent to the client.

### 4. Override OpenAI base URL (e.g. mock or proxy)

```bash
export OPENAI_BASE_URL=https://api.openai.com
./target/release/unblock-proxy
```

## Security (Phase 2)

- No raw PII in logs; only `request_id`, `mask_ms`, `total_ms`, `status`, and entity counts.
- No `unwrap()` in production code; `?` and explicit error handling.
- Mapping is ephemeral (in-memory); removed after response is sent (Phase 3: Redis with TTL).
- Authorization header is forwarded to OpenAI; no keys logged.

## Latency mitigation (Phase 2)

- NER runs in `tokio::task::spawn_blocking` to avoid blocking the async runtime.
- Phase 2 uses regex-only NER (EMAIL, PHONE, SSN); ONNX can be enabled by setting `NER_MODEL_DIR` and adding the `ort` feature when the quantized model is available.
- Session options for future ONNX: `intra_op_num_threads(1)`, sequential execution, as documented in the NER engine.

---

**Phase 2 complete.** Proceed to Phase 3 (State & Compliance: Redis, PostgreSQL audit, policy engine).
