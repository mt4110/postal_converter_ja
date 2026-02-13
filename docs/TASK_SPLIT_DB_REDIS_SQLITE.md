# 実行タスク分割 (DB統合テスト / Redis / SQLite)

最終更新: 2026-02-12

## 方針

- `bootstrap.sh + mise.toml + make setup` は今回スコープ外
- Nix + Docker 前提で進行
- まず品質担保 (DB統合テスト) を完了し、その後に性能改善 (Redis) と配布拡張 (SQLite) を進める

## タスク一覧

### タスク1: PostgreSQL + MySQL 統合テスト完成

ステータス: 完了

- 1-1. `worker/common/tests/db_integration.rs` で DB round-trip テスト実装
- 1-2. CI に `integration-db` ジョブ追加 (`docker compose` で2DB起動)
- 1-3. ローカル実行手順を確定
- 完了条件
- CI の `integration-db` が green
- Postgres と MySQL の両テストが実行される

### タスク2: Redis キャッシュ (docker compose オプション)

ステータス: 完了

- 2-1. `docker-compose.yml` に Redis サービス追加 (profile: `cache`)
- 2-2. API にキャッシュ層追加 (`/postal_codes/{zip_code}`, `/postal_codes/search`)
- 2-3. Crawler 完了時のキャッシュ失効方式を実装
- 2-4. `README` に有効化手順を追記
- 完了条件
- Redis 無効時でも従来動作
- Redis 有効時にヒット率ログが確認可能

### タスク3: SQLite read-only PoC

ステータス: 完了

- 3-1. `DATABASE_TYPE=sqlite` の read-only モード設計
- 3-2. 最小クエリ (`zip`, `search`) の実装
- 3-3. データ生成フロー (Postgres dump -> SQLite) の試作
- 3-4. モバイル/組み込み向け利用制約をドキュメント化
- 完了条件
- SQLite 単体で API の主要検索が動作
- 制約事項 (更新方法・容量・速度) が明文化されている

### タスク4: SQLite 配布運用の自動化 (生成→成果物→リリース)

ステータス: 完了

- 4-1. `scripts/package_sqlite_release.sh` を追加
- 4-2. Crawler に `CRAWLER_RUN_ONCE` を追加（CI/バッチ用）
- 4-3. GitHub Actions `sqlite-release.yml` を追加（手動実行）
- 4-4. チェックサム/manifest 付き成果物を `artifacts/sqlite/` へ出力
- 完了条件
- 手動実行で SQLite 配布成果物が生成される
- 必要時に GitHub Release へ添付できる

## 分担しやすい境界

- AI-A: タスク1のみ担当 (テスト/CI)
- AI-B: タスク2のみ担当 (Redis)
- AI-C: タスク3のみ担当 (SQLite PoC)
- AI-D: タスク4のみ担当 (SQLite 配布自動化)

各AIは、自分のタスク以外のファイルを編集しないこと。
