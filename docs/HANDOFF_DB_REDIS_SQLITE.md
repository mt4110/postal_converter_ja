# Handoff Note (DB/Redis/SQLite)

最終更新: 2026-02-12

## 現在の実装状態

- OpenAPI/Swagger 実装済み
- `/openapi.json` と `/docs` を API に追加済み
- Nix 前提の起動手順へ README/Launcher を更新済み
- Rust/Cargo は `1.91.1` 固定、CI も同版へ更新済み

## 今回の進捗 (タスク1)

- 追加: `worker/common/tests/db_integration.rs`
- 追加: CI ジョブ `integration-db` (`.github/workflows/ci.yml`)
- コンパイル確認:
- `nix develop --command bash -lc 'cd worker && cargo test -p common --test db_integration --locked --no-run'` は成功
- Docker実機実行:
  - `cargo test -p common --test db_integration --locked -- --nocapture` で MySQL/PostgreSQL ともに `ok`

## 今回の進捗 (タスク2)

- `docker-compose.yml` に Redis サービスを追加 (`profile: cache`, port `3206`)
- API に Redis キャッシュを追加
  - 対象: `/postal_codes/{zip_code}`, `/postal_codes/search`, `/postal_codes/prefectures`, `/postal_codes/cities`
  - TTL: `REDIS_CACHE_TTL_SECONDS` (default `300`)
- Crawler に Redis キャッシュ失効処理を追加
  - `REDIS_URL` が設定されている場合、更新後に `FLUSHDB` を実行
- Docker実機実行:
  - API起動後に `/postal_codes/9990001` を2回呼び出し
  - `redis-cli EXISTS postal:zip:9990001` が `1` になり、キャッシュ格納を確認

## 今回の進捗 (タスク3)

- API に `DATABASE_TYPE=sqlite` を追加 (read-only PoC)
- SQLite 用環境変数 `SQLITE_DATABASE_PATH` を追加
- PostgreSQL から SQLite 生成するスクリプトを追加
  - `scripts/build_sqlite_from_postgres.sh`
- ドキュメントを追加
  - `docs/SQLITE_READONLY_POC.md`
- Docker実機実行:
  - PostgreSQLへテストデータ投入
  - スクリプトで SQLite 生成
  - SQLiteモードAPI起動後、`/health`, `/postal_codes/{zip}` , `/postal_codes/search` の応答を確認

## 今回の進捗 (タスク4)

- SQLite 配布パッケージスクリプトを追加
  - `scripts/package_sqlite_release.sh`
  - 出力: `artifacts/sqlite/postal_codes-YYYYMMDD.sqlite3`
  - 付属: `checksums-YYYYMMDD.txt`, `manifest-YYYYMMDD.txt`
- Crawler に `CRAWLER_RUN_ONCE` を追加
  - CI/バッチで1サイクル取り込み後に終了可能
- GitHub Actions を追加
  - `.github/workflows/sqlite-release.yml`
  - 手動実行で「PostgreSQL起動 -> Crawler1回実行 -> SQLite生成 -> Artifact保存 -> 任意Release添付」

## タスク2の関連ファイル

- `docker-compose.yml`
- `worker/api/src/main.rs`
- `worker/api/Cargo.toml`
- `worker/crawler/src/main.rs`
- `worker/crawler/Cargo.toml`
- `worker/api/.env.example`
- `worker/crawler/.env.example`
- `worker/crawler/.env.unencrypted`

## ローカル実行時の注意

- Docker daemon 未起動だと DB 統合テストは実行できない
- 失敗例:
- `Cannot connect to the Docker daemon at unix:///Users/.../.docker/run/docker.sock`

## 次の実行コマンド

```bash
source /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
mkdir -p storage/mysql/mysql_data storage/postgres/postgres_data
docker compose up -d mysql postgres
nix develop --command bash -lc 'cd worker && MYSQL_DATABASE_URL=mysql://mysql_user:u_password@127.0.0.1:3204/zip_code_db POSTGRES_DATABASE_URL=postgres://postgres:postgres_password@127.0.0.1:3205/zip_code_db cargo test -p common --test db_integration --locked -- --nocapture'
docker compose down -v
```

SQLite PoC 検証:

```bash
docker compose up -d postgres
nix develop --command bash -lc "./scripts/build_sqlite_from_postgres.sh"
DATABASE_TYPE=sqlite SQLITE_DATABASE_PATH=storage/sqlite/postal_codes.sqlite3 nix develop --command bash -lc "cd worker/api && cargo run --release --bin api"
```

SQLite 配布成果物生成:

```bash
docker compose up -d postgres
nix develop --command bash -lc "cd worker/crawler && CRAWLER_RUN_ONCE=true cargo run --release --bin crawler"
nix develop --command bash -lc "./scripts/package_sqlite_release.sh"
docker compose down -v
```

## 未着手

- なし (タスク1〜4完了)
