# Penetration Test Report (Phase 6)

**Scope:** Masking bypass, log extraction, API abuse. **All tests use synthetic data only; no real PII.**

## Executive Summary

- **Masking bypass:** Tests send obfuscated or standard synthetic PII patterns; expected control is that responses and logs never contain raw PII.
- **Log injection:** Tests send newlines, unicode escapes, and large payloads; expected control is that application logs contain only metadata (counts, request_id), never user content.
- **API abuse:** Policy and Stats endpoints must return 401 when unauthenticated; rate limiting (when enabled) should return 429 under excess load.

## Findings and Remediations

| ID  | Category        | Test description                          | Expected control                         | Remediation if failed |
|-----|-----------------|--------------------------------------------|------------------------------------------|------------------------|
| M1  | Masking bypass  | Standard synthetic email/phone in message  | Response body has no raw email/phone     | Ensure NER+regex cover and mask before forwarding; never log content |
| M2  | Masking bypass  | Obfuscated "user at example dot com"        | No requirement to mask (design choice)    | Document as accepted limitation or add heuristic rules |
| M3  | Masking bypass  | Synthetic SSN in message                   | Response body has no raw SSN             | Block or mask SSN per policy; never echo in logs |
| L1  | Log injection   | Newline / control chars in content         | Logs do not contain user message content | Log only request_id, entity_count, entity_types; never log `content` |
| L2  | Log injection   | Unicode escapes in JSON                   | Parsed safely; no injection into logs     | Use structured logging; never concatenate user input into log strings |
| L3  | Log injection   | Very large payload                         | No truncation that leaks into logs       | Limit request size if needed; log length/count only |
| A1  | API abuse       | GET /api/v1/policy without key             | 401 Unauthorized                         | Enforce X-Proxy-API-Key; reject missing/wrong key |
| A2  | API abuse       | GET /api/v1/stats without session          | 401 Unauthorized                         | Require NextAuth session or valid proxy key |
| A3  | API abuse       | Rapid requests to proxy                    | 429 when rate limit exceeded (if enabled)| Add rate limiting middleware; document limit |

## Evidence

- Scripts: `masking_bypass.sh`, `log_injection.sh`, `api_abuse.sh`.
- Run with proxy (and dashboard) up; capture HTTP codes and optionally response bodies. No real PII is used in any script.

## Compliance Note

- Access control (policy/stats): Evidence of authentication (CC6.1).
- Audit logging: Evidence that only non-PII metadata is logged (CC7.2).
- Input handling: Evidence against log injection (CC7.1).

---

**Phase 6 pen-test complete.** Re-run after any change to masking, logging, or auth.
