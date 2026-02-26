# Load Test Report (Phase 6)

## Objective

- **Target:** 1000 RPS against Proxy `POST /v1/chat/completions`.
- **Latency target:** Total request p95 &lt; 20ms overhead (proxy mask + reidentify). End-to-end p95 depends on upstream LLM; proxy contribution should stay under 20ms.
- **Error rate:** &lt; 1%.

## How to Run

```bash
# Install k6 (e.g. brew install k6, or see https://k6.io/docs/getting-started/installation/)
export OPENAI_API_KEY=sk-...   # or use mock; proxy may return 502 if upstream unreachable
k6 run security/load-test/load_test.js
```

Optional env:

- `PROXY_URL` — default `http://127.0.0.1:8080`
- `OPENAI_API_KEY` — Bearer token for proxy

## Results Template

| Metric        | Target        | Observed (fill after run) |
|---------------|---------------|----------------------------|
| RPS           | 1000          | —                          |
| p50 (ms)      | —             | —                          |
| p95 (ms)      | &lt;20 overhead | —                        |
| p99 (ms)      | —             | —                          |
| Error rate    | &lt;1%         | —                          |

**Note:** Full request duration includes upstream LLM. To measure proxy-only overhead, use the Rust benchmark: `cargo bench --bench latency` (target &lt;20ms for mask+reidentify).

## k6 Thresholds

- `http_req_duration`: p95 &lt; 5000 ms (relaxed for e2e including upstream).
- `errors`: rate &lt; 0.01.

For proxy-only latency validation, rely on `cargo bench --bench latency` and the Phase 2/3 deliverable numbers.

---

**Phase 6 load test script and report complete.**
