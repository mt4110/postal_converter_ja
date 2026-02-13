# v0.7.0 (Beta) - Sales-ready onboarding kit

## 概要

`v0.7.0` は「導入加速」に焦点を当てたリリースです。  
ローカル導入の再現性と、顧客向けデモの即応性を強化しました。

## 主な変更点

### 1. 導入SDKサンプルの拡張

- Frontend の導入サンプルを 3 ユースケース構成に拡張:
  - EC 配送フォーム
  - 会員登録フォーム
  - コールセンター入力支援フォーム

### 2. 導入・運用ドキュメントの標準化

- 受託導入向けチェックリストを追加:
  - `docs/CONTRACTOR_ONBOARDING_CHECKLIST.md`
- SQLite 月次運用チェックリストを追加:
  - `docs/SQLITE_MONTHLY_OPERATION_CHECKLIST.md`
- v0.7.0 以降の実行計画を追加:
  - `docs/V0_7_TO_V1_EXECUTION_PLAN.md`

### 3. オンボーディング実機証跡の追加

- 同一ホスト端末での `onboard -> curl -> stop` の証跡を追加:
  - `docs/ONBOARDING_REHEARSAL_EVIDENCE.md`

## バージョン整合

- README バッジを `0.7.0` に更新
- `worker/api`, `worker/common`, `worker/crawler` の `Cargo.toml` を `0.7.0` に更新
- `frontend/package.json` を `0.7.0` に更新

## 次の焦点 (v0.8.0)

- GitHub Actions + Terraform の本番適用基盤を 1 クラウドに固定して完成させる
- OIDC を前提に `plan/apply` 再現性を高める
