# Dashboard Standard v0.9.0 (OPS-02)

最終更新: 2026-02-14 (JST)

## 1. 目的

運用者が 1 画面で SLO 逸脱リスクを判断できる最小ダッシュボードを定義する。

## 2. 対象

- 環境: `prod`（必須）、`stg`（任意）
- 期間: `Last 15m / 1h / 24h` を切替可能にする

## 3. 必須パネル

| No | パネル名 | 指標 | 判定基準 |
| --- | --- | --- | --- |
| 1 | Request Throughput | `requests_total` | 急減/急増の異常検知 |
| 2 | 5xx Error Rate | `errors_total / requests_total` | 1.0%以上で注意、3.0%以上で重大 |
| 3 | p95 Latency | `GET /postal_codes/*` の p95 | 300ms超で注意 |
| 4 | Ready Status | `/ready` の 2xx 比率 | 連続失敗で重大 |
| 5 | Not Found Ratio | `not_found_total / requests_total` | データ品質/検索傾向の変化確認 |
| 6 | Update Freshness | 更新ジョブ最終成功からの経過時間 | 12h超で警戒、24h超で重大 |
| 7 | DB Error Count | DB接続失敗件数 | 継続増加で調査 |
| 8 | Redis Error Count | Redis接続失敗件数 | 継続増加で調査 |

## 4. 画面レイアウト

- 1段目（即時判定）: `Ready`, `5xx Error Rate`, `p95 Latency`
- 2段目（負荷/傾向）: `Request Throughput`, `Not Found Ratio`
- 3段目（依存系）: `DB Error Count`, `Redis Error Count`, `Update Freshness`

## 5. 閾値整合

閾値は以下と一致させる:

- `docs/SLO_SLI_v0_9_0.md`
- `docs/ALERT_POLICY_v0_9_0.md`

例:

- P2: 5分平均 5xx が 1%以上
- P1: 5分平均 5xx が 3%以上
- P2: p95 が 300ms 以上を 15 分継続

## 6. JSON 管理ルール

- ダッシュボードJSONは環境依存値を最小化する。
- 変数化できる項目（namespace, service名）はテンプレート化する。
- 閾値変更時は `v0.9.xxx` Issue に理由と差分を記録する。

## 7. 受け入れ条件

- 上記 8 パネルが 1 画面で視認可能である。
- `Last 15m` で一次対応に必要な判定ができる。
- `docs/OPERATIONS_RUNBOOK_v0.9.md` の一次対応フローと矛盾しない。
