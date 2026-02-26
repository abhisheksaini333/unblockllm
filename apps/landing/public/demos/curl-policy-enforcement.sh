#!/usr/bin/env bash
# UnblockLLM policy enforcement demo — valid, copy-pasteable.
# Replace OPENAI_API_KEY and UNBLOCKLLM_BASE_URL with your values.

curl -X POST "${UNBLOCKLLM_BASE_URL:-https://127.0.0.1:8080}/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${OPENAI_API_KEY}" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [
      {"role": "user", "content": "My SSN is 123-45-6789. What is it?"}
    ]
  }'
# Expected: 400 or redacted response per policy (e.g. block SSN).
