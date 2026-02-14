# Handoff for v0.8.3

## 現在地点（v0.8.2）

- Helm デフォルトの Kubernetes 雛形を追加済み
- Kustomize base を追加済み
- ArgoCD Application 雛形を追加済み
- 既存運用向け導入設計図を追加済み（`K8S_ADOPTION_BLUEPRINT.md`）

## v0.8.3 で優先する TODO

1. Helm lint / template を CI に追加（`deploy/helm/postal-converter-ja`）
2. ArgoCD `Application` の環境別化（dev/stg/prod の分離）
3. Secret の平文排除（External Secrets 連携の雛形）
4. Ingress と NetworkPolicy の最小テンプレート追加
5. 導入リハーサル証跡（同一端末で `deploy -> health/ready`）を docs 化

## マイルストーン整合

- v0.8.x:
  - デプロイ基盤仕上げ（AWS先行 + Kubernetes最小運用ルート）
- v0.9.0:
  - マルチクラウド再拡張（GCP/Azure）と運用強化
- v1.0.0:
  - 商用導入標準化（運用Runbook、監査、サポートフロー固定）

## 次スレ冒頭メッセージ（コピペ用）

```text
feature/v0.8.3 で開始します。
v0.8.2 は Helm/Kustomize/ArgoCD の最小雛形と導入設計図まで完了しています。
次は v0.8.3 の1タスク目として「Helm lint/template を CI に追加」から進めてください。
完了後は 1コミット単位で差分確認し、push/PR まで進めます。
```
