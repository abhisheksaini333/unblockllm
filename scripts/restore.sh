#!/usr/bin/env bash
# Phase 7: Restore Postgres audit DB from a backup. Run from repo root.
# Usage: ./scripts/restore.sh <path-to-audit_YYYYMMDD_HHMMSS.sql.gz>
# Requires: DATABASE_URL, gunzip, psql. Drops and recreates schema for clean restore (destructive).

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <path-to-audit_YYYYMMDD_HHMMSS.sql.gz>" >&2
  exit 1
fi

BACKUP_FILE="$1"
if [[ ! -f "$BACKUP_FILE" ]]; then
  echo "File not found: $BACKUP_FILE" >&2
  exit 1
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "DATABASE_URL is not set." >&2
  exit 1
fi

echo "Restoring from $BACKUP_FILE into DATABASE_URL..."
gunzip -c "$BACKUP_FILE" | psql "$DATABASE_URL" -v ON_ERROR_STOP=1
echo "Restore finished. Verify with: psql \$DATABASE_URL -c 'SELECT COUNT(*) FROM audit_logs;'"
