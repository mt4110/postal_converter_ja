#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DB="${1:-$ROOT_DIR/storage/sqlite/postal_codes.sqlite3}"
TMP_CSV="$(mktemp -t postal_codes_sqlite.XXXXXX.csv)"

cleanup() {
  rm -f "$TMP_CSV"
}
trap cleanup EXIT

if ! command -v docker >/dev/null 2>&1; then
  echo "docker command not found."
  exit 1
fi

if ! command -v sqlite3 >/dev/null 2>&1; then
  echo "sqlite3 command not found. Run in nix shell:"
  echo "  nix develop --command bash -lc './scripts/build_sqlite_from_postgres.sh'"
  exit 1
fi

echo "[1/4] Checking PostgreSQL container readiness..."
docker exec postgres_container pg_isready -U postgres -d zip_code_db >/dev/null

echo "[2/4] Exporting postal_codes from PostgreSQL..."
docker exec postgres_container psql -U postgres -d zip_code_db -c "\copy (
  SELECT
    zip_code,
    prefecture_id,
    city_id,
    prefecture,
    city,
    COALESCE(town, '')
  FROM postal_codes
) TO STDOUT WITH CSV" >"$TMP_CSV"

mkdir -p "$(dirname "$OUT_DB")"
rm -f "$OUT_DB"

echo "[3/4] Creating SQLite schema..."
sqlite3 "$OUT_DB" "
PRAGMA journal_mode=DELETE;
PRAGMA synchronous=NORMAL;
CREATE TABLE postal_codes (
  zip_code TEXT NOT NULL,
  prefecture_id INTEGER NOT NULL,
  city_id TEXT NOT NULL,
  prefecture TEXT NOT NULL,
  city TEXT NOT NULL,
  town TEXT NOT NULL,
  PRIMARY KEY (zip_code, prefecture_id, city, town)
);
"

echo "[4/4] Importing CSV into SQLite..."
sqlite3 "$OUT_DB" ".mode csv" ".import '$TMP_CSV' postal_codes"
sqlite3 "$OUT_DB" "
CREATE INDEX idx_postal_codes_zip_code ON postal_codes (zip_code, town);
ANALYZE;
VACUUM;
"

ROW_COUNT="$(sqlite3 "$OUT_DB" "SELECT COUNT(*) FROM postal_codes;")"
echo "Done. SQLite DB generated at: $OUT_DB"
echo "Rows imported: $ROW_COUNT"
