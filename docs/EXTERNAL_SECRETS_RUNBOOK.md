# External Secrets Minimal Runbook (v0.8.4)

この Runbook は、`postal_converter_ja` の Kubernetes 運用で External Secrets を使う最小手順を定義します。

## 1. 前提

- External Secrets Operator が導入済み。
- 環境ごとに `ClusterSecretStore` が存在する:
  - `postal-converter-ja-dev`
  - `postal-converter-ja-stg`
  - `postal-converter-ja-prod`
- Helm values は `secret.create=false` を維持する。
- Helm values は `externalSecret.enabled=true` を有効にする。

## 2. 切替手順（平文 Secret から ExternalSecret へ）

以下は `prod` 例。`dev/stg` では namespace と values を読み替える。

1. 既存 Secret を退避:

```bash
kubectl -n postal-converter-ja-prod get secret postal-converter-ja-secret -o yaml > /tmp/postal-converter-ja-secret.backup.yaml
```

2. SecretStore の疎通を確認:

```bash
kubectl get clustersecretstore postal-converter-ja-prod
kubectl -n external-secrets get pods
```

3. Helm values の参照先キーを確認:

```bash
grep -n "externalSecret:" deploy/helm/postal-converter-ja/values-prod.yaml
```

4. Helm apply（ExternalSecret 有効）:

```bash
helm upgrade --install postal-converter-ja deploy/helm/postal-converter-ja \
  -n postal-converter-ja-prod \
  --create-namespace \
  -f deploy/helm/postal-converter-ja/values.yaml \
  -f deploy/helm/postal-converter-ja/values-prod.yaml
```

5. 同期確認:

```bash
kubectl -n postal-converter-ja-prod get externalsecret
kubectl -n postal-converter-ja-prod describe externalsecret postal-converter-ja-external-secret
kubectl -n postal-converter-ja-prod get secret postal-converter-ja-secret -o jsonpath='{.data}' | jq
```

6. アプリ確認:

```bash
kubectl -n postal-converter-ja-prod rollout status deploy/postal-converter-ja
kubectl -n postal-converter-ja-prod port-forward svc/postal-converter-ja 3202:3202
curl -fsS http://127.0.0.1:3202/ready
```

## 3. 障害時の一次確認

### Case A: ExternalSecret が `Ready=False`

確認:

```bash
kubectl -n postal-converter-ja-prod describe externalsecret postal-converter-ja-external-secret
kubectl -n external-secrets logs deploy/external-secrets -f --tail=200
```

見るべき点:

- `ClusterSecretStore not found`
- remote key / property の typo
- 認証エラー（cloud IAM / service account）

### Case B: `postal-converter-ja-secret` が未作成 or キー不足

確認:

```bash
kubectl -n postal-converter-ja-prod get secret postal-converter-ja-secret -o yaml
```

見るべき点:

- `POSTGRES_DATABASE_URL`
- `MYSQL_DATABASE_URL`
- `SQLITE_DATABASE_PATH`
- `REDIS_URL`

### Case C: API が Ready にならない

確認:

```bash
kubectl -n postal-converter-ja-prod logs deploy/postal-converter-ja --tail=200
kubectl -n postal-converter-ja-prod get events --sort-by=.lastTimestamp | tail -n 30
```

見るべき点:

- DB URL 形式不正
- Redis 到達不可（`READY_REQUIRE_CACHE=true` の場合）
- Secret 更新後に Pod が古い環境変数を保持しているケース

必要に応じて再起動:

```bash
kubectl -n postal-converter-ja-prod rollout restart deploy/postal-converter-ja
kubectl -n postal-converter-ja-prod rollout status deploy/postal-converter-ja
```

## 4. 緊急ロールバック

1. 直前リリースへ戻す:

```bash
helm -n postal-converter-ja-prod history postal-converter-ja
helm -n postal-converter-ja-prod rollback postal-converter-ja <REVISION>
```

2. Secret が壊れている場合は退避済み Secret を復元:

```bash
kubectl apply -f /tmp/postal-converter-ja-secret.backup.yaml
kubectl -n postal-converter-ja-prod rollout restart deploy/postal-converter-ja
```

3. 復旧後、`values-*.yaml` と SecretManager 側の key/property を見直して再実施する。
