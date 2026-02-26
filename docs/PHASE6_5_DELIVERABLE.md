# Phase 6.5: Residual Risk Hardening & Enterprise Readiness

**Objective:** Address remaining residual risks from the Production Readiness Review. Move from "Launch Ready" to "Enterprise Contract Ready."

## Risk Remediation Summary

| Risk ID | Category | Status | Deliverable |
|---------|----------|--------|-------------|
| **SEC-01** | Dependency Vuln | Documented | Cargo.toml postgres-only; rsa (sqlx-mysql) documented in docs/SECURITY.md as Accepted Risk. |
| **SEC-02** | API Key Rotation | Done | `POST /api/v1/keys/rotate`, `GET /api/v1/keys/status`; docs/SECURITY.md. |
| **SEC-03** | TLS Boundary | Done | Middleware HTTPS redirect (x-forwarded-proto + direct http); docs/DEPLOYMENT.md (LB requirement). |
| **SEC-04** | Distributed Rate Limit | Done | Redis-backed rate limit when REDIS_URL set (main.rs + rate_limit.rs). |
| **COM-01** | Audit Log Retention | Done | migrations/003_audit_retention.sql; 90-day TTL documented in SECURITY_MODEL.md, DEPLOYMENT.md. |
| **COM-02** | Right to Erasure | Done | scripts/anonymize_user.py; COMPLIANCE_PACKET.md. |
| **COM-03** | Sub-processor Docs | Done | docs/SECURITY_MODEL.md (LLM providers, COM-03 note). |
| **PERF-01** | ONNX SLA | Done | docs/SLA.md (Regex <5ms, ONNX <50ms). |
| **PERF-02** | Redis Pool | Done | DEPLOYMENT.md (connection pool note, load test reference). |
| **PERF-03** | Bundle Analyzer | Done | npm run analyze; security/bundle-analysis/README.md; next.config.js withBundleAnalyzer. |
| **OPS-01** | Redis Fallback | Done | Block Mode (503) when Redis unreachable; SECURITY_MODEL.md, chat.rs map_store_error. |
| **OPS-02** | Secret Injection | Done | docs/DEPLOYMENT.md (K8s Secrets, AWS Secrets Manager). |
| **OPS-03** | Incident Response | Done | docs/INCIDENT_RESPONSE.md (4-step: Kill Switch, Notify, Patch, Post-Mortem). |

## Verification (Run Locally)

| Check | Command | Expected |
|-------|---------|----------|
| SEC-01 | `cargo audit` | 0 CRITICAL/HIGH; 1 MEDIUM (rsa) documented in SECURITY.md |
| SEC-02 | `curl -X POST .../api/v1/keys/rotate` (with session) | New key in JSON |
| SEC-03 | `curl -I http://localhost:3000` (production) | 307 → https |
| SEC-04 | 2 proxy instances + k6 | Shared rate limit (Redis keys) |
| COM-01 | Run retention DELETE | Old audit_logs removed |
| COM-02 | `python scripts/anonymize_user.py <user_id>` | user_id hashed in audit_logs |
| PERF-01 | Read docs/SLA.md | Regex <5ms, ONNX <50ms |
| OPS-01 | Kill Redis, send request | 503 from proxy |
| OPS-03 | Read INCIDENT_RESPONSE.md | 4-step process |

## Skills Applied

- **find-skills** (discovery): security-compliance, devops-incident-responder.
- **Project skills:** rust-best-practices, pen-testing, docker-expert, security-compliance (from .agents/skills).

---

**Phase 6.5 complete.** Enterprise Contract Ready. Proceed to Phase 7 (Launch Prep) on confirmation.
