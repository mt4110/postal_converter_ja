# Runbook: 更新失敗対応 v0.9.0 (RNB-01)

最終更新: 2026-02-14 (JST)

## 1. 目的

郵便番号データ更新ジョブが失敗した際に、運用担当が手順だけで
原因切り分けと復旧判断まで到達できる状態を作る。

## 2. 対象インシデント

- 更新ジョブが失敗し、最新データ反映が止まっている
- 更新ジョブは成功したが、APIの結果が古い/不整合
- 更新処理後に API 健康状態が悪化した

## 3. 事前情報（開始前に記録）

- 検知時刻
- 対象環境 (`dev/stg/prod`)
- 直近変更（デプロイ、Secret変更、DB変更）
- 影響範囲（API全体 / 一部機能）

## 4. 一次対応フロー（15分）

1. 稼働状態確認

```bash
kubectl -n postal-converter-ja-prod get deploy,po,job
kubectl -n postal-converter-ja-prod get events --sort-by=.lastTimestamp | tail -n 50
curl -fsS https://<prod-host>/ready
```

2. 更新ジョブの結果確認

```bash
kubectl -n postal-converter-ja-prod get jobs
kubectl -n postal-converter-ja-prod logs job/<update-job-name> --tail=300
```

3. APIエラー傾向確認

```bash
kubectl -n postal-converter-ja-prod logs deploy/postal-converter-ja --tail=300
```

4. 判定

- Secret/接続異常: `docs/EXTERNAL_SECRETS_RUNBOOK.md` へ分岐
- 更新ジョブ単体失敗: 本書 5章へ進む
- 更新後の挙動不良: 本書 6章へ進む

## 5. 更新ジョブ失敗の切り分け

### 5.1 データ取得失敗（ネットワーク/取得先）

兆候:

- job log に timeout / download failure
- 再実行で成功する可能性あり

対応:

1. 一時障害かを確認（同時刻の外形監視、ネットワーク障害情報）
2. 1回のみ再実行
3. 再失敗時はデータ取得元障害としてエスカレーション

### 5.2 DB書き込み失敗

兆候:

- migration error / transaction rollback / unique conflict
- API側にもDB接続エラーが増える

対応:

1. 接続情報とSecret同期状態を確認
2. DB容量/ロック状況を確認
3. 破壊的再実行はせず、原因確定まで停止

### 5.3 変換処理失敗（フォーマット不整合）

兆候:

- parse error / schema mismatch
- 特定レコードで失敗

対応:

1. 失敗入力サンプルを保全
2. 変換ロジック異常としてアプリ担当へ引き継ぎ
3. 直近正常版データを維持し、反映は見送り

## 6. 更新後の挙動不良（成功扱いだが異常）

### 6.1 API結果が古い

確認:

- 更新ジョブ成功時刻
- キャッシュ有効状態
- `not_found_total` の急増有無

対応:

1. キャッシュ障害疑いなら `docs/RUNBOOK_CACHE_INCIDENT_v0_9_0.md` へ分岐
2. 必要時にキャッシュ無効化または再起動
3. `/ready` と検索APIで復旧確認

### 6.2 検索結果が不自然（急減/急増）

対応:

1. 直近更新との差分確認
2. 異常が大きい場合は `docs/RUNBOOK_DB_ROLLBACK_v0_9_0.md` を参照して前回安定版へのロールバック判断
3. 影響が軽微なら監視強化で継続観察

## 7. エスカレーション基準

以下のいずれかで P1 扱い:

- `ready` 失敗が10分継続
- 5分平均5xxが3%以上
- 更新停止により主要機能が実質利用不能

上記以外でも、更新停止が12時間超見込みなら P2 でエスカレーション。

## 8. 復旧判定

復旧完了は以下を満たすこと:

- 更新ジョブが成功し、エラーなく終了
- `/ready` が安定して 2xx
- 代表クエリで検索結果が正常
- 監視値（5xx / latency）が平常範囲

## 9. 事後対応

- 障害タイムライン作成（検知/対応/復旧）
- 根本原因と再発防止策を `v0.9.xxx` Issue 化
- `docs/SLO_SLI_v0_9_0.md` と `docs/ALERT_POLICY_v0_9_0.md` に必要なら反映

## 10. 人間テスト観点（QA-01連携）

- 初見担当者が 30分以内に「再実行可否」を判断できるか
- 参照Runbookの分岐先を迷わず選択できるか
- 復旧判定条件を証跡付きで満たせるか
