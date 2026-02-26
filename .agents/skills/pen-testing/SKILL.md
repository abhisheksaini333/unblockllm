---
name: pen-testing
description: Apply penetration testing and security validation for PII protection, log injection, masking bypass, and API abuse. Use when running security/pen-test scripts or validating that no raw PII appears in logs or responses.
---

# Pen Testing & Security Validation

Use this skill when **running pen-test scripts**, **validating PII protection**, or **checking for log injection / API abuse** in the unblockllm proxy and dashboard.

## When to Use

- Executing scripts in `security/pen-test/` (masking bypass, log injection, API abuse).
- Validating that proxy logs show only placeholders (e.g. `[EMAIL_1]`), never raw PII.
- Checking audit tables for metadata-only columns (no `content` or PII).
- Reviewing rate limiting and auth on chat/completions endpoints.
- Preparing or interpreting pen-test reports (e.g. `security/pen-test/REPORT.md`).

## In-Repo Assets

| Asset | Purpose |
|-------|---------|
| `security/pen-test/masking_bypass.sh` | Verify masking cannot be bypassed to leak PII. |
| `security/pen-test/log_injection.sh` | Check log injection / sanitization. |
| `security/pen-test/api_abuse.sh` | Rate limit and API abuse checks. |
| `security/pen-test/REPORT.md` | Pen-test findings and status. |
| `docs/SECURITY.md` | Known advisories (e.g. rsa) and accepted risks. |

## Pass/Fail Criteria (PII & Logs)

- **Pass:** Proxy logs and audit DB contain only request_id, user_id, entity_count, entity_types (and similar metadata). No raw email, phone, or other PII in logs or persisted content.
- **Fail:** Any raw PII in stdout/stderr, response body (unless re-identified for same request), or audit table content columns.

## Integration with Other Skills

- **owasp-security** — OWASP Top 10 (injection, auth, logging).
- **security-compliance** — Incident response, monitoring, audit.
- **security-compliance-audit** — Control evidence for SOC2/GDPR.

## Optional External Skill

For broader penetration testing methodology:
- `npx skills add aj-geddes/useful-ai-prompts@penetration-testing`
