# Local Development Guide

This guide answers: **why we have migrations**, **how to run the stack with HMR** for landing and dashboard, and **how to test the entire unblockllm solution** locally.

---

## 1. Migrations folder (`migrations/`)

### Why is it needed?

- **Schema creation is automatic:** The **proxy** creates the `audit_logs` table on startup when `DATABASE_URL` is set (see `crates/unblock-proxy/src/audit.rs`). The **dashboard** creates its tables (`users`, `subscriptions`, `policy_config`, `key_rotation_events`) on first use via `ensureSchema()` in `apps/dashboard/src/lib/db.ts`. You do **not** need the migrations folder for the app to run.
- **What the migrations folder is for:** It holds **optional, one-off SQL migrations** for **compliance and operations**, not for creating tables. Currently it contains:
  - **`003_audit_retention.sql`** — Documents the 90-day audit retention policy (adds a `COMMENT` on `audit_logs`) and shows the exact `DELETE` to run for retention. Required for COM-01 / SECURITY_MODEL / COMPLIANCE_PACKET.

### Why keep it?

- **Audit and compliance:** Auditors and SECURITY_MODEL/SLA docs reference it. It is the single place that defines the retention policy and the exact cleanup SQL.
- **Operational clarity:** New environments (staging/production) can run it once so the retention policy is documented in the DB and the same cleanup command is used everywhere.

### When do migrations run?

- **They do not run automatically.** There is no migration runner that executes `migrations/*.sql` on deploy or startup.
- **When to run `003_audit_retention.sql`:**
  1. **Once per database**, after the proxy has already created `audit_logs` (i.e. after at least one proxy startup with `DATABASE_URL` set).
  2. From repo root:  
     `psql "$DATABASE_URL" -f migrations/003_audit_retention.sql`
- **Ongoing retention:** The actual cleanup is a **scheduled job** (cron or app job), not this file. Run periodically:  
  `DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL '90 days';`  
  (See DEPLOYMENT.md and SECURITY_MODEL.md.)

---

## 2. Run sequence and HMR (landing + dashboard)

To see the **landing page** and **dashboard** locally and **edit with Hot Module Replacement (HMR)**:

### Step 1: Start infrastructure (Redis + Postgres)

From the **repo root**:

```bash
cd /Users/abhishek/Documents/datavedam/unblockllm
docker compose up -d
```

Wait until Postgres and Redis are healthy (`docker compose ps`).

### Step 2: Environment variables

Create or use a `.env` in the repo root (and/or in `apps/dashboard` for NextAuth/Stripe) so the proxy and dashboard can talk to the same DB.

**Repo root `.env`** (for proxy and for dashboard if you run it from root or reference this):

```bash
# Optional for proxy
REDIS_URL=redis://127.0.0.1:6379/
DATABASE_URL=postgres://unblock:unblock_dev@127.0.0.1:5432/unblock_audit

# For proxy
OPENAI_API_KEY=sk-your-key
# Optional: if dashboard runs and you want proxy to fetch policy from it
DASHBOARD_URL=http://127.0.0.1:3000
PROXY_API_KEY=your-secure-proxy-key
```

**`apps/dashboard/.env`** (copy from `apps/dashboard/.env.example`):

- `DATABASE_URL` — same as above.
- `NEXTAUTH_URL=http://localhost:3000`
- `NEXTAUTH_SECRET` — e.g. `openssl rand -base64 32`
- `PROXY_API_KEY` — same value as above so dashboard and proxy agree.
- Stripe vars optional for local UI (use test keys if you test billing).

### Step 3: Run with HMR (recommended for dev)

Run **landing** and **dashboard** with their **dev servers** so you get HMR. Do **not** run them via Docker for active UI development.

**Terminal 1 — Dashboard (port 3000, HMR):**

```bash
cd apps/dashboard
npm install
npm run dev
```

**Terminal 2 — Landing (port 3001, HMR):**

```bash
cd apps/landing
npm install
npm run dev
```

**Terminal 3 — Proxy (optional, for full E2E):**

```bash
# From repo root, with .env that has REDIS_URL, DATABASE_URL, OPENAI_API_KEY, etc.
cargo run --release -p unblock-proxy
# Or: cargo run -p unblock-proxy   (debug build, faster compile)
```

**Optional — Public docs (port 3002):**

```bash
cd apps/docs
npm install
npm run dev
```

### Step 4: Open in browser

- **Landing:** http://localhost:3001  
- **Dashboard:** http://localhost:3000 (register/login, then use dashboard)  
- **Docs:** http://localhost:3002  
- **Proxy API:** http://localhost:8080 (when proxy is running)

Changing code in `apps/landing` or `apps/dashboard` will hot-reload in the browser.

### Summary: run order

1. `docker compose up -d` (Redis + Postgres)
2. Set `.env` (root and/or `apps/dashboard`)
3. `npm run dev` in `apps/dashboard` and `apps/landing` (and optionally `apps/docs`)
4. Optionally `cargo run -p unblock-proxy` for API/audit testing

