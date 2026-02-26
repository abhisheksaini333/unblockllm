#!/usr/bin/env bash
# API abuse: rate limiting and auth bypass. Synthetic/expected failures only.
set -e
PROXY_URL="${PROXY_URL:-http://127.0.0.1:8080}"
DASHBOARD_URL="${DASHBOARD_URL:-http://127.0.0.1:3000}"

echo "[*] API abuse pen-test"
echo "[*] Proxy: $PROXY_URL | Dashboard: $DASHBOARD_URL"

# 1) Policy API without key — must 401
code=$(curl -s -o /dev/null -w "%{http_code}" "$DASHBOARD_URL/api/v1/policy" 2>/dev/null || true)
echo "[test 1] GET /api/v1/policy without X-Proxy-API-Key — HTTP $code (expected 401)"
[ "$code" = "401" ] && echo "[PASS]" || echo "[CHECK] Expected 401"

# 2) Policy API with wrong key — must 401
code=$(curl -s -o /dev/null -w "%{http_code}" -H "X-Proxy-API-Key: wrong-key" "$DASHBOARD_URL/api/v1/policy" 2>/dev/null || true)
echo "[test 2] GET /api/v1/policy with wrong key — HTTP $code (expected 401)"
[ "$code" = "401" ] && echo "[PASS]" || echo "[CHECK] Expected 401"

# 3) Stats without session — must 401
code=$(curl -s -o /dev/null -w "%{http_code}" "$DASHBOARD_URL/api/v1/stats" 2>/dev/null || true)
echo "[test 3] GET /api/v1/stats unauthenticated — HTTP $code (expected 401)"
[ "$code" = "401" ] && echo "[PASS]" || echo "[CHECK] Expected 401"

# 4) Proxy chat without auth — may 401 or 200 depending on proxy config
code=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$PROXY_URL/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"hi"}]}' 2>/dev/null || true)
echo "[test 4] POST /v1/chat/completions no Bearer — HTTP $code"

# 5) Rapid requests (rate limit check) — 10 quick requests
echo "[test 5] 10 rapid requests to proxy..."
for i in 1 2 3 4 5 6 7 8 9 10; do
  curl -s -o /dev/null -w "%{http_code}\n" -X POST "$PROXY_URL/v1/chat/completions" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer ${OPENAI_API_KEY:-sk-test}" \
    -d '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"ping"}]}' &
done
wait
echo "[*] Rate limiting: if configured, excess requests should get 429. See REPORT.md."
