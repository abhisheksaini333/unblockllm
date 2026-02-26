# Security

## API Key Rotation (SEC-02)

1. **Rotate:** Authenticate to the dashboard (session), then `POST /api/v1/keys/rotate`. The response contains a new key (one-time); store it in your secret manager (K8s Secret, AWS Secrets Manager).
2. **Deploy:** Update `PROXY_API_KEY` in the proxy’s environment with the new value and restart the proxy. The old key is invalid after you switch.
3. **Status:** `GET /api/v1/keys/status` (session or `X-Proxy-API-Key`) returns the last `rotated_at` timestamp.

Do not commit raw keys to the repo or store them in `.env` in production. See `docs/DEPLOYMENT.md`.

## Known Advisories

### Rust Dependency: `rsa` (Medium)
- **Advisory:** RUSTSEC-2023-0071 (Marvin timing side-channel)
- **Path:** `sqlx` → `sqlx-mysql` (transitive) → `rsa`
- **Mitigation:**
  1. We use `sqlx-postgres`, not `sqlx-mysql`. The MySQL driver (and thus the `rsa` dependency) is not instantiated in production.
  2. The dependency is used for database handshake cryptography, not application-level PII encryption.
  3. PII is never stored; only metadata (counts/types) is persisted.
- **Status:** Accepted Risk. No upgrade path available without breaking `sqlx` compatibility. SEC-01: Proxy uses only `postgres` + `runtime-tokio-rustls` (see `crates/unblock-proxy/Cargo.toml`); the `sqlx` meta-crate may still pull `sqlx-mysql` in the lockfile. To eliminate `rsa` entirely, a future change could depend on `sqlx-postgres` and `sqlx-core` directly.
