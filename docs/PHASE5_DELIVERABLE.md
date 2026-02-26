# Phase 5 Deliverable: Control Plane (Revenue Layer)

## File structure

```
unblockllm/
├── apps/dashboard/
│   ├── .env.example           # Stripe, DB, Auth, PROXY_API_KEY
│   ├── package.json           # next-auth, stripe, recharts, pg, tailwind
│   ├── src/
│   │   ├── app/
│   │   │   ├── api/
│   │   │   │   ├── auth/[...nextauth]/route.ts
│   │   │   │   ├── auth/register/route.ts
│   │   │   │   ├── stripe/checkout/route.ts
│   │   │   │   ├── webhooks/stripe/route.ts
│   │   │   │   └── v1/
│   │   │   │       ├── policy/route.ts        # Proxy auth: X-Proxy-API-Key
│   │   │   │       ├── policy/config/route.ts # Dashboard GET/PUT (session)
│   │   │   │       └── stats/route.ts         # Aggregate audit (no PII)
│   │   │   ├── dashboard/
│   │   │   │   ├── page.tsx       # Stats charts, links
│   │   │   │   ├── policy/page.tsx
│   │   │   │   └── billing/page.tsx
│   │   │   ├── login/page.tsx
│   │   │   ├── register/page.tsx
│   │   │   └── page.tsx          # Home → redirect or sign in
│   │   ├── components/Providers.tsx
│   │   └── lib/
│   │       ├── db.ts             # Pool, initDashboardSchema
│   │       └── auth.ts           # Credentials, verifyPassword
│   └── tailwind.config.js, postcss.config.js
├── crates/unblock-proxy/src/
│   ├── main.rs                  # DASHBOARD_URL → spawn_remote_poll
│   └── policy/mod.rs            # set_config, spawn_remote_poll
└── docs/PHASE5_DELIVERABLE.md   # This file
```

## Environment variables

| Variable | Where | Description |
|----------|--------|-------------|
| `DATABASE_URL` | Dashboard | Phase 3 Postgres (audit_logs + dashboard tables) |
| `NEXTAUTH_URL` | Dashboard | e.g. `http://localhost:3000` |
| `NEXTAUTH_SECRET` | Dashboard | Generate with `openssl rand -base64 32` |
| `STRIPE_SECRET_KEY` | Dashboard | Stripe secret key (test/live) |
| `STRIPE_PUBLISHABLE_KEY` | Dashboard | For client-side Stripe (optional) |
| `STRIPE_WEBHOOK_SECRET` | Dashboard | From Stripe CLI or Dashboard webhook |
| `STRIPE_PRICE_ID_PRO` | Dashboard | Price ID for Pro plan |
| `STRIPE_PRICE_ID_ENTERPRISE` | Dashboard | Price ID for Enterprise plan |
| `PROXY_API_KEY` | Dashboard + Proxy | Shared secret for `/api/v1/policy` |
| `DASHBOARD_URL` | Proxy | e.g. `http://localhost:3000` — enables remote policy poll |
| `PROXY_API_KEY` | Proxy | Same as dashboard (X-Proxy-API-Key when polling) |

## Stripe setup

1. Create products in Stripe Dashboard: **Free** (no price), **Pro**, **Enterprise**.
2. Create recurring prices for Pro and Enterprise; copy Price IDs to `STRIPE_PRICE_ID_PRO` and `STRIPE_PRICE_ID_ENTERPRISE`.
3. Add webhook endpoint: `https://your-domain.com/api/webhooks/stripe` (or use Stripe CLI for local).
4. Subscribe to: `checkout.session.completed`, `customer.subscription.updated`, `customer.subscription.deleted`.
5. Copy webhook signing secret to `STRIPE_WEBHOOK_SECRET`.

## Verification

### 1. Dashboard

```bash
cd apps/dashboard
cp .env.example .env   # fill in DATABASE_URL, NEXTAUTH_SECRET, etc.
npm install
npm run dev
```

- Open `http://localhost:3000` → Register → Sign in → Dashboard.
- Dashboard shows aggregate stats (requests, entities masked) and links to Policy and Billing.

### 2. Stripe Checkout (local)

- Set `STRIPE_SECRET_KEY`, `STRIPE_PRICE_ID_PRO`, `STRIPE_WEBHOOK_SECRET` in `.env`.
- Use Stripe CLI to forward webhooks:  
  `stripe listen --forward-to localhost:3000/api/webhooks/stripe`
- In Dashboard go to Billing → Subscribe to Pro → complete checkout (test card `4242 4242 4242 4242`).

### 3. Remote Policy API (curl)

```bash
# Replace PROXY_API_KEY and base URL with your values
curl -s -H "X-Proxy-API-Key: YOUR_PROXY_API_KEY" http://localhost:3000/api/v1/policy
```

Expected: `{"mask":["EMAIL","PHONE",...],"block":[]}`

### 4. Proxy with remote policy

```bash
export DASHBOARD_URL="http://localhost:3000"
export PROXY_API_KEY="same-key-as-dashboard"
export OPENAI_API_KEY="sk-..."
cargo run --release
```

Proxy will poll `/api/v1/policy` every 30s and apply mask/block from dashboard. No file reload when `DASHBOARD_URL` is set.

## Dashboard security

- **No PII:** Stats show only counts and entity type names (e.g. "500 emails masked", "EMAIL: 120"). No request content or user content.
- **Auth:** NextAuth JWT (Credentials). Session required for dashboard pages and for `/api/v1/stats` and `/api/v1/policy/config`.
- **Remote Policy:** Only proxy with correct `X-Proxy-API-Key` can read `/api/v1/policy`.

## Stripe CLI (local webhook testing)

```bash
stripe listen --forward-to localhost:3000/api/webhooks/stripe
# Use the printed webhook signing secret in .env as STRIPE_WEBHOOK_SECRET
```

---

**Phase 5 complete.** Ready for Phase 6 (Security Hardening).
