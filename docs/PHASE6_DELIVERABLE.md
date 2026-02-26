# Phase 6 Deliverable: Security Hardening & Compliance

## File structure

```
unblockllm/
├── security/
│   ├── README.md                 # Scans, SBOM, pen-test, load-test
│   ├── sbom.json                 # CycloneDX SBOM (Dashboard npm)
│   ├── pen-test/
│   │   ├── README.md
│   │   ├── masking_bypass.sh     # Synthetic PII bypass attempts
│   │   ├── log_injection.sh      # Log injection (synthetic)
│   │   ├── api_abuse.sh          # Policy/stats auth, rate limit
│   │   └── REPORT.md
│   └── load-test/
│       ├── load_test.js          # k6 script (1000 RPS target)
│       └── REPORT.md
├── docs/
│   ├── COMPLIANCE_PACKET.md      # SOC2 Type I mapping (CC1–CC9)
│   └── PHASE6_DELIVERABLE.md     # This file
└── crates/unblock-proxy/src/
    ├── middleware/
    │   ├── mod.rs
    │   └── rate_limit.rs         # tower_governor, RATE_LIMIT_PER_SECOND
    └── main.rs                   # Rate limit on chat, into_make_service_with_connect_info
apps/dashboard/
├── next.config.js                # HSTS, X-Frame-Options, X-Content-Type-Options
└── src/middleware.ts             # HTTPS redirect in production (x-forwarded-proto)
```

## Verification

### Pen-test (synthetic data only)

```bash
# Start proxy (and optionally dashboard)
export OPENAI_API_KEY=sk-test
cargo run --release &
# Run scripts
PROXY_URL=http://127.0.0.1:8080 ./security/pen-test/masking_bypass.sh
./security/pen-test/log_injection.sh
DASHBOARD_URL=http://127.0.0.1:3000 ./security/pen-test/api_abuse.sh
```

### Load test (k6)

```bash
k6 run security/load-test/load_test.js
# Env: PROXY_URL, OPENAI_API_KEY. Target 1000 RPS; proxy-only overhead target p95 <20ms (see cargo bench --bench latency).
```

### Dependency scans

```bash
cargo audit          # Rust: 1 medium (rsa via sqlx-mysql transitive; we use postgres only)
cd apps/dashboard && npm audit   # Node: fix Critical/High where possible
```

### SBOM

- Dashboard: `cd apps/dashboard && npm sbom --sbom-format=cyclonedx > ../../security/sbom.json`
- Rust: optional `cargo install cargo-cyclonedx` then `cargo cyclonedx` from crate dirs.

## Hardening summary

| Control | Implementation |
|--------|----------------|
| Rate limiting | Proxy: tower_governor on /v1/chat/completions, per peer IP; RATE_LIMIT_PER_SECOND (default 1000), burst 2x. Returns 429 when exceeded. |
| HTTPS | Dashboard: middleware redirects HTTP→HTTPS in production when x-forwarded-proto=http; next.config headers: HSTS (production), X-Frame-Options, X-Content-Type-Options. |
| CORS | Dashboard: same-origin by default; no permissive Access-Control-Allow-Origin. |

## Compliance

- **docs/COMPLIANCE_PACKET.md:** SOC2 Type I control mapping (CC1–CC9) with evidence (audit schema, encryption, access control, CI, pen-test, SBOM).

---

**Phase 6 complete.** Ready for Phase 7 (Launch Prep).
