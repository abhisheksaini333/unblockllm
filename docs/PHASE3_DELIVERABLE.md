# Phase 3 Deliverable: State & Compliance

## File structure

```
unblockllm/
├── docker-compose.yml          # Redis + Postgres (persistent volumes)
├── config/
│   └── policy.yaml             # mask / block entity types
├── crates/unblock-proxy/src/
│   ├── state.rs                # Redis-backed store (fallback: in-memory)
│   ├── audit.rs                # Postgres audit log (metadata only)
│   ├── policy/
│   │   └── mod.rs              # YAML policy engine + hot-reload
│   ├── handlers/chat.rs        # Uses policy filter, calls audit.log
│   └── ...
└── docs/
    └── PHASE3_DELIVERABLE.md   # This file
```

## Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| `REDIS_URL` | No (fallback: in-memory) | Redis connection URL, e.g. `redis://127.0.0.1:6379/` |
| `DATABASE_URL` | No (audit no-op) | Postgres connection URL, e.g. `postgres://unblock:unblock_dev@127.0.0.1:5432/unblock_audit` |
| `OPENAI_BASE_URL` | No | Default `https://api.openai.com` |
| `POLICY_CONFIG_PATH` | No | Default `config/policy.yaml` |

**Encryption / TLS (production):**

- Use `rediss://` for Redis TLS (e.g. `rediss://...` with cert verification).
- Use `postgres://` with `sslmode=require` (or `postgresql://...?sslmode=require`) for Postgres TLS.
- Never commit secrets; use env vars or a secret manager.

## Docker Compose

**Start Redis + Postgres (from repo root):**

```bash
docker compose up -d
```

**Stop:**

```bash
docker compose down
```

**With persistent data (default):** Volumes `redis_data` and `postgres_data` keep data across restarts.

**Connection URLs for local dev:**

```bash
export REDIS_URL="redis://127.0.0.1:6379/"
export DATABASE_URL="postgres://unblock:unblock_dev@127.0.0.1:5432/unblock_audit"
```

## Postgres schema (audit_logs)

Created automatically on first run when `DATABASE_URL` is set.

```sql
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    request_id TEXT NOT NULL,
    user_id TEXT,
    entity_count INT NOT NULL,
    entity_types TEXT NOT NULL DEFAULT '[]',   -- JSON array of type names, e.g. ["EMAIL","PHONE"]
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**No PII:** Only `request_id`, optional `user_id` (opaque), `entity_count`, and `entity_types` (e.g. `["EMAIL","PHONE"]`) are stored. Never log raw text or mapping values.

## Policy engine (config/policy.yaml)

- **mask:** Entity types to redact with placeholders. Empty = all detected types.
- **block:** Entity types that cause the request to be rejected (400). Example: `block: [SSN]`.

Policy is reloaded every 5 seconds (configurable in code). Change the file and save; no restart needed.

## Redis mapping store

- Key pattern: `unblock:map:{request_id}`.
- Value: Redis hash (placeholder → original). **Never log or persist raw PII elsewhere.**
- TTL: 60 seconds per key (ephemeral).

## Verification

### 1. Docker + proxy

```bash
docker compose up -d
export REDIS_URL="redis://127.0.0.1:6379/"
export DATABASE_URL="postgres://unblock:unblock_dev@127.0.0.1:5432/unblock_audit"
export OPENAI_API_KEY="sk-..."
cargo run --release
```

### 2. Trigger a request (creates audit row)

```bash
curl -s http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"My email is test@example.com."}],"stream":false}'
```

### 3. Verify audit log entry (no PII)

```bash
docker compose exec postgres psql -U unblock -d unblock_audit -c \
  "SELECT request_id, entity_count, entity_types, created_at FROM audit_logs ORDER BY created_at DESC LIMIT 5;"
```

Expected: One row per request with `request_id`, `entity_count` (e.g. 1), `entity_types` (e.g. `["EMAIL"]`), and `created_at`. No user content or mapping values.

### 4. Tests and benchmark

```bash
cargo test
cargo bench --bench latency
```

## Security (Phase 3)

- **Audit:** Only metadata; no PII or mapping values in `audit_logs`.
- **Redis:** Mapping keys have 60s TTL; use TLS (`rediss://`) in production.
- **Postgres:** Credentials via `DATABASE_URL`; use TLS and restricted roles in production.
- **Policy:** File-based; ensure `config/policy.yaml` is not writable by untrusted users.

---

**Phase 3 complete.** Proceed to Phase 4 (DX & SDK).
