#!/usr/bin/env bash
set -euo pipefail

# Generates or checks SQLx offline metadata for the Rust workspace.
# Usage:
#   ./scripts/sqlx_prepare_workspace.sh          # regenerate .sqlx metadata
#   ./scripts/sqlx_prepare_workspace.sh --check  # verify metadata is up-to-date

MODE="${1:-}"
if [[ "$MODE" != "" && "$MODE" != "--check" ]]; then
  echo "Usage: $0 [--check]" >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required" >&2
  exit 1
fi

if ! cargo sqlx --help >/dev/null 2>&1; then
  echo "cargo-sqlx is required. Install with:" >&2
  echo "  cargo install sqlx-cli --no-default-features --features sqlite" >&2
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREPARE_DB="${ROOT_DIR}/.sqlx/prepare.db"

mkdir -p "${ROOT_DIR}/.sqlx"
rm -f "${PREPARE_DB}"

python3 - "$ROOT_DIR" "$PREPARE_DB" <<'PY'
import sqlite3
import sys
from pathlib import Path

root = Path(sys.argv[1])
db_path = Path(sys.argv[2])

content_sql = root / "crates/iqrah-storage/migrations_content/20241126000001_unified_content_schema.sql"
user_sql = root / "crates/iqrah-storage/migrations_user/20241126000001_user_schema.sql"
sessions_sql = root / "crates/iqrah-storage/migrations_user/20250115000001_sessions.sql"

def normalize_schema_version(sql: str) -> str:
    sql = sql.replace(
        "CREATE TABLE schema_version (",
        "CREATE TABLE IF NOT EXISTS schema_version (",
        1,
    )
    sql = sql.replace(
        "INSERT INTO schema_version (version, description)",
        "INSERT OR IGNORE INTO schema_version (version, description)",
        1,
    )
    return sql

conn = sqlite3.connect(db_path)
try:
    conn.executescript(normalize_schema_version(content_sql.read_text(encoding="utf-8")))
    conn.executescript(normalize_schema_version(user_sql.read_text(encoding="utf-8")))
    conn.executescript(sessions_sql.read_text(encoding="utf-8"))
finally:
    conn.close()
PY

export DATABASE_URL="sqlite://${PREPARE_DB}"
export SQLX_OFFLINE=false

if [[ "$MODE" == "--check" ]]; then
  cargo sqlx prepare --workspace --check
else
  cargo sqlx prepare --workspace
fi
