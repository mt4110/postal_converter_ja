# Docs Inventory (v0.9.1)

最終更新: 2026-02-17 (JST)

## 1. 目的

`v0.9.1` の非人間タスクとして、ドキュメントの棚卸しを行い、
不要ドキュメントの削除候補と `.gitignore` 運用方針を明確化する。

## 2. 今回の確定対応

- 削除: `docs/V0_9_0_SIGNED_TAG_DRAFT.md`
  - 理由: `v0.9.0` の公開Releaseが作成済みで、ドラフト運用手順は役目を終えたため。
  - 代替: `docs/V0_9_0_RELEASE_NOTES.md`（公開用ノートの正本）

## 3. 継続管理 (Keep)

- `docs/V0_9_0_RELEASE_NOTES.md`
- `docs/V0_9_0_RUNBOOK_EXECUTION_PLAN.md`
- `docs/V0_9_0_ACCEPTANCE.md`
- `docs/HUMAN_TEST_SCENARIO_v0_9_0.md`
- `docs/RUNBOOK_*_v0_9_0.md`

理由:
- `v0.9.2` で実施予定の `QA-02/03/04` に直接参照されるため。

## 4. 次回候補 (Review Candidates)

- `docs/HANDOFF_v0.8.3.md`
- `docs/TASK_SPLIT_DB_REDIS_SQLITE.md`

候補理由:
- バージョンが古く、現行運用の一次導線からは外れている。
- ただし参照履歴・意思決定根拠として残す価値があるため、即削除はしない。

## 5. `.gitignore` 方針 (docs)

- 追記済みルール:
  - `docs/*_LOCAL.md`

運用ルール:
- 個人作業メモや一時草稿は `*_LOCAL.md` 命名で作成し、リポジトリ追跡対象外にする。
- 共有対象のドキュメントは通常命名で作成し、必ずレビューを通して残す。
