# Phase 4 Deliverable: Developer Experience & SDK

## File structure

```
unblockllm/
├── python/
│   ├── pyproject.toml           # Publishable package (openai, typer)
│   ├── unblockllm/
│   │   ├── __init__.py          # UnblockClient export
│   │   ├── client.py             # UnblockClient (base_url → proxy)
│   │   ├── pii.py               # Regex PII detection (EMAIL, PHONE, SSN)
│   │   └── cli/
│   │       └── scan.py          # unblockllm-scan (Typer, JSON/HTML report)
│   └── tests/
│       ├── test_client.py       # SDK unit tests (mock)
│       └── test_cli_scan.py     # CLI integration tests
├── apps/
│   └── sdk-node/
│       ├── package.json         # @unblockllm/sdk, ESM + CJS
│       ├── tsconfig*.json
│       └── src/
│           └── index.ts         # UnblockClient
└── docs/
    ├── QUICKSTART.md            # Install SDK, run proxy, send request
    ├── SECURITY_MODEL.md        # Zero-trust, audit schema
    ├── CLI_USAGE.md             # unblockllm-scan usage
    └── PHASE4_DELIVERABLE.md    # This file
```

## Verification

### Python SDK and CLI

```bash
cd python && pip install -e ".[dev]"
unblockllm-scan --help
pytest -v
```

### Node SDK

```bash
cd apps/sdk-node && npm install && npm run build
```

### CLI test command (synthetic log)

```bash
echo 'Contact support@example.com or 555-123-4567.' > /tmp/sample.txt
unblockllm-scan --file /tmp/sample.txt --output report.html --format html
# Expect: Scan Result: 2 PII leak(s) found. Report written to report.html
```

## Constraints satisfied

- **Privacy:** CLI processes files locally; no data uploaded.
- **Security:** SDK does not log PII; default proxy URL is https.
- **Code quality:** Type hints (Python/TS); no hardcoded secrets.
- **Compatibility:** Phase 3 env vars (REDIS_URL, DATABASE_URL) are proxy-side; SDK uses UNBLOCKLLM_BASE_URL, OPENAI_API_KEY).
- **Viral hook:** CLI output includes shareable line: "Scan Result: N PII leak(s) found."

---

**Phase 4 complete.** Ready for Phase 5 (Control Plane).
