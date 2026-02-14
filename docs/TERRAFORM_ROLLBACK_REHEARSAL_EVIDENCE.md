# Terraform Rollback Rehearsal Evidence (v0.8.0)

- Executed at (JST): 2026-02-13 16:58:52 JST
- Branch: `feature/v0.8.0`
- Commit: `bb6204e`
- Mode: ローカル offline rehearsal（AWS credentials なし）

## Goal

`v0.8.0` の Exit Criteria にある「Deployment rollback path is documented and tested once」を満たすため、AWS未接続で `apply -> rollback(destroy)` を1回検証した。

## Commands

```bash
terraform -chdir=infra/terraform/platforms/aws init -backend=false -no-color
terraform -chdir=infra/terraform/platforms/aws apply -auto-approve -refresh=false -input=false -lock=false -var-file=../../environments/dev/aws.tfvars -no-color
terraform -chdir=infra/terraform/platforms/aws output -no-color
terraform -chdir=infra/terraform/platforms/aws destroy -auto-approve -refresh=false -input=false -lock=false -var-file=../../environments/dev/aws.tfvars -no-color
terraform -chdir=infra/terraform/platforms/aws output -no-color || true
```

## Output (excerpt)

```txt
$ terraform ... apply ... -var-file=../../environments/dev/aws.tfvars -no-color
Changes to Outputs:
  + skeleton_summary = "terraform skeleton ready: platform=aws, env=dev, region=ap-northeast-1"
Apply complete! Resources: 0 added, 0 changed, 0 destroyed.

$ terraform ... output -no-color
skeleton_summary = "terraform skeleton ready: platform=aws, env=dev, region=ap-northeast-1"

$ terraform ... destroy ... -var-file=../../environments/dev/aws.tfvars -no-color
Changes to Outputs:
  - skeleton_summary = "terraform skeleton ready: platform=aws, env=dev, region=ap-northeast-1" -> null
Destroy complete! Resources: 0 destroyed.

$ terraform ... output -no-color
Warning: No outputs found
```

## Result

- `apply` 後に `skeleton_summary` output が state に保存されることを確認
- `destroy` 後に output が消えることを確認（rollback最小経路の検証完了）
- 現段階（skeleton）では実リソース作成なしのため、`Resources: 0` は想定どおり
