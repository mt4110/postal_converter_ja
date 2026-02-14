# Kubernetes Rehearsal Evidence (v0.8.3)

- Executed at (JST): 2026-02-14 13:46:57 JST
- Branch: `codex/v0.8.3-external-secrets`
- Base Commit: `c8c2449`（このコミット上で v0.8.3 追加差分を作業中に実行）
- Scope: Helm/ArgoCD env分離 + ExternalSecret/Ingress/NetworkPolicy + CIマニフェスト検証

## 1) Helm validation commands

```bash
/opt/homebrew/opt/helm@3/bin/helm version --short
/opt/homebrew/opt/helm@3/bin/helm lint deploy/helm/postal-converter-ja

/opt/homebrew/opt/helm@3/bin/helm template postal-converter-ja deploy/helm/postal-converter-ja \
  --namespace postal-converter-ja > /tmp/pcj-default.yaml

/opt/homebrew/opt/helm@3/bin/helm template postal-converter-ja deploy/helm/postal-converter-ja \
  --namespace postal-converter-ja-dev \
  -f deploy/helm/postal-converter-ja/values-dev.yaml > /tmp/pcj-dev.yaml

/opt/homebrew/opt/helm@3/bin/helm template postal-converter-ja deploy/helm/postal-converter-ja \
  --namespace postal-converter-ja-stg \
  -f deploy/helm/postal-converter-ja/values-stg.yaml > /tmp/pcj-stg.yaml

/opt/homebrew/opt/helm@3/bin/helm template postal-converter-ja deploy/helm/postal-converter-ja \
  --namespace postal-converter-ja-prod \
  -f deploy/helm/postal-converter-ja/values-prod.yaml > /tmp/pcj-prod.yaml
```

## 2) Helm validation result (excerpt)

```txt
v3.20.0+gb2e4314

==> Linting deploy/helm/postal-converter-ja
[INFO] Chart.yaml: icon is recommended

1 chart(s) linted, 0 chart(s) failed
```

Rendered resources check:

```txt
/tmp/pcj-dev.yaml: kind NetworkPolicy / Ingress / ExternalSecret を確認
/tmp/pcj-stg.yaml: kind NetworkPolicy / Ingress / ExternalSecret を確認
/tmp/pcj-prod.yaml: kind NetworkPolicy / Ingress / ExternalSecret を確認
```

## 3) kubeconform validation commands

```bash
kubeconform -v
kubeconform -strict -summary -ignore-missing-schemas /tmp/pcj-default.yaml
kubeconform -strict -summary -ignore-missing-schemas /tmp/pcj-dev.yaml
kubeconform -strict -summary -ignore-missing-schemas /tmp/pcj-stg.yaml
kubeconform -strict -summary -ignore-missing-schemas /tmp/pcj-prod.yaml
```

## 4) kubeconform result

```txt
v0.7.0

Summary: 3 resources found in 1 file - Valid: 3, Invalid: 0, Errors: 0, Skipped: 0
Summary: 6 resources found in 1 file - Valid: 5, Invalid: 0, Errors: 0, Skipped: 1
Summary: 6 resources found in 1 file - Valid: 5, Invalid: 0, Errors: 0, Skipped: 1
Summary: 6 resources found in 1 file - Valid: 5, Invalid: 0, Errors: 0, Skipped: 1
```

Note: `Skipped: 1` は CRD (`ExternalSecret`) のスキーマ未提供によるもので、CI では `-ignore-missing-schemas` を指定して既知挙動として扱う。

## 5) ArgoCD env split check

```bash
yq e '.metadata.name' deploy/argocd/application-postal-converter-ja.yaml
yq e '.spec.source.helm.valueFiles[]' deploy/argocd/application-postal-converter-ja.yaml
```

```txt
postal-converter-ja-dev
postal-converter-ja-stg
postal-converter-ja-prod

values-dev.yaml
values-stg.yaml
values-prod.yaml
```

## Result

- `dev/stg/prod` の values 分離が Helm/ArgoCD の両方で反映されることを確認
- Ingress + NetworkPolicy + ExternalSecret の最小雛形が環境 values で有効化できることを確認
- CI で Helm render 後に kubeconform 検証を追加できる構成であることを確認
