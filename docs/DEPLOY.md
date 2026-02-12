# Deployment Guide (GitHub Actions + Terraform Skeleton)

最終更新: 2026-02-12

このドキュメントは、`GitHub Actions + Terraform` でマルチプラットフォーム展開へ進めるための最小骨格を説明します。

## 1. 追加された骨格

- Workflow: `.github/workflows/terraform-multiplatform.yml`
- Terraform root modules:
  - `infra/terraform/platforms/aws`
  - `infra/terraform/platforms/gcp`
  - `infra/terraform/platforms/azure`
- サンプル変数:
  - `infra/terraform/environments/dev/aws.tfvars`
  - `infra/terraform/environments/dev/gcp.tfvars`
  - `infra/terraform/environments/dev/azure.tfvars`

## 2. GitHub Actions の使い方

### Pull Request 時

- `terraform fmt -check -recursive`
- `terraform init -backend=false`
- `terraform validate`

を `aws/gcp/azure` で実行します。

### 手動実行 (workflow_dispatch)

`action=plan` を選ぶと、3プラットフォームで `terraform plan` を実行します。
`action=apply` を選ぶと、AWS のみ `terraform apply` を実行します（確認トークン必須）。

> `gcp.tfvars` の `project_id = "replace-me"` が未置換の場合は、意図的に `plan` をスキップします。

CLI から実行する場合:

```bash
# validate
./scripts/run_terraform_workflow.sh --action validate --environment dev --ref codex/feature/v0.3.0

# plan
./scripts/run_terraform_workflow.sh --action plan --environment dev --ref codex/feature/v0.3.0

# apply (AWS only)
./scripts/run_terraform_workflow.sh --action apply --environment dev --confirm-apply APPLY_AWS --ref codex/feature/v0.3.0
```

## 3. #Node TODO の意味

今回の骨格には `#Node:` コメントを入れてあり、そのまま次タスクのノードとして使えます。

- `AWS OIDC`: 実装済み
- `#Node: module "network"...`
- `#Node: module "api_runtime"...`
- `GCP OIDC`: 実装済み
- `Azure OIDC`: 実装済み

この `#Node:` を順に実装していくと、`validate -> plan -> apply` の順で安全に拡張できます。

## 4. 次に設定する Secrets/Variables (実運用前)

### AWS

- Secret: `AWS_ROLE_TO_ASSUME`
- Variable: `AWS_REGION` (未設定時は `ap-northeast-1`)

GitHub Actions では `aws-actions/configure-aws-credentials@v4` を使い、OIDC で AssumeRole します。
`workflow_dispatch` の `action=plan` 実行時、`matrix.platform == aws` で有効になります。

IAM Role 側の trust policy は最小で以下をベースにしてください（`ORG/REPO` は置換）。

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "Federated": "arn:aws:iam::<ACCOUNT_ID>:oidc-provider/token.actions.githubusercontent.com"
      },
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "token.actions.githubusercontent.com:aud": "sts.amazonaws.com"
        },
        "StringLike": {
          "token.actions.githubusercontent.com:sub": "repo:ORG/REPO:*"
        }
      }
    }
  ]
}
```

### GCP

- Secret: `GCP_WORKLOAD_IDENTITY_PROVIDER`
- Secret: `GCP_SERVICE_ACCOUNT`
- Variable: `GCP_PROJECT_ID`

`google-github-actions/auth@v2` で Workload Identity Federation を利用します。
`GCP_WORKLOAD_IDENTITY_PROVIDER` と `GCP_SERVICE_ACCOUNT` が未設定の場合、GCP の `plan` はスキップされます。

### Azure

- Secret: `AZURE_CLIENT_ID`
- Secret: `AZURE_TENANT_ID`
- Secret: `AZURE_SUBSCRIPTION_ID`

`azure/login@v2` で OIDC ログインします。
3つの Secret が未設定の場合、Azure の `plan` はスキップされます。

Secrets/Variables をまとめて設定する場合:

```bash
# dry-run (値を表示)
./scripts/setup_github_oidc_vars.sh --use-dummy

# 実際に反映
./scripts/setup_github_oidc_vars.sh --use-dummy --write --repo mt4110/postal_converter_ja
```

## 5. ローカル検証

```bash
terraform -chdir=infra/terraform/platforms/aws init -backend=false
terraform -chdir=infra/terraform/platforms/aws validate

terraform -chdir=infra/terraform/platforms/gcp init -backend=false
terraform -chdir=infra/terraform/platforms/gcp validate

terraform -chdir=infra/terraform/platforms/azure init -backend=false
terraform -chdir=infra/terraform/platforms/azure validate
```
