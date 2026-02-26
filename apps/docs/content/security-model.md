# unblockllm Security Model

Zero-trust privacy proxy: PII is redacted locally before any text reaches a public LLM. This document summarizes the model and audit schema.

## Zero-Trust Overview

1. **Assume breach:** Treat all traffic as sensitive. No raw PII is sent to the LLM provider; the proxy masks it first.
2. **Local redaction:** NER + regex run on the proxy (or in the CLI, locally). Masked text is forwarded; placeholders are used for re-identification.
3. **No PII in logs:** Logs and audit store only metadata: request IDs, entity counts, entity type names (e.g. `EMAIL`, `PHONE`). Never raw text or mapping values.
4. **Ephemeral mapping:** Placeholder → original mapping is stored in Redis with a short TTL (e.g. 60s) and never written to durable audit storage.
5. **TLS:** Use `https` for the proxy in production. Use `rediss://` for Redis and `sslmode=require` for Postgres when available.

## Audit Log Schema

Audit data is stored only when `DATABASE_URL` is set. Schema (Postgres):

```sql
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    request_id TEXT NOT NULL,
    user_id TEXT,
    entity_count INT NOT NULL,
    entity_types TEXT NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

- **request_id:** Opaque identifier for the request (no PII).
- **user_id:** Optional opaque user identifier.
- **entity_count:** Number of redacted entities in the request.
- **entity_types:** JSON array of type names only (e.g. `["EMAIL","PHONE","SSN"]`). No raw values.

No user content, no placeholder values, and no mapping data are ever written to this table.

## SDK and CLI

- **SDK:** Does not log request or response content. Use `https` for `UNBLOCKLLM_BASE_URL` in production.
- **CLI (`unblockllm-scan`):** Runs entirely on your machine. No data is uploaded; regex-based PII detection is local only.

## Policy Engine

The proxy can block or allow entity types via `config/policy.yaml`. Only type names are configured; no PII in config.

## Sub-Processors (Data Flow)

- **LLM providers:** When the proxy forwards requests upstream, it sends **masked** text only (placeholders like `[EMAIL_1]`). Sub-processors include OpenAI, Anthropic, and other OpenAI-compatible API hosts configured via `OPENAI_BASE_URL`.
- **Metadata:** Token counts, model names, and request/response structure may be visible to the LLM provider; no raw PII is sent.

## Redis Fallback (OPS-01)

When `REDIS_URL` is set and Redis is **unreachable**, the proxy operates in **Block Mode**: it returns **503 Service Unavailable** and does not bypass redaction. When `REDIS_URL` is unset, the proxy uses an in-memory store (single-instance only).

## Audit Log Retention (COM-01)

Audit logs are retained for **90 days**. Run a scheduled job:  
`DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL '90 days';`
