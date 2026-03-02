# Deployment Guide

Production deployment for unblockllm proxy and dashboard. Follow SKILL.md and SECURITY_MODEL.md for constraints.

## TLS and HTTPS

- **Proxy:** The proxy **must** sit behind a TLS-terminating load balancer (e.g. AWS ALB, Nginx, Cloud Load Balancer). Do **not** terminate TLS inside the Rust proxy; use the LB for TLS and optional client cert validation.
- **Dashboard:** Next.js middleware enforces HTTPS in production when `x-forwarded-proto` is `http` (307 redirect). Run the dashboard behind the same TLS-terminating LB.
- Use `rediss://` for Redis and `sslmode=require` for Postgres when available.

## Secret Management (OPS-02)

- **Do not** store production secrets in `.env` files committed to the repo or on disk on production servers. Use a secret manager or orchestration-native secrets.
- **Supported:**
  - **Kubernetes:** Mount secrets as env vars (e.g. `envFrom` with `secretRef`) or as files. Set `PROXY_API_KEY`, `DATABASE_URL`, `REDIS_URL`, `OPENAI_BASE_URL` from Secrets. Example: `envFrom: [ { secretRef: { name: "unblockllm-secrets" } } ]`.
  - **AWS Secrets Manager:** Inject secrets at startup (e.g. sidecar or init container that writes env or files); or use a provider that sets env before process start.
- Rotate keys via Dashboard `POST /api/v1/keys/rotate` (session auth); then update the secret in K8s/AWS and restart the proxy so it picks up the new `PROXY_API_KEY`.

## Redis (PERF-02, OPS-01)

- **Connection pool:** The proxy uses a single `ConnectionManager` per process (see `crates/unblock-proxy/src/state.rs`). For high throughput, run load tests (e.g. `k6 run security/load-test/load_test.js`) and monitor Redis connection wait time; if wait time exceeds ~1ms, consider tuning Redis `maxclients` or running multiple proxy instances. No application-level pool size knob; one connection per process.
- **Fallback:** If Redis is unreachable and `REDIS_URL` is set, the proxy returns **503 (Block Mode)** and does not bypass redaction or forward traffic. See SECURITY_MODEL.md.

## Audit Log Retention

- Audit logs are retained for **90 days**. Run periodically (cron or scheduler):
  `DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL '90 days';`
- **Migrations:** The `audit_logs` table is created automatically by the proxy at startup. The file `migrations/003_audit_retention.sql` is optional: run **once per database** with `psql "$DATABASE_URL" -f migrations/003_audit_retention.sql` to add the retention comment. Migrations do **not** run automatically. See **docs/LOCAL_DEVELOPMENT.md** for when and how to run them.

## Backup and Restore (Phase 7)

- **Backup:** Run daily (cron) or on demand: `./scripts/backup.sh [BACKUP_DIR]`. Requires `DATABASE_URL`; optional `REDIS_URL`. Produces `audit_YYYYMMDD_HHMMSS.sql.gz` in `BACKUP_DIR` (default: `./backups`). Redis: script triggers BGSAVE; for RDB copy use Redis persistence dir.
- **Restore:** `./scripts/restore.sh ./backups/audit_YYYYMMDD_HHMMSS.sql.gz`. Requires `DATABASE_URL`. Destructive: applies SQL to current DB; verify backup before restore.
- **Cron example:** `0 2 * * * cd /path/to/unblockllm && DATABASE_URL=... ./scripts/backup.sh /var/backups/unblockllm`

## Health Probes

- **`GET /health`** — Liveness probe. Returns `{"status":"ok","version":"..."}`. Always 200 if the process is alive.
- **`GET /readyz`** — Readiness probe. Checks Redis/memory store, Postgres audit, and NER model availability. Returns 200 with `{"status":"ready","checks":{...}}` or 503 if not ready.
- **`GET /metrics`** — Prometheus metrics endpoint. Exposes `http_requests_total`, `http_request_duration_seconds`, `pii_entities_detected_total`, `pii_masking_duration_seconds`, `upstream_request_duration_seconds`.

## Anthropic Support

- **`POST /v1/messages`** — Anthropic Messages API proxy. Same PII masking/re-identification as OpenAI, supports both string and content-block message formats.
- Set `ANTHROPIC_BASE_URL` to override the upstream (default: `https://api.anthropic.com`).
- Pass `x-api-key` and `anthropic-version` headers as usual.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `OPENAI_BASE_URL` | `https://api.openai.com` | Upstream OpenAI API |
| `ANTHROPIC_BASE_URL` | `https://api.anthropic.com` | Upstream Anthropic API |
| `REDIS_URL` | (none) | Redis for state + rate limiting |
| `DATABASE_URL` | (none) | Postgres for audit logging |
| `NER_MODEL_DIR` | (none) | Path to ONNX NER model directory |
| `POLICY_CONFIG_PATH` | `config/policy.yaml` | PII policy config |
| `LOG_FORMAT` | `text` | Set to `json` for structured JSON logging |
| `SENTRY_DSN` | (none) | Sentry error tracking |

## Monitoring & Alerts

- **Self-hosted:** `docker compose -f docker-compose.monitoring.yml up -d` — Prometheus (port 9090) and Grafana (port 3001). Grafana auto-provisions the "UnblockLLM Proxy" dashboard with request rate, latency percentiles, error rate, PII entity detection, and upstream LLM latency panels. Prometheus is pre-configured to scrape the proxy at `proxy:8080/metrics`.
- **Sentry (optional):** Set `NEXT_PUBLIC_SENTRY_DSN` and/or `SENTRY_DSN` for the Dashboard (Next.js) and `SENTRY_DSN` for the Proxy (Rust). If unset, no events are sent. See [Sentry for Next.js](https://docs.sentry.io/platforms/javascript/guides/nextjs/) and [Sentry for Rust](https://docs.sentry.io/platforms/rust/).

## Multi-Arch and Images

- Build proxy (with ONNX model): `docker build -f Dockerfile.proxy -t unblockllm/proxy:tag .`
- Build proxy (regex-only, smaller): `docker build -f Dockerfile.proxy --build-arg INCLUDE_ONNX_MODEL=false -t unblockllm/proxy:tag-slim .`
- Build dashboard: `docker build -f Dockerfile.dashboard -t unblockllm/dashboard:tag .`
- For `linux/amd64` or `linux/arm64`: use `--platform` or `docker buildx` (see docs/PHASE7_DELIVERABLE.md).
- **CI/CD:** On tag push (`v*`), the release workflow builds the proxy binary, creates a GitHub Release, and pushes a Docker image to `ghcr.io/<repo>/proxy:<version>`.
