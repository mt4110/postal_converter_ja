# Kubernetes Deployment Guide (v0.8.2)

このドキュメントは、Postal Converter JA の Kubernetes 導入ルートを定義します。

## 方針

- デフォルト導入: Helm (`deploy/helm/postal-converter-ja`)
- 補助ルート: Kustomize base (`deploy/k8s/base`)
- GitOps運用: ArgoCD Application (`deploy/argocd/application-postal-converter-ja.yaml`)

## 1) Helm（デフォルト）

前提:

- `kubectl` が利用可能
- `helm` が利用可能

```bash
helm lint deploy/helm/postal-converter-ja

helm upgrade --install postal-converter-ja deploy/helm/postal-converter-ja \
  --namespace postal-converter-ja \
  --create-namespace
```

疎通確認:

```bash
kubectl -n postal-converter-ja get pods,svc
kubectl -n postal-converter-ja port-forward svc/postal-converter-ja 3202:3202
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
argocd app get postal-converter-ja
argocd app sync postal-converter-ja
argocd app wait postal-converter-ja --health --operation
```

## 運用メモ

- `values.yaml` の接続情報はサンプル値です。実運用では Secret 管理基盤（External Secrets, SOPS など）へ移行してください。
- `image.repository` / `image.tag` はCI生成イメージに合わせて固定してください。
