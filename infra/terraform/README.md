# Terraform AWS Baseline

`infra/terraform` には、AWS 先行展開の最小骨格を配置しています。

- `platforms/aws`: AWS 向け root module
- `platforms/gcp`: 将来拡張用の骨格（現行 workflow 対象外）
- `platforms/azure`: 将来拡張用の骨格（現行 workflow 対象外）
- `environments/dev/aws.tfvars`: dev 用の変数サンプル
- `environments/stg/aws.tfvars`: stg 用の変数サンプル
- `environments/prod/aws.tfvars`: prod 用の変数サンプル

## Local Validate

```bash
nix develop --command terraform -chdir=infra/terraform/platforms/aws init -backend=false
nix develop --command terraform -chdir=infra/terraform/platforms/aws validate
```

## Notes

- v0.8.0 は AWS に固定し、`validate -> plan -> apply` の再現性を優先します。
- `plan` は `AWS_ROLE_TO_ASSUME` 未設定時に offline mode で実行可能です（Skeleton検証用）。
- GCP/Azure は v0.9+ で段階的に再導入する前提です。
