#!/usr/bin/env bash
# Phase 7: Backup Postgres (audit DB) and Redis. Run from repo root.
# Usage: ./scripts/backup.sh [BACKUP_DIR]
# Requires: pg_dump (Postgres), redis-cli (Redis) if REDIS_URL set.
# Set DATABASE_URL and optionally REDIS_URL. Backups go to BACKUP_DIR (default: ./backups).

set -euo pipefail

BACKUP_DIR="${1:-./backups}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p "$BACKUP_DIR"

echo "Backup started at $(date -Iseconds)"

# Postgres (audit DB)
if [[ -n "${DATABASE_URL:-}" ]]; then
  PG_FILE="$BACKUP_DIR/audit_$TIMESTAMP.sql"
  pg_dump "$DATABASE_URL" --no-owner --no-acl -f "$PG_FILE"
  gzip -f "$PG_FILE"
  echo "Postgres backup: ${PG_FILE}.gz"
else
  echo "DATABASE_URL not set; skipping Postgres backup."
fi

# Redis (optional: BGSAVE then copy RDB if local file known, or use redis-cli BGSAVE)
if [[ -n "${REDIS_URL:-}" ]]; then
  REDIS_FILE="$BACKUP_DIR/redis_$TIMESTAMP.rdb"
  if command -v redis-cli &>/dev/null; then
    redis-cli -u "$REDIS_URL" BGSAVE
    echo "Redis BGSAVE triggered. For RDB copy, use Redis persistence dir or redis-cli --rdb $REDIS_FILE (if supported)."
  else
    echo "redis-cli not found; skipping Redis snapshot."
  fi
else
  echo "REDIS_URL not set; skipping Redis."
fi

echo "Backup finished at $(date -Iseconds). Directory: $BACKUP_DIR"
