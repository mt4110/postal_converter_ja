#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACT_DIR="${ROOT_DIR}/artifacts/sqlite"
VERSION_LABEL="${1:-$(date -u +%Y%m%d)}"
DB_FILENAME="postal_codes-${VERSION_LABEL}.sqlite3"
OUT_DB="${ARTIFACT_DIR}/${DB_FILENAME}"
CHECKSUM_FILE="${ARTIFACT_DIR}/checksums-${VERSION_LABEL}.txt"
MANIFEST_FILE="${ARTIFACT_DIR}/manifest-${VERSION_LABEL}.txt"

mkdir -p "$ARTIFACT_DIR"

"${ROOT_DIR}/scripts/build_sqlite_from_postgres.sh" "$OUT_DB"

if command -v sha256sum >/dev/null 2>&1; then
  DB_SHA256="$(sha256sum "$OUT_DB" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  DB_SHA256="$(shasum -a 256 "$OUT_DB" | awk '{print $1}')"
else
  echo "Neither sha256sum nor shasum is available."
  exit 1
fi

if stat -f "%z" "$OUT_DB" >/dev/null 2>&1; then
  DB_SIZE_BYTES="$(stat -f "%z" "$OUT_DB")"
else
  DB_SIZE_BYTES="$(stat -c "%s" "$OUT_DB")"
fi

ROW_COUNT="$(sqlite3 "$OUT_DB" "SELECT COUNT(*) FROM postal_codes;")"
GENERATED_AT_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

printf "%s  %s\n" "$DB_SHA256" "$(basename "$OUT_DB")" >"$CHECKSUM_FILE"
cat >"$MANIFEST_FILE" <<EOF
generated_at_utc=${GENERATED_AT_UTC}
artifact_file=$(basename "$OUT_DB")
size_bytes=${DB_SIZE_BYTES}
row_count=${ROW_COUNT}
sha256=${DB_SHA256}
EOF

echo "SQLite release artifact created:"
echo "  DB: $OUT_DB"
echo "  Checksums: $CHECKSUM_FILE"
echo "  Manifest: $MANIFEST_FILE"
