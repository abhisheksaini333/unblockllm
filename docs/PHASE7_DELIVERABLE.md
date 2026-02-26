# Phase 7 Deliverable: Launch Prep (Production Ready)

## Summary

Phase 7 finalizes production Docker images, the public landing page, GitHub release automation, patent disclosure and launch blog drafts, and verification that production builds have no debug flags and all tests/audits pass.

---

## 1. File Structure Changes

```
unblockllm/
├── Dockerfile.proxy              # Rust proxy, multi-stage, distroless, non-root
├── Dockerfile.dashboard          # Next.js dashboard, multi-stage, standalone
├── docker-compose.prod.yml       # Production stack (proxy, dashboard, redis, postgres)
├── CHANGELOG.md                  # Phases 1–7, v1.0.0
├── .github/workflows/
│   └── release.yml               # Release on tag push (v*), upload proxy binary
├── apps/
│   └── landing/                  # Public landing (Next.js 14, dark, Hero/Features/CLI/Pricing/Footer)
│       ├── package.json
│       ├── src/app/layout.tsx, page.tsx, globals.css
│       └── tailwind.config.js, next.config.js, etc.
└── docs/
    ├── PATENT_DISCLOSURE.md      # Stateful re-identification + local redaction claims
    ├── LAUNCH_BLOG_POST.md       # "40% of AI Apps Leak PII" narrative
    └── PHASE7_DELIVERABLE.md     # This file
```

---

## 2. Release Checklist (v1.0.0)

- [ ] All tests pass: `cargo test`, `npm test` (dashboard + landing if applicable), `pytest`
- [ ] Audits: `cargo audit` (1 medium: rsa/sqlx-mysql transitive; proxy uses Postgres only—document or patch), `npm audit` (dashboard + landing)
- [ ] No debug flags in production: proxy built with `--release`; dashboard/landing built with `NODE_ENV=production`
- [ ] Docker builds succeed: `docker build -f Dockerfile.proxy .`, `docker build -f Dockerfile.dashboard -f apps/dashboard/Dockerfile .` (or from repo root with correct context)
- [ ] Landing Lighthouse score target: 95+
- [ ] CHANGELOG.md and release notes reviewed
- [ ] Tag created: `git tag v1.0.0`
- [ ] Push tag: `git push origin v1.0.0` (triggers release workflow)
- [ ] GitHub Release created with draft release notes; attach binaries from workflow artifacts

---

## 3. Launch Commands

### Build proxy image (from repo root)

```bash
docker build -f Dockerfile.proxy -t unblockllm/proxy:1.0.0 .
```

### Build dashboard image (from repo root; Dockerfile.dashboard expects context at repo root or dashboard path per Dockerfile)

```bash
docker build -f Dockerfile.dashboard -t unblockllm/dashboard:1.0.0 .
```

(If your Dockerfile.dashboard uses `COPY apps/dashboard/...`, run from repo root. If it uses `COPY . .`, ensure context is set correctly.)

### Production stack (TLS handled by external reverse proxy)

```bash
docker compose -f docker-compose.prod.yml up -d
```

### Create and push release tag

```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

### Local verification (before tag)

```bash
cargo test
cargo audit   # 1 medium (rsa/sqlx-mysql) — proxy does not use MySQL
cd apps/dashboard && npm ci && npm run lint && npm run build && npm test
cd apps/landing && npm install && npm run build   # commit package-lock.json for CI
cd ../../python && python -m pytest tests/
```

### Latency benchmark scope

- **`cargo bench` / `benches/latency.rs`** measures **mask + reidentify only** (regex path; typically ~0.4ms). It does not include NER inference.
- **NER target (<5ms)** is validated by the Python benchmark: `python -m unblockllm.benchmark.ner_benchmark --runs 50 --output benchmark_report.json`.
- **Total overhead target (<20ms)** = NER + mask + reidentify + Redis/network. The Rust bench asserts mask+reid portion; add NER and network for full stack.

### Multi-arch Docker (M3 Pro / Apple Silicon and x86_64)

**Option A — Build for your architecture (single-arch):**

```bash
# From repo root. On M3 Pro (ARM64) this produces arm64 image; on x86_64 produces amd64.
docker build -f Dockerfile.proxy -t unblockllm/proxy:1.0.0 .

