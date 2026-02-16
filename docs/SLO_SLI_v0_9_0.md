# SLO/SLI 定義 v0.9.0 (OPS-01)

最終更新: 2026-02-14 (JST)

## 1. 対象

- 対象サービス: `postal_converter_ja` API
- 主対象環境: `prod`（`stg` は検証用）
- 評価単位: 月次（暦月）

## 2. SLI / SLO 定義

| 項目 | SLI (計測定義) | SLO (目標) | 計測窓 |
| --- | --- | --- | --- |
| Availability | `/ready` が `2xx` を返した割合 | 99.9%以上 | 月次 |
| Latency | `GET /postal_codes/*` の p95 | 300ms 未満 | 5分 |
| Server Error Rate | `5xx / 全リクエスト` | 1.0% 未満 | 5分平均 |
| Freshness | 郵便番号更新ジョブの最終成功からの経過時間 | 24時間以内に1回以上成功 | 常時 |

## 3. Error Budget

Availability SLO 99.9% の許容停止時間:

- 30日: 43分12秒
- 31日: 44分38秒
- 28日: 40分19秒

運用ルール:

- 1か月の Error Budget 消費率が 50% を超えたら変更凍結候補を検討。
- 80% を超えたら新規リスク変更は原則停止し、安定化タスクを優先。

## 4. データソース

- HTTP メトリクス: `requests_total`, `errors_total`, `average_latency_ms`
- 稼働判定: `/ready` の HTTP ステータス
- 鮮度判定: 更新ジョブ最終成功時刻（ログ/メトリクス）
- 補助指標: DB/Redis 接続エラー

## 5. 逸脱判定

以下いずれかで SLO 逸脱リスクあり:

- 5分平均 5xx 比率が 1.0%以上
- 5分窓 p95 が 300ms 以上
- `/ready` 失敗が連続観測
- 更新ジョブ成功が 24時間を超えて途絶

## 6. レビュー運用

- 日次: 指標の異常有無を確認
- 週次: 閾値の妥当性（誤検知/見逃し）を確認
- 月次: Error Budget 消費を集計し、改善を `v0.9.xxx` Issue 化
