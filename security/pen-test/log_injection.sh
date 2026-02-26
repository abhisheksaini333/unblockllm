#!/usr/bin/env bash
# Log injection attempts — try to force PII-like content into logs. SYNTHETIC only.
set -e
PROXY_URL="${PROXY_URL:-http://127.0.0.1:8080}"
API_KEY="${OPENAI_API_KEY:-sk-test-key}"

echo "[*] Log injection pen-test (synthetic payloads)"
echo "[*] Proxy: $PROXY_URL"

# Payloads that might get logged as-is if logging is naive (newlines, control chars, etc.)
# We use clearly synthetic strings so no real PII is ever used.
run_req() {
  local body="$1"
  curl -s -o /tmp/pen_inj.json -w "%{http_code}" \
    -X POST "$PROXY_URL/v1/chat/completions" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $API_KEY" \
    -d "$body"
}

# 1) Newline in content — ensure logs don't dump full message
code=$(run_req '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"hello\nINJECTED_LINE_SYNTHETIC_123"}]}')
echo "[test 1] Newline in content — HTTP $code"

# 2) JSON escape sequences — no raw PII
code=$(run_req '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"test \\u0040 example.com"}]}')
echo "[test 2] Unicode escape — HTTP $code"

# 3) Very long content (no PII) — ensure no truncation that leaks
code=$(run_req "{\"model\":\"gpt-4o-mini\",\"messages\":[{\"role\":\"user\",\"content\":\"$(printf 'A%.0s' {1..5000})\"}]}")
echo "[test 3] Large payload — HTTP $code"

echo "[*] Log injection tests done. Verify proxy logs contain no raw user content (only counts/IDs). See REPORT.md."
