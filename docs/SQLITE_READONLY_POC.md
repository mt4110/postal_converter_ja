# SQLite Read-Only PoC

最終更新: 2026-02-12

## 目的

- `DATABASE_TYPE=sqlite` で API の read-only 検索を実行できるようにする
- iOS/Android/組み込み環境向けに単一ファイルDB配布の可能性を検証する

## 対象機能

- `GET /postal_codes/{zip_code}`
- `GET /postal_codes/search?address=...&limit=...`
- `GET /postal_codes/prefectures`
- `GET /postal_codes/cities?prefecture_id=...`

## 使い方

1. PostgreSQL にデータを投入済みの状態にする
2. SQLite DB を生成

```bash
nix develop --command bash -lc "./scripts/build_sqlite_from_postgres.sh"
```

配布用アーティファクトを生成する場合:

```bash
nix develop --command bash -lc "./scripts/package_sqlite_release.sh"
```

成果物:

- `artifacts/sqlite/postal_codes-YYYYMMDD.sqlite3`
- `artifacts/sqlite/checksums-YYYYMMDD.txt`
- `artifacts/sqlite/manifest-YYYYMMDD.txt`

3. API を SQLite モードで起動

```bash
DATABASE_TYPE=sqlite \
SQLITE_DATABASE_PATH=storage/sqlite/postal_codes.sqlite3 \
nix develop --command bash -lc "cd worker/api && cargo run --release --bin api"
```

## 制約

- SQLite は read-only PoC 前提
- Crawler から SQLite へ直接投入する機能は未実装
- 更新は PostgreSQL/MySQL で行い、SQLite は再生成して配布する運用
- 大量同時書き込み用途には不向き

## データ更新運用案

1. Crawler で PostgreSQL を更新
2. `scripts/build_sqlite_from_postgres.sh` で SQLite を再生成
3. 生成した `storage/sqlite/postal_codes.sqlite3` を配布物へ組み込む

## リリース自動化 (GitHub Actions)

- ワークフロー: `.github/workflows/sqlite-release.yml`
- `workflow_dispatch` で手動実行
- 実行内容:
- PostgreSQL 起動
- Crawler を `CRAWLER_RUN_ONCE=true` で1サイクル実行
- SQLite アーティファクト生成
- Artifact 保存
- 必要時のみ GitHub Release へ添付
