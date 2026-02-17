# v0.9.0 Release Notes

## 日本語

### タイトル
`v0.9.0 (Beta) - 運用準備パッケージとランチャーUX強化`

### 本文
`v0.8.0` から `v0.9.0` では、運用準備とオペレーター体験を中心に改善しました。  
比較範囲: `v0.8.0...v0.9.0`（53 commits / 64 files changed）

主な差分:
- 運用標準の追加
  - v0.9.0 実行計画、運用Runbook（更新失敗 / キャッシュ障害 / DBロールバック）
  - SLO/SLI、アラートポリシー、ダッシュボード標準
  - 月次監査、NDAオンボーディング、見積テンプレート
  - Human test シナリオ、v0.9タスク用 Issue Template
- ランチャーの運用UXと信頼性を改善
  - API/Frontend を管理プロセスとして起動し、PID復旧と Ctrl+C 復帰を強化
  - 起動完了判定に readiness チェックを導入
  - マウスクリック操作、URL/ログ表示、ローディング視認性を改善
- フロントエンドの郵便番号検索を修正
  - Postal SDK の `fetch Illegal invocation` 問題を修正
- デプロイ基盤を強化
  - Helm-first 構成、ArgoCD環境分離（dev/stg/prod）、Ingress/NetworkPolicy、External Secrets 雛形
  - CI に Helm lint/template と kubeconform 検証を追加
- 依存関係アップデート（v0.9.0タグ取り込み分）
  - frontend: `@typescript-eslint/parser`, `framer-motion`, `styled-components`
  - worker/api: `redis`, `rusqlite`
  - worker/crawler: `redis`, `zip`
  - GitHub Actions: `dtolnay/rust-toolchain`

互換性:
- APIコントラクトの破壊的変更はありません。
- 主な変更はドキュメント追加と運用/UX改善です。

関連PR:
- `#94` v0.9.0 operations readiness package and launcher UX hardening
- `#84` - `#91` dependabot updates merged into v0.9.0

---

## English

### Title
`v0.9.0 (Beta) - Operations Readiness Package and Launcher UX Hardening`

### Body
From `v0.8.0` to `v0.9.0`, this release focuses on operational readiness and operator experience.  
Comparison range: `v0.8.0...v0.9.0` (53 commits / 64 files changed)

Key changes:
- Added operational standards and readiness assets
  - v0.9.0 execution plan and runbooks (update failure / cache incident / DB rollback)
  - SLO/SLI, alert policy, and dashboard standards
  - Monthly audit, NDA onboarding, and estimation templates
  - Human test scenario and v0.9 task issue template
- Improved launcher UX and reliability
  - Managed API/frontend processes with PID recovery and safer Ctrl+C behavior
  - Readiness checks before marking services as started
  - Mouse-click actions, clearer URL/log guidance, and better loading visibility
- Fixed frontend postal lookup behavior
  - Resolved Postal SDK `fetch Illegal invocation`
- Strengthened deployment baseline
  - Helm-first structure, ArgoCD env split (dev/stg/prod), ingress/network policy, External Secrets skeleton
  - Added Helm lint/template and kubeconform validation in CI
- Dependency updates included in v0.9.0 tag
  - frontend: `@typescript-eslint/parser`, `framer-motion`, `styled-components`
  - worker/api: `redis`, `rusqlite`
  - worker/crawler: `redis`, `zip`
  - GitHub Actions: `dtolnay/rust-toolchain`

Compatibility:
- No breaking API contract changes.
- Most changes are additive docs plus operations/UX hardening.

Related PRs:
- `#94` v0.9.0 operations readiness package and launcher UX hardening
- `#84` - `#91` dependabot updates merged into v0.9.0
