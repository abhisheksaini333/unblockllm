#!/usr/bin/env python3
"""
Pen test: API abuse. Rate limiting and auth bypass.
- Proxy: /v1/chat/completions without key, with bad key.
- Dashboard: /api/v1/policy without key; /api/v1/stats without session.
Synthetic only; no real credentials.
"""

import os
import sys
import json
import urllib.request
import urllib.error

PROXY_URL = os.environ.get("PROXY_URL", "http://127.0.0.1:8080")
DASHBOARD_URL = os.environ.get("DASHBOARD_URL", "http://127.0.0.1:3000")


def get(url: str, headers: dict | None = None) -> tuple[int, str]:
    req = urllib.request.Request(url, headers=headers or {}, method="GET")
    try:
        with urllib.request.urlopen(req, timeout=5) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode() if e.fp else ""


def post_json(url: str, body: dict, headers: dict | None = None) -> tuple[int, str]:
    h = {"Content-Type": "application/json", **(headers or {})}
    req = urllib.request.Request(url, data=json.dumps(body).encode(), headers=h, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=5) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode() if e.fp else ""


def main() -> None:
    results = []

    # Proxy: request without Authorization
    status, _ = post_json(
        f"{PROXY_URL.rstrip('/')}/v1/chat/completions",
        {"model": "gpt-4o-mini", "messages": [{"role": "user", "content": "Hi"}], "stream": False},
        headers={},
    )
    results.append({"target": "proxy_no_auth", "status": status, "expected": "401 or 200 (proxy may forward)"})
    print(f"[proxy_no_auth] status={status}")

    # Dashboard: policy API without X-Proxy-API-Key (must 401)
    status, _ = get(f"{DASHBOARD_URL.rstrip('/')}/api/v1/policy")
    results.append({"target": "dashboard_policy_no_key", "status": status, "expected": 401})
    print(f"[dashboard_policy_no_key] status={status} (expected 401)")

    # Dashboard: stats without session (must 401)
    status, _ = get(f"{DASHBOARD_URL.rstrip('/')}/api/v1/stats")
    results.append({"target": "dashboard_stats_no_session", "status": status, "expected": 401})
    print(f"[dashboard_stats_no_session] status={status} (expected 401)")

    # Optional: burst of requests to proxy to observe rate limiting (run after rate limit is on)
    for i in range(5):
        post_json(
            f"{PROXY_URL.rstrip('/')}/v1/chat/completions",
            {"model": "gpt-4o-mini", "messages": [{"role": "user", "content": "ping"}], "stream": False},
            headers={"Authorization": "Bearer sk-test-synthetic"},
        )
    results.append({"target": "proxy_burst", "note": "Check server for 429 if rate limit enabled"})
    print("[proxy_burst] 5 requests sent; check for 429 if rate limit active")

    out = os.path.join(os.path.dirname(__file__), "api_abuse_results.json")
    with open(out, "w") as f:
        json.dump({"results": results}, f, indent=2)
    print(f"Results written to {out}")


if __name__ == "__main__":
    main()
