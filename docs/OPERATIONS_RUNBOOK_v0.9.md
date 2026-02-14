# Operations Runbook Draft for v0.9

このドキュメントは v0.9 で整備する運用 Runbook の初版ドラフトです。

## 1. SLI / SLO（初期案）

対象サービス: `postal_converter_ja` API

### Availability

- SLI: `/ready` が 2xx を返した割合
- SLO: 月次 99.9% 以上
- Error budget: 43m 12s / 30日

### Latency

- SLI: `GET /postal_codes/*` の p95
- SLO: p95 < 300ms（5分窓）

### Correctness / Error Rate

- SLI: 5xx 比率（`errors_total / requests_total`）
- SLO: 5xx 比率 < 1.0%（5分平均）

### Freshness（バッチ連携）

- SLI: 郵便番号更新ジョブ最終成功からの経過時間
- SLO: 24時間以内に最新成功が1回以上

## 2. 監視ダッシュボード最低要件

- `requests_total`
- `errors_total`
- `not_found_total`
- `average_latency_ms`
- `/ready` の HTTP ステータス
- DB / Redis 接続エラー件数（ログベース）

## 3. 一次対応フロー（15分）

1. 事象分類:
   - 5xx 増加
   - Ready 失敗継続
   - 高レイテンシ
   - データ更新停止
2. 影響範囲の確認:
   - `dev/stg/prod` のどの環境か
   - 単一 Pod か全 Pod か
3. 即時コマンド:

```bash
kubectl -n postal-converter-ja-prod get deploy,po,svc,ingress,externalsecret
kubectl -n postal-converter-ja-prod get events --sort-by=.lastTimestamp | tail -n 30
kubectl -n postal-converter-ja-prod logs deploy/postal-converter-ja --tail=200
curl -fsS https://<prod-host>/ready
```

4. 判定:
   - Secret/接続起因なら `EXTERNAL_SECRETS_RUNBOOK.md` に沿って切り分け
   - リリース起因なら Helm rollback を優先
   - インフラ起因（LB/Ingress/DB 障害）なら platform 担当へ即時エスカレーション

## 4. エスカレーション基準

- 10分以上 `ready` 失敗継続
- 5xx 比率が 5分平均で 3% 超
- 主要機能（郵便番号検索）が実質利用不能

上記のいずれかを満たした場合:

- Incident チャンネルに告知
- On-call / platform 担当に連絡
- 暫定対処（rollback or traffic drain）を先行

## 5. 復旧後の必須アクション

- 障害タイムライン作成（検知時刻、一次対応、復旧時刻）
- 再発防止タスクを issue 化
- SLO 逸脱時は error budget 消費を記録
- 必要なら SLI 定義・アラート閾値を更新

## 6. v0.9 で確定させる項目

- SLI 計測基盤（Prometheus/Grafana か他基盤か）
- オンコール体制（担当ローテーション、連絡経路）
- Runbook の自動化（診断コマンド集約、復旧手順の半自動化）