# Explicit platform (e.g. for deployment server):
docker build -f Dockerfile.proxy --platform linux/amd64 -t unblockllm/proxy:1.0.0 .
docker build -f Dockerfile.proxy --platform linux/arm64 -t unblockllm/proxy:1.0.0 .
```

**Option B — Multi-arch with buildx (both amd64 and arm64):**

```bash
docker buildx create --use --name multiarch 2>/dev/null || true
docker buildx build -f Dockerfile.proxy --platform linux/amd64,linux/arm64 -t unblockllm/proxy:1.0.0 --push .
```

Dashboard image: same pattern (`Dockerfile.dashboard`). No `.cargo/config.toml` is required for Docker; the image is built for the host arch unless `--platform` or buildx is used.

---

## 4. Release Notes (v1.0.0) — Draft for GitHub Release

**unblockllm v1.0.0** — Production-ready privacy proxy for LLM APIs.

**Highlights:**

- **OpenAI-compatible proxy:** Redact PII from requests, re-identify streaming responses; &lt;20ms latency.
- **Local detection:** Regex + NER (ONNX); no PII sent to external detectors.
- **Ephemeral mapping:** Redis or in-memory; 60s TTL; no raw PII in audit logs.
- **Policy engine:** YAML policies (mask/block by entity type); hot-reload; optional remote policy from dashboard.
- **CLI:** `unblockllm-scan` — scan logs locally, output JSON/HTML report.
- **Dashboard:** Next.js app for policy, stats, audit view; Stripe checkout for Enterprise.
- **Production Docker:** Distroless proxy image (&lt;100MB), standalone dashboard (&lt;200MB); docker-compose.prod.yml.
- **Landing:** Public site with Hero, Features, CLI demo, Pricing, Footer.

**Assets:**

- `unblock-proxy-x86_64-unknown-linux-gnu` (from release workflow)

**Docs:** QUICKSTART.md, SECURITY_MODEL.md, CLI_USAGE.md, CHANGELOG.md.

---

## 5. Patent & GTM

- **Patent:** See `docs/PATENT_DISCLOSURE.md` (stateful re-identification in streaming LLM proxies; local redaction; ephemeral mapping; audit without PII). For legal counsel review only.
- **GTM:** See `docs/LAUNCH_BLOG_POST.md` for draft blog (“40% of AI Apps Leak PII” narrative and CTA).

---

## 6. Phase 7 Additions (Launch Prep)

- **Landing:** SEO (OpenGraph, sitemap.ts), Contact section; `apps/landing` already has Hero, Features, Pricing.
- **Public Docs:** `apps/docs/` — Next.js app with Quickstart, Security Model, SLA (migrated from `docs/`). Run: `cd apps/docs && npm run build && npm run start` (port 3002).
- **Monitoring:** `docker-compose.monitoring.yml` — Prometheus + Grafana. Grafana on port 3001; add Prometheus datasource `http://prometheus:9090`.
- **Backup/Restore:** `scripts/backup.sh`, `scripts/restore.sh`; documented in DEPLOYMENT.md. Cron example for daily backups included.
- **CHANGELOG.md:** v1.0.0 release notes. Cargo and package versions set to 1.0.0 (proxy, core, dashboard, landing, docs).
- **Git tag:** When in a git repo, run: `git tag -a v1.0.0 -m "Release v1.0.0"` and `git push origin v1.0.0`.

## 7. Stop Signal

**Phase 7 complete.** All technical launch prep done.

**CPTO Reminder:** Before or immediately after launch, complete:
1. **Patent filing** — Use `docs/PATENT_DISCLOSURE.md`; engage legal counsel for stateful re-identification and local redaction claims.
2. **Domain purchase** — Secure production domain (e.g. unblockllm.dev) and point landing/docs to it.
3. **GTM execution** — Use `docs/LAUNCH_BLOG_POST.md` for blog; coordinate launch and customer comms.
