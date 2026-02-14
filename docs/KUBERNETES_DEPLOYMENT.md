# Kubernetes Deployment Guide (v0.8.3)

このドキュメントは、Postal Converter JA の Kubernetes 導入ルートを定義します。

## 方針（v0.8.3）

- デフォルト導入: Helm (`deploy/helm/postal-converter-ja`)
- 補助ルート: Kustomize base (`deploy/k8s/base`)
- GitOps運用: ArgoCD Application (`deploy/argocd/application-postal-converter-ja.yaml`)
- 環境分離: `dev/stg/prod` の 3 namespace + 3 values ファイル

## 1) Helm（デフォルト）

前提:

- `kubectl` が利用可能
- `helm` が利用可能

1) 構文チェック:

```bash
helm lint deploy/helm/postal-converter-ja
```

2) 環境別デプロイ例（dev）:

```bash
helm upgrade --install postal-converter-ja deploy/helm/postal-converter-ja \
  --namespace postal-converter-ja-dev \
  --create-namespace \
  -f deploy/helm/postal-converter-ja/values-dev.yaml
```

疎通確認:

```bash
kubectl -n postal-converter-ja-dev get pods,svc,ingress,networkpolicy,externalsecret
kubectl -n postal-converter-ja-dev port-forward svc/postal-converter-ja 3202:3202
curl -fsS http://127.0.0.1:3202/health
curl -fsS http://127.0.0.1:3202/ready
```

## 2) Kustomize base（最小雛形）

```bash
kubectl apply -k deploy/k8s/base
kubectl -n postal-converter-ja get pods,svc
```

## 3) ArgoCD（GitOps運用ルート）

前提:

- ArgoCD がクラスタにインストール済み

```bash
kubectl apply -f deploy/argocd/application-postal-converter-ja.yaml
```

ArgoCD CLI を使う場合:

```bash
argocd app get postal-converter-ja-dev
argocd app get postal-converter-ja-stg
argocd app get postal-converter-ja-prod
argocd app sync postal-converter-ja-dev
argocd app sync postal-converter-ja-stg
argocd app sync postal-converter-ja-prod
argocd app wait postal-converter-ja-dev --health --operation
argocd app wait postal-converter-ja-stg --health --operation
argocd app wait postal-converter-ja-prod --health --operation
```

## 運用メモ

- `values.yaml` は共通デフォルトです。環境差分は `values-dev.yaml` / `values-stg.yaml` / `values-prod.yaml` で管理してください。
- `secret.create=false` を維持し、接続情報は External Secrets 側で管理してください。
- `ingress.enabled` / `networkPolicy.enabled` は環境 values で明示的に有効化してください。
- `image.repository` / `image.tag` は CI 生成イメージに合わせて固定してください。
