# v0.8.0 (Beta) - AWS-first deployment baseline

## 概要

`v0.8.0` は「デプロイ基盤」に焦点を当てたリリースです。  
GitHub Actions + Terraform を AWS 先行で固定し、`plan/apply/destroy` の運用経路を整備しました。

## 主な変更点

### 1. AWS 先行の CI/CD IaC パイプライン

- `.github/workflows/terraform-multiplatform.yml` を AWS baseline に整理
- `workflow_dispatch` で `validate/plan/apply/destroy` を実行可能化
- `apply` は `APPLY_AWS`、`destroy` は `DESTROY_AWS` の確認トークン必須

### 2. OIDC と環境分離

- GitHub OIDC 前提で AWS AssumeRole 実行
- `infra/terraform/environments/{dev,stg,prod}/aws.tfvars` を整備
- skeleton モードでは `AWS_ROLE_TO_ASSUME` 未設定でも `plan` を実行可能

### 3. ロールバック運用の標準化

- `docs/DEPLOY.md` に rollback runbook（`destroy` 経路）を追加
- 実行証跡を追加:
  - `docs/TERRAFORM_OFFLINE_PLAN_EVIDENCE.md`
  - `docs/TERRAFORM_ROLLBACK_REHEARSAL_EVIDENCE.md`

### 4. 実行補助スクリプト改善

- `scripts/run_terraform_workflow.sh` が `destroy` 対応
- workflow run の監視時に run ID 解決のリトライを追加（取りこぼし対策）

## バージョン整合

- README/EN README バッジを `0.8.0` に更新
- `worker/api`, `worker/common`, `worker/crawler` の `Cargo.toml` を `0.8.0` に更新
- `frontend/package.json` を `0.8.0` に更新

## 次の焦点 (v0.9.0)

- SLO/SLI と障害時運用（Runbook/監査）の強化
- 監視・運用自動化の標準化
