# Deployment Guide (GitHub Actions + Terraform AWS Baseline)

最終更新: 2026-02-12

このドキュメントは、`GitHub Actions + Terraform` で AWS を先行ターゲットとして運用する最小構成を説明します。

## 1. 追加された骨格

- Workflow: `.github/workflows/terraform-multiplatform.yml` (AWS baseline)
- Terraform root modules:
  - `infra/terraform/platforms/aws`
- サンプル変数:
  - `infra/terraform/environments/dev/aws.tfvars`

## 2. GitHub Actions の使い方

### Pull Request 時

- `terraform fmt -check -recursive`
- `terraform init -backend=false`
- `terraform validate`

を `aws` で実行します。

### 手動実行 (workflow_dispatch)

`action=plan` を選ぶと、AWS で `terraform plan` を実行します。  
`action=apply` を選ぶと、AWS で `terraform apply` を実行します（確認トークン必須）。  
`action=destroy` を選ぶと、AWS で `terraform destroy` を実行します（確認トークン必須）。

- `AWS_ROLE_TO_ASSUME` が未設定の場合: `plan` は offline mode で実行（Skeleton検証用途）
- `AWS_ROLE_TO_ASSUME` が設定済みの場合: OIDC で AssumeRole して `plan/apply` を実行

CLI から実行する場合:

```bash
# validate
./scripts/run_terraform_workflow.sh --action validate --environment dev --ref feature/v0.8.0

# plan
./scripts/run_terraform_workflow.sh --action plan --environment dev --ref feature/v0.8.0

# apply (AWS only)
./scripts/run_terraform_workflow.sh --action apply --environment dev --confirm-apply APPLY_AWS --ref feature/v0.8.0

# destroy (AWS only)
./scripts/run_terraform_workflow.sh --action destroy --environment dev --confirm-destroy DESTROY_AWS --ref feature/v0.8.0
```

## 3. v0.8.0 の運用方針

- 先行ターゲットは AWS に固定
- `validate -> plan -> apply` の順で dev 環境を安定化
- GCP/Azure は v0.9+ で再導入

## 4. 次に設定する Secrets/Variables (実運用前)

### AWS

- Secret: `AWS_ROLE_TO_ASSUME`
- Variable: `AWS_REGION` (未設定時は `ap-northeast-1`)

`apply` / `destroy` を実行するには AWS アカウントと IAM Role（OIDC trust policy 設定済み）が必須です。  
`plan` は Skeleton 段階では secret 未設定でも実行できます（offline mode）。

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
          "token.actions.githubusercontent.com:aud": "sts.amazonaws.com",
          "token.actions.githubusercontent.com:job_workflow_ref": "ORG/REPO/.github/workflows/terraform-multiplatform.yml@refs/heads/main"
        },
        "StringLike": {
          "token.actions.githubusercontent.com:sub": "repo:ORG/REPO:ref:refs/heads/main"
        }
      }
    }
  ]
}
```

AWS Secrets/Variables をまとめて設定する場合:

```bash
# dry-run (値を表示)
./scripts/setup_github_oidc_vars.sh --use-dummy

# 実際に反映
./scripts/setup_github_oidc_vars.sh --use-dummy --write --repo mt4110/postal_converter_ja
```

## 5. ローカル検証

`terraform` がローカルシェルで見えない場合は Nix dev shell 経由で実行してください（Nix では OpenTofu 互換の `terraform` コマンドを提供）。
Homebrew の `terraform` は 1.5.7 で固定される場合があるため、バージョン差異回避のためにも Nix 利用を推奨します。

```bash
nix develop --command terraform version
nix develop --command terraform fmt -check -recursive infra/terraform
nix develop --command terraform -chdir=infra/terraform/platforms/aws init -backend=false
nix develop --command terraform -chdir=infra/terraform/platforms/aws validate

```

## 6. 実行証跡 (v0.8.0)

- offline plan 証跡: `docs/TERRAFORM_OFFLINE_PLAN_EVIDENCE.md`
- rollback rehearsal 証跡: `docs/TERRAFORM_ROLLBACK_REHEARSAL_EVIDENCE.md`

## 7. Rollback Path (v0.8.0 minimum)

Skeleton 段階では、rollback は `terraform destroy` を最小経路とします。

```bash
terraform -chdir=infra/terraform/platforms/aws apply -auto-approve -refresh=false -input=false -lock=false -var-file=../../environments/dev/aws.tfvars
terraform -chdir=infra/terraform/platforms/aws destroy -auto-approve -refresh=false -input=false -lock=false -var-file=../../environments/dev/aws.tfvars
```

GitHub Actions から実行する場合（確認トークン付き）:

```bash
./scripts/run_terraform_workflow.sh --action destroy --environment dev --confirm-destroy DESTROY_AWS --ref feature/v0.8.0
```

実AWS運用に入った後（OIDC + 実リソースあり）は、同じ経路で `dev` から先に検証してから `stg/prod` に展開します。
