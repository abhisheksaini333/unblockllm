#!/usr/bin/env python3
"""
Pen test: Masking bypass. Synthetic data only — no real PII.
Attempts to confuse NER/regex with obfuscated patterns.
"""

import os
import sys
import json
import urllib.request
import urllib.error

PROXY_URL = os.environ.get("PROXY_URL", "http://127.0.0.1:8080")
API_KEY = os.environ.get("OPENAI_API_KEY", "sk-test-synthetic-key-for-pentest")


def post_chat(content: str) -> tuple[int, dict | None]:
    """POST to /v1/chat/completions. Returns (status_code, body_or_none)."""
    url = f"{PROXY_URL.rstrip('/')}/v1/chat/completions"
    body = {
        "model": "gpt-4o-mini",
        "messages": [{"role": "user", "content": content}],
        "stream": False,
    }
    req = urllib.request.Request(
        url,
        data=json.dumps(body).encode(),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {API_KEY}",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as r:
            return r.status, json.loads(r.read().decode()) if r.length else {}
    except urllib.error.HTTPError as e:
        return e.code, None
    except Exception as e:
        print(f"Request failed: {e}", file=sys.stderr)
        return -1, None


def main() -> None:
    # Synthetic only: obfuscated-looking strings that might bypass naive regex
    cases = [
        ("Plain synthetic email", "Contact support at testuser@example-dot-com."),
        ("Email with spaces (invalid but might leak)", "Email: test at example dot com"),
        ("Synthetic SSN-like without dashes", "ID 123456789 might be a number."),
        ("Unicode in synthetic local part", "Reply to u\u200bser@example.com for info."),
    ]
    results = []
    for name, payload in cases:
        status, body = post_chat(payload)
        # We only check that the proxy responds; we do not send payload to real OpenAI
        # In a full run, we would assert that response content does not contain raw PII
        results.append({"test": name, "status": status, "payload_preview": payload[:50]})
        print(f"[{name}] status={status}")
    # Write machine-readable result for REPORT
    out = os.path.join(os.path.dirname(__file__), "masking_bypass_results.json")
    with open(out, "w") as f:
        json.dump({"results": results}, f, indent=2)
    print(f"Results written to {out}")


if __name__ == "__main__":
    main()
