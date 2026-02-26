#!/usr/bin/env python3
"""
Pen test: Log extraction. Synthetic data only — no real PII.
Attempts to inject payloads that might force content into logs (e.g. error paths).
"""

import os
import sys
import json
import urllib.request
import urllib.error

PROXY_URL = os.environ.get("PROXY_URL", "http://127.0.0.1:8080")


def post_raw(body: bytes, headers: dict | None = None) -> tuple[int, str]:
    """POST raw body to /v1/chat/completions. Returns (status, response_text)."""
    url = f"{PROXY_URL.rstrip('/')}/v1/chat/completions"
    h = {"Content-Type": "application/json", **(headers or {})}
    req = urllib.request.Request(url, data=body, headers=h, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=5) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode() if e.fp else ""
    except Exception as e:
        return -1, str(e)


def main() -> None:
    results = []
    # Invalid JSON to trigger error path (must not log request body)
    status, resp = post_raw(b"not json at all")
    results.append({"test": "invalid_json", "status": status, "response_preview": resp[:200]})
    print(f"[invalid_json] status={status}")

    # Oversized payload (might hit error logging)
    big = json.dumps({"model": "gpt-4o-mini", "messages": [{"role": "user", "content": "x" * 100000}]})
    status, resp = post_raw(big.encode())
    results.append({"test": "oversized_payload", "status": status})
    print(f"[oversized_payload] status={status}")

    # Deeply nested to trigger parser limits
    nested = {"model": "gpt-4o-mini", "messages": [{"role": "user", "content": [["a"] * 100] * 100}]}
    status, resp = post_raw(json.dumps(nested).encode())
    results.append({"test": "nested_payload", "status": status})
    print(f"[nested_payload] status={status}")

    out = os.path.join(os.path.dirname(__file__), "log_extraction_results.json")
    with open(out, "w") as f:
        json.dump({"results": results}, f, indent=2)
    print(f"Results written to {out}")
    # Manual: verify server logs do not contain request body or PII
    print("Manual check: ensure proxy logs do not contain request body or PII.")


if __name__ == "__main__":
    main()
