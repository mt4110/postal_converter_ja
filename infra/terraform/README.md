# Terraform Multi-Platform Skeleton

`infra/terraform` には、マルチプラットフォーム展開の最小骨格を配置しています。

- `platforms/aws`: AWS 向け root module
- `platforms/gcp`: GCP 向け root module
- `platforms/azure`: Azure 向け root module
- `environments/dev/*.tfvars`: dev 用の変数サンプル

## Local Validate

```bash
terraform -chdir=infra/terraform/platforms/aws init -backend=false
terraform -chdir=infra/terraform/platforms/aws validate

terraform -chdir=infra/terraform/platforms/gcp init -backend=false
terraform -chdir=infra/terraform/platforms/gcp validate

terraform -chdir=infra/terraform/platforms/azure init -backend=false
terraform -chdir=infra/terraform/platforms/azure validate
```

## Notes

- `#Node:` コメントは、この骨格から本実装へ進めるための TODO ノードです。
- `gcp.tfvars` の `project_id = "replace-me"` は必ず実値に置換してください。
