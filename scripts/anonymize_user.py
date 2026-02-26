#!/usr/bin/env python3
"""
COM-02: GDPR Right to Erasure. Anonymize a user_id in audit_logs.
Usage: python scripts/anonymize_user.py <user_id>
Requires: DATABASE_URL env (same Postgres as proxy/dashboard).
No PII in logs; only hashed or NULL for user_id.
"""

import os
import sys
import hashlib
import psycopg2
from psycopg2.extras import execute_values

def main():
    if len(sys.argv) != 2:
        print("Usage: anonymize_user.py <user_id>", file=sys.stderr)
        sys.exit(1)
    user_id = sys.argv[1].strip()
    if not user_id:
        print("user_id must be non-empty", file=sys.stderr)
        sys.exit(1)

    url = os.environ.get("DATABASE_URL")
    if not url:
        print("DATABASE_URL is not set", file=sys.stderr)
        sys.exit(1)

    # Anonymize: replace with deterministic hash so referential integrity
    # (e.g. same user_id in multiple rows) is preserved as anonymized.
    anonymized = "anon_" + hashlib.sha256(user_id.encode()).hexdigest()[:16]

    try:
        conn = psycopg2.connect(url)
        cur = conn.cursor()
        cur.execute(
            "UPDATE audit_logs SET user_id = %s WHERE user_id = %s",
            (anonymized, user_id),
        )
        n = cur.rowcount
        conn.commit()
        cur.close()
        conn.close()
        print(f"Anonymized {n} row(s) for user_id (replaced with hash).")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
