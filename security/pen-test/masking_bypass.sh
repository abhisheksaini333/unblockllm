#!/usr/bin/env bash
# Masking bypass attempts using SYNTHETIC data only. No real PII.
set -e
PROXY_URL="${PROXY_URL:-http://127.0.0.1:8080}"
API_KEY="${OPENAI_API_KEY:-sk-test-key}"

echo "[*] Masking bypass pen-test (synthetic data only)"
echo "[*] Proxy: $PROXY_URL"

# Synthetic patterns that might confuse simple regex (obfuscated email/phone)
# Expected: proxy should still mask or block; we check response does not contain raw value.
SYNTH_EMAIL="testuser@example-dot-com"
SYNTH_EMAIL2="user [at] example [dot] com"
SYNTH_PHONE="555.0100"
SYNTH_SSN="000-00-0000"

run_req() {
  local content="$1"
  curl -s -o /tmp/pen_resp.json -w "%{http_code}" \
    -X POST "$PROXY_URL/v1/chat/completions" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $API_KEY" \
    -d "{\"model\":\"gpt-4o-mini\",\"messages\":[{\"role\":\"user\",\"content\":\"$content\"}],\"stream\":false}"
}

# 1) Standard synthetic email/phone — should be masked in response
code=$(run_req "Contact me at test@example.com or 555-123-4567.")
echo "[test 1] Standard synthetic — HTTP $code"
if [ "$code" = "200" ]; then
  if grep -q "test@example.com" /tmp/pen_resp.json 2>/dev/null; then
    echo "[FAIL] Raw email appeared in response"
  else
    echo "[PASS] No raw email in response"
  fi
fi

# 2) Obfuscated-style text (no valid regex match) — no PII in request
code=$(run_req "My id is user at example dot com.")
echo "[test 2] Obfuscated text — HTTP $code"
echo "[INFO] Expected: 200; response may or may not mask 'at' patterns (design choice)."

# 3) SSN-like pattern (synthetic)
code=$(run_req "SSN: 111-22-3333 for testing.")
echo "[test 3] Synthetic SSN — HTTP $code"
if [ "$code" = "200" ]; then
  if grep -q "111-22-3333" /tmp/pen_resp.json 2>/dev/null; then
    echo "[FAIL] Raw SSN appeared in response"
  else
    echo "[PASS] No raw SSN in response"
  fi
fi

echo "[*] Masking bypass tests done. See REPORT.md for findings."
