# Terraform Offline Plan Evidence (v0.8.0)

- Executed at (JST): 2026-02-13 16:12:07 JST
- Branch: `feature/v0.8.0`
- Commit: `07c47d9`
- Mode: AWS credentials/secrets なしの offline plan 検証

## Commands

```bash
terraform version
terraform -chdir=infra/terraform/platforms/aws init -backend=false -no-color
terraform -chdir=infra/terraform/platforms/aws validate -no-color
terraform -chdir=infra/terraform/platforms/aws plan -refresh=false -input=false -lock=false -var-file=../../environments/dev/aws.tfvars -no-color
terraform -chdir=infra/terraform/platforms/aws plan -refresh=false -input=false -lock=false -var-file=../../environments/stg/aws.tfvars -no-color
terraform -chdir=infra/terraform/platforms/aws plan -refresh=false -input=false -lock=false -var-file=../../environments/prod/aws.tfvars -no-color
```

## Output (excerpt)

```txt
$ terraform version
Terraform v1.14.5
on darwin_arm64

$ terraform -chdir=infra/terraform/platforms/aws init -backend=false -no-color
Initializing provider plugins...
- Reusing previous version of hashicorp/aws from the dependency lock file
- Using previously-installed hashicorp/aws v5.100.0
Terraform has been successfully initialized!

$ terraform -chdir=infra/terraform/platforms/aws validate -no-color
Success! The configuration is valid.

$ terraform ... -var-file=../../environments/dev/aws.tfvars -no-color
Changes to Outputs:
  + skeleton_summary = "terraform skeleton ready: platform=aws, env=dev, region=ap-northeast-1"

$ terraform ... -var-file=../../environments/stg/aws.tfvars -no-color
Changes to Outputs:
  + skeleton_summary = "terraform skeleton ready: platform=aws, env=stg, region=ap-northeast-1"

$ terraform ... -var-file=../../environments/prod/aws.tfvars -no-color
Changes to Outputs:
  + skeleton_summary = "terraform skeleton ready: platform=aws, env=prod, region=ap-northeast-1"
```

## Result

- `init` / `validate` / `plan(dev|stg|prod)` はすべて成功
- AWS アカウント未接続でも v0.8.0 の Terraform skeleton 検証が可能であることを確認
