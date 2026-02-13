# SQLite Monthly Release Operation Checklist

Last updated: 2026-02-13 (JST)

## 1) Preparation

- Confirm PostgreSQL container is healthy
- Confirm latest crawler source URL is reachable
- Confirm `artifacts/sqlite/` has enough disk space

## 2) Monthly Run Steps

1. Import latest postal data once:
   - `./scripts/onboard.sh --profile demo`
2. Build SQLite release package:
   - `./scripts/onboard.sh --profile sqlite-release`
3. Verify artifacts are generated:
   - `artifacts/sqlite/postal_codes-YYYYMMDD.sqlite3`
   - `artifacts/sqlite/checksums-YYYYMMDD.txt`
   - `artifacts/sqlite/manifest-YYYYMMDD.txt`

## 3) Verification

- Check checksum file includes SQLite artifact
- Spot-check query against generated SQLite file (sample zip code)
- Confirm manifest contains generation timestamp and source info

## 4) Rollback Drill (Monthly)

- Select latest known-good `data_version`
- Run rollback command (postgres/mysql as needed)
- Verify API lookup returns expected baseline result

## 5) Handoff / Release Notes

- Record data version and execution timestamp
- Record operator name and command logs
- Attach checksum and manifest to release note
