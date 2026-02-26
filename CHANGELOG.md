# Changelog

All notable changes to unblockllm are documented here.

## [1.0.0] — 2025-02-23

### Added

- **Proxy (Rust):** OpenAI-compatible HTTP proxy with local PII redaction (regex + optional ONNX NER), streaming re-identification, Redis or in-memory mapping store, Postgres audit (metadata only), policy engine (YAML), rate limiting (in-memory or Redis-backed).
- **Dashboard (Next.js):** Policy config, stats, Stripe billing, key rotation (`POST /api/v1/keys/rotate`), NextAuth.
- **Landing (Next.js):** Public marketing site with Hero, Features, CLI demo, Pricing, Contact; SEO (OpenGraph, sitemap).
- **Public Docs (Next.js):** Quickstart, Security Model, SLA at `apps/docs`.
- **SDKs:** Python (`unblockllm`) and Node (`@unblockllm/sdk`) clients.
- **CLI:** `unblockllm-scan` for local log scanning (regex-based).
- **Docker:** `Dockerfile.proxy` (distroless, non-root), `Dockerfile.dashboard`, `docker-compose.prod.yml`, `docker-compose.monitoring.yml` (Prometheus + Grafana).
- **Backup/restore:** `scripts/backup.sh` (pg_dump + Redis), `scripts/restore.sh`; 90-day audit retention (COM-01).
- **Compliance:** SOC2-ready packet, SECURITY_MODEL, INCIDENT_RESPONSE, SLA (Regex <5ms, ONNX <50ms), GDPR Right to Erasure script (`scripts/anonymize_user.py`).
- **Security:** No PII in logs; no `unwrap`/`expect` in production proxy path; TLS via LB; key rotation; Sentry-ready (optional).

### Fixed

- Rate limit layer returns `Result` (no `.expect()` in production).
- Docker proxy build: `protoc` installed in builder; dashboard: `mkdir -p public` for standalone.

### Documentation

- QUICKSTART, SECURITY_MODEL, SLA, DEPLOYMENT, COMPLIANCE_PACKET, SECURITY (known advisories), PATENT_DISCLOSURE, LAUNCH_BLOG_POST, PHASE*_DELIVERABLE.

---

[1.0.0]: https://github.com/unblockllm/unblockllm/releases/tag/v1.0.0
