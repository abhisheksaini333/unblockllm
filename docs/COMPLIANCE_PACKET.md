# SOC2 Type I Compliance Packet (Phase 6)

Evidence and mapping of controls for unblockllm (Zero-Trust Privacy Proxy and Control Plane). This document supports SOC2 Type I readiness.

---

## Control Mapping (CC1–CC9)

### CC1 – Control Environment

| Control | Evidence |
|--------|----------|
| Governance | SKILL.md and phase deliverables define non-negotiable constraints (no PII in logs, no unwrap in production, &lt;20ms overhead). |
| Structure | Monorepo layout: proxy (Rust), SDKs (Python/Node), dashboard (Next.js), docs. |
| Ethics | No real PII in tests or pen-test scripts; synthetic data only (see security/pen-test/). |

### CC2 – Communication

| Control | Evidence |
|--------|----------|
| Security expectations | README, SECURITY_MODEL.md, and PHASE*_DELIVERABLE.md document security and privacy expectations. |
| Reporting | Audit logs (Phase 3) and dashboard stats provide visibility; no PII in reports. |

### CC3 – Risk Assessment

| Control | Evidence |
|--------|----------|
| Risk identification | Pen-test REPORT.md (security/pen-test/REPORT.md) documents masking bypass, log injection, and API abuse tests. |
| Mitigation | Rate limiting (proxy), auth on dashboard APIs, policy-based block/mask. |

### CC4 – Monitoring

| Control | Evidence |
|--------|----------|
| Monitoring activities | Audit log table (entity_count, entity_types, request_id); dashboard /api/v1/stats for aggregate usage. |
| No PII in monitoring | Audit schema and SECURITY_MODEL.md: only metadata stored; no raw content or mapping values. |

### CC5 – Control Activities

| Control | Evidence |
|--------|----------|
| Access control | NextAuth (dashboard); X-Proxy-API-Key for /api/v1/policy; Bearer for proxy. |
| Encryption | TLS recommended (rediss://, postgres sslmode=require); env vars for secrets (no hardcoded secrets). |
| Change management | CI (.github/workflows/ci.yml): fmt, clippy, tests, build; changes tracked in version control. |

### CC6 – Logical and Physical Access

| Control | Evidence |
|--------|----------|
| Logical access | Dashboard: session-based auth; Policy API: API key. Proxy: API key to upstream. |
| RBAC | Dashboard users; policy config and stats restricted to authenticated users. |

### CC7 – System Operations

| Control | Evidence |
|--------|----------|
| Logging | Audit logs (Postgres); proxy logs (counts/IDs only, no PII). Log injection tests in pen-test. |
| Change management | CI runs on push; build and test gates. |

### CC8 – Change Management

| Control | Evidence |
|--------|----------|
| Changes | Code changes via version control; CI validates build and tests. |
| Documentation | PHASE*_DELIVERABLE.md and docs/ updated with each phase. |

### CC9 – Risk Mitigation

| Control | Evidence |
|--------|----------|
| Vendor / dependency risk | cargo audit, npm audit; SBOM: security/sbom.json (Dashboard), security/sbom-proxy.json (Proxy). Target: zero critical vulnerabilities. |
| Pen-test and load test | security/pen-test/, security/load-test/; findings in REPORT.md. |

---

## Evidence Summary

| Area | Location |
|------|----------|
| Audit log schema (no PII) | PHASE3_DELIVERABLE.md; crates/unblock-proxy/src/audit.rs |
| Encryption / TLS | PHASE3_DELIVERABLE.md; .env.example (rediss://, sslmode=require) |
| Access control | Dashboard NextAuth; PROXY_API_KEY; Phase 5 deliverable |
| Change management / CI | .github/workflows/ci.yml |
| Pen-test | security/pen-test/REPORT.md |
| Load test | security/load-test/REPORT.md |
| SBOM (Dashboard) | security/sbom.json |
| SBOM (Proxy/Rust) | security/sbom-proxy.json (cargo cyclonedx) |
| Dependency scans | cargo audit; npm audit (see security/README.md) |
| Right to Erasure (GDPR) | scripts/anonymize_user.py; SECURITY_MODEL.md; 90-day retention (migrations/003). |

---

**Phase 6 compliance packet complete.** Use for SOC2 Type I readiness and auditor review.
