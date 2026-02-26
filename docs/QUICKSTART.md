# unblockllm Quickstart

Get the proxy running and send your first request through the SDK in a few minutes.

## 1. Install the SDK

**Python:**

```bash
cd python && pip install -e ".[dev]"
```

**Node.js:**

```bash
cd apps/sdk-node && npm install
```

## 2. Run the proxy

From the repo root, start Redis and Postgres (optional for basic use), then the proxy:

```bash
# Optional: state and audit (Phase 3)
docker compose up -d
export REDIS_URL="redis://127.0.0.1:6379/"
export DATABASE_URL="postgres://unblock:unblock_dev@127.0.0.1:5432/unblock_audit"

# Required: OpenAI API key and proxy
export OPENAI_API_KEY="sk-..."
cargo run --release
```

The proxy listens on `http://127.0.0.1:8080` by default.

## 3. Send a request

**Python:**

```python
from unblockllm import UnblockClient

client = UnblockClient()  # uses UNBLOCKLLM_BASE_URL or https://127.0.0.1:8080
resp = client.chat.completions.create(
    model="gpt-4o-mini",
    messages=[{"role": "user", "content": "Say hello."}],
)
print(resp.choices[0].message.content)
```

**Node.js:**

```javascript
const { UnblockClient } = require("@unblockllm/sdk"); // or import for ESM

const client = new UnblockClient();
const completion = await client.chat.completions.create({
  model: "gpt-4o-mini",
  messages: [{ role: "user", content: "Say hello." }],
});
console.log(completion.choices[0].message.content);
```

## 4. Environment variables

| Variable | Description |
|----------|-------------|
| `UNBLOCKLLM_BASE_URL` | Proxy URL (default: `https://127.0.0.1:8080`) |
| `OPENAI_API_KEY` or `UNBLOCKLLM_API_KEY` | API key for the upstream provider |
| `REDIS_URL` | Optional; Redis for mapping store (Phase 3) |
| `DATABASE_URL` | Optional; Postgres for audit logs (Phase 3) |

Use **https** for the proxy URL in production. No PII is logged by the SDK or proxy.

## Next steps

- [Security model](SECURITY_MODEL.md) — zero-trust and audit
- [CLI usage](CLI_USAGE.md) — scan logs for PII locally
