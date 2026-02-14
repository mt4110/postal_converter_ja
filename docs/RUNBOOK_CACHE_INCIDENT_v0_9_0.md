# Runbook: キャッシュ障害対応 v0.9.0 (RNB-02)

最終更新: 2026-02-14 (JST)

## 1. 目的

Redis キャッシュ起因の障害（stale データ、miss 暴走、接続不良）に対して、
運用担当が短時間で切り分けと緩和を実施できるようにする。

## 2. 対象インシデント

- 更新後も古い検索結果が返る（stale）
- キャッシュヒット率が急落し、DB負荷が急増
- Redis 接続エラーが継続し API のレイテンシ/5xx が増加

## 3. 事前情報（開始時に記録）

- 検知時刻
- 対象環境 (`dev/stg/prod`)
- 直近変更（デプロイ、更新ジョブ、Secret更新）
- 影響範囲（検索API全体 / 一部）

## 4. 一次対応フロー（15分）

1. API 健康状態とエラー傾向確認

```bash
curl -fsS https://<prod-host>/ready
kubectl -n postal-converter-ja-prod logs deploy/postal-converter-ja --tail=300
```

2. Redis 周辺の状態確認

```bash
kubectl -n postal-converter-ja-prod get pods,svc | grep -i redis
kubectl -n postal-converter-ja-prod logs deploy/postal-converter-ja --tail=300 | grep -i -E "redis|cache|timeout|connection"
```

3. 判定

- Redis 接続失敗が主因: 本書 5章
- stale データが主因: 本書 6章
- miss 暴走（DB負荷過多）が主因: 本書 7章
- Secret/認証起因: `docs/EXTERNAL_SECRETS_RUNBOOK.md` へ分岐

## 5. Redis 接続失敗対応

兆候:

- `connection refused` / `timeout` / `auth failure`
- APIログに Redis エラーが連続

対応:

1. Redis Pod/Service の生存確認
2. 接続先設定（`REDIS_URL`）と Secret 同期状態を確認
3. 一時復旧として API 再起動を1回実施

```bash
kubectl -n postal-converter-ja-prod rollout restart deploy/postal-converter-ja
kubectl -n postal-converter-ja-prod rollout status deploy/postal-converter-ja
```

4. 復旧しない場合は Redis/基盤担当へエスカレーション

## 6. stale データ対応（更新済みなのに古い）

兆候:

- 更新ジョブ成功後も検索結果が更新前のまま
- `not_found_total` や問い合わせ件数が更新直後から増加

対応:

1. 更新ジョブ成功時刻とキャッシュ更新時刻を確認
2. キャッシュ失効処理の実行有無を確認
3. 必要時にキャッシュクリアまたは API 再起動で再読込
4. 代表クエリで差分確認（更新前後）

## 7. miss 暴走対応（DB負荷増大）

兆候:

- ヒット率低下と同時に DB レイテンシ上昇
- API p95 悪化、5xx 増加

対応:

1. Redis 接続健全性を再確認
2. キャッシュキー設計/TTL 変更の有無を確認
3. 直近リリース差分が原因ならロールバックを検討
4. 一時的にスロットリングやトラフィック調整を実施

## 8. エスカレーション基準

以下のいずれかで P1:

- `ready` 失敗が10分継続
- 5分平均5xxが3%以上
- キャッシュ障害が主因で主要検索が利用不能

以下は P2:

- p95 300ms超が15分継続
- miss 暴走で DB 負荷が高止まり

## 9. 復旧判定

復旧完了は以下を満たすこと:

- `/ready` が安定して 2xx
- Redis エラーが収束
- p95/5xx が平常レンジへ復帰
- 代表クエリで最新データ整合を確認

## 10. 事後対応

- 障害タイムライン作成（検知/対応/復旧）
- 根本原因を `v0.9.xxx` Issue 化
- 必要に応じて `docs/DASHBOARD_STANDARD_v0_9_0.md` の可視化項目を更新

## 11. 人間テスト観点（QA-01連携）

- 初見担当者が 25分以内に「stale / 接続失敗 / miss 暴走」を区別できるか
- 緩和策選定（再起動、失効、エスカレーション）を迷わず実行できるか
- 復旧判定を証跡付きで完了できるか