---

## 3. Step-by-step: test the entire solution locally

Follow this sequence to run and test the full stack (infra + proxy + dashboard + landing, with HMR for apps).

### Prerequisites

- Docker (for Redis + Postgres)
- Rust (for proxy)
- Node 18+ (for dashboard, landing, docs)

### 1. Clone and go to repo root

```bash
cd /Users/abhishek/Documents/datavedam/unblockllm
```

### 2. Start Redis and Postgres

```bash
docker compose up -d
docker compose ps   # wait until both healthy
```

### 3. (Optional) Run the retention migration once

Only needed if you want the audit table comment and to align with compliance docs. Do this **after** the proxy has created `audit_logs` (e.g. after step 6 once).

```bash
export DATABASE_URL="postgres://unblock:unblock_dev@127.0.0.1:5432/unblock_audit"
psql "$DATABASE_URL" -f migrations/003_audit_retention.sql
```

### 4. Configure environment

- Copy `apps/dashboard/.env.example` to `apps/dashboard/.env` and set at least:
  - `DATABASE_URL=postgres://unblock:unblock_dev@127.0.0.1:5432/unblock_audit`
  - `NEXTAUTH_URL=http://localhost:3000`
  - `NEXTAUTH_SECRET=$(openssl rand -base64 32)`
  - `PROXY_API_KEY=dev-proxy-key-change-in-prod`
- In repo root, create `.env` (or export) with:
  - `DATABASE_URL`, `REDIS_URL` (same as above)
  - `OPENAI_API_KEY=sk-...`
  - `PROXY_API_KEY=dev-proxy-key-change-in-prod` (same as dashboard)
  - Optionally `DASHBOARD_URL=http://127.0.0.1:3000` for proxy policy fetch

### 5. Install app dependencies

```bash
cd apps/dashboard && npm install && cd ../..
cd apps/landing  && npm install && cd ../..
# Optional:
# cd apps/docs   && npm install && cd ../..
```

### 6. Start the proxy (creates `audit_logs` on first run)

From repo root:

```bash
cargo run -p unblock-proxy
```

Leave it running. On first request with `DATABASE_URL` set, it creates the `audit_logs` table. Stop with Ctrl+C when you’re done testing the proxy.

### 7. Start dashboard and landing (separate terminals, HMR)

**Terminal A — Dashboard:**

```bash
cd apps/dashboard
npm run dev
```

**Terminal B — Landing:**

```bash
cd apps/landing
npm run dev
```

### 8. Test the flows

- **Landing:** Open http://localhost:3001 — check copy, links, and “Get started” (e.g. to dashboard or signup).
- **Dashboard:** Open http://localhost:3000 — register a user, log in, open dashboard, policy, billing (Stripe in test mode if configured).
- **Proxy:** With proxy running on 8080, send a request (see QUICKSTART.md), e.g.:

  ```bash
  curl -X POST http://127.0.0.1:8080/v1/chat/completions \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $OPENAI_API_KEY" \
    -d '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"Say hi"}]}'
  ```

- **Audit:** After using the proxy with `DATABASE_URL` set, check that rows appear:

  ```bash
  psql "$DATABASE_URL" -c "SELECT id, request_id, entity_count, created_at FROM audit_logs ORDER BY created_at DESC LIMIT 5;"
  ```

### 9. Optional: production-like stack with Docker

To run proxy + dashboard as built images (no HMR):

```bash
# Set DATABASE_URL, NEXTAUTH_URL, NEXTAUTH_SECRET, PROXY_API_KEY, OPENAI_API_KEY, etc.
docker compose -f docker-compose.prod.yml up -d
```

- Dashboard: http://localhost:3000  
- Proxy: http://localhost:8080  

Use this to verify production build; for day-to-day UI work, prefer the HMR flow in section 2.

### 10. Optional: monitoring stack

```bash
docker compose -f docker-compose.monitoring.yml up -d
```

- Prometheus: http://localhost:9090  
- Grafana: http://localhost:3001 (default admin/admin); add Prometheus datasource `http://prometheus:9090`

---

## Quick reference

| Goal                         | Command / URL |
|-----------------------------|----------------|
| Infra only                  | `docker compose up -d` |
| Dashboard (HMR)              | `cd apps/dashboard && npm run dev` → http://localhost:3000 |
| Landing (HMR)                | `cd apps/landing && npm run dev` → http://localhost:3001 |
| Docs (HMR)                  | `cd apps/docs && npm run dev` → http://localhost:3002 |
| Proxy                       | `cargo run -p unblock-proxy` → http://localhost:8080 |
| Run retention migration     | `psql "$DATABASE_URL" -f migrations/003_audit_retention.sql` (once per DB) |
| Full stack (no HMR)         | `docker compose -f docker-compose.prod.yml up -d` |
| Monitoring                  | `docker compose -f docker-compose.monitoring.yml up -d` |
