# Penetration Testing (Phase 6)

**Synthetic data only.** No real PII. Use against a running proxy (and optionally dashboard).

- `masking_bypass.sh` — Attempt to bypass NER/regex masking with obfuscated patterns.
- `log_injection.sh` — Attempt to force PII into logs (injection payloads).
- `api_abuse.sh` — Rate limiting and auth bypass attempts on Dashboard/Proxy APIs.

Run with: `PROXY_URL=http://127.0.0.1:8080 DASHBOARD_URL=http://127.0.0.1:3000 ./masking_bypass.sh` (env optional).
