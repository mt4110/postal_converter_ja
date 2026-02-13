# Onboarding Profiles

最終更新: 2026-02-12

`scripts/onboard.sh` は、ローカル導入を一括実行するためのスクリプトです。

## 目的

- Nix読込
- Docker起動
- `.env` 自動生成 (未作成時のみ)
- 必要サービス起動
- 疎通テスト
- 次の操作表示 (URL / 停止コマンド)

## 使い方

```bash
# 標準 (dev)
./scripts/onboard.sh

# プロファイル指定
./scripts/onboard.sh --profile dev
./scripts/onboard.sh --profile demo
./scripts/onboard.sh --profile sqlite-release

# 停止
./scripts/onboard.sh --stop
```

## profile: dev

用途:

- 開発作業をすぐ開始したいとき

実行内容:

- PostgreSQL / Redis 起動
- API 起動 (`http://127.0.0.1:3202`)
- Swagger 疎通確認 (`/docs`)
- Frontend 起動 (`http://127.0.0.1:3203`)

注意:

- Crawler は自動実行しないため、DBにデータが無い場合は検索結果が空になる

## profile: demo

用途:

- デモ前に最新データを取り込んだ状態で起動したいとき

実行内容:

- PostgreSQL / Redis 起動
- Crawler を `CRAWLER_RUN_ONCE=true` で1回実行
- API / Swagger 起動確認
- Frontend 起動確認

## profile: sqlite-release

用途:

- SQLite 配布成果物を生成したいとき

実行内容:

- PostgreSQL 起動
- Crawler 1回実行 (最新データ取り込み)
- `scripts/package_sqlite_release.sh` 実行
- `artifacts/sqlite/` に成果物生成
- 処理後に Docker サービス停止

成果物:

- `artifacts/sqlite/postal_codes-YYYYMMDD.sqlite3`
- `artifacts/sqlite/checksums-YYYYMMDD.txt`
- `artifacts/sqlite/manifest-YYYYMMDD.txt`

## ログ

- API: `/tmp/postal_converter_ja_onboard/api.log`
- Frontend: `/tmp/postal_converter_ja_onboard/frontend.log`

## 停止コマンド

```bash
./scripts/onboard.sh --stop
```
