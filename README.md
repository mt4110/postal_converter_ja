# 郵便番号自動最新化システム (Postal Converter JA)

![Version](https://img.shields.io/badge/version-0.8.0-blue.svg)
![Status](https://img.shields.io/badge/status-beta-orange.svg)

English README: [ENGLISH_README.md](./docs/ENGLISH_README.md)

このプロジェクトは、日本郵便のデータを自動的に取得・更新し、常に最新の郵便番号データを提供するシステムです。
Rust 製のバックエンド（Crawler + API）と、Next.js 製のフロントエンドで構成されています。

## 特徴

- **自動更新**: Crawler が定期的に（デフォルト 24 時間）日本郵便の CSV を取得し、データベースを更新します。
- **差分更新**: 廃止された郵便番号の自動削除や、変更があったデータの更新を効率的に行います。
- **高速な API**: Rust (Axum)製の API サーバーが、郵便番号検索や住所検索を提供します。
- **モダンなフロントエンド**: Next.js (React) + TypeScript + Tailwind CSS によるサンプル実装が含まれています。
- **ハイブリッド環境**: Nix による再現性の高い開発環境と、Docker による手軽な DB 構築を組み合わせています。
- **DB 切り替え対応**: MySQL と PostgreSQL の両方に対応。環境変数で切り替え可能です。

## アーキテクチャ

- **Frontend**: Next.js (React), TypeScript, Tailwind CSS, Radix UI
- **Backend API**: Rust (Axum), tokio-postgres, mysql_async
- **Crawler**: Rust, Tokio, Reqwest, CSV
- **Database**: MySQL & PostgreSQL (両対応、環境変数で選択)
- **Infrastructure**: Docker Compose (DB), Nix (Rust/Node environment)

## 前提条件

以下のツールが必要です：

- **Nix**: 開発環境の構築に使用します（Rust ツールチェーン、ビルドツールなど）
- **Docker**: データベースの実行に使用します
- **補足**: Node.js / Yarn / Go / Rust は `nix develop` で提供されます（Nix 前提）

### Nix のインストール

まだ Nix をインストールしていない場合は、以下のコマンドでインストールしてください：

```bash
# 公式インストーラー
sh <(curl -L https://nixos.org/nix/install)

# または Determinate Systems インストーラー（推奨）
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

---

## 🚀 クイックスタート (Launcher)

Go 製の CLI ランチャーを使って、簡単に環境を起動できます。

```bash
cd launcher
nix develop --command go run main.go
```

または、ビルドして実行：

```bash
cd launcher
nix develop --command go build -o postal-launcher
./postal-launcher
```

ランチャーから以下の操作が可能です：

- データベースの起動/停止 (Docker)
- Crawler, API, Frontend の一括起動 (新しいターミナルで開きます)

## 📹 クイックスタート (Launcher) demo映像

<!-- markdownlint-disable MD033 -->
<video src="https://github.com/mt4110/postal_converter_ja/releases/download/v0.9.0/2026-02-26_11-33-48.mp4" controls width="800"></video>
<!-- markdownlint-enable MD033 -->

---

## 🛠 手動セットアップ & 実行

### 一括導入 (onboard.sh)

プロファイル付きでローカル導入を一括実行できます。

```bash
./scripts/onboard.sh --profile dev
```

詳細は `docs/ONBOARDING_PROFILES.md` を参照してください。

### 環境セットアップ自動化 (Nix + Docker)

Nix / Docker 前提のセットアップを自動化する場合:

```bash
./scripts/setup_nix_docker.sh --profile dev
```

詳細は `docs/SETUP_NIX_DOCKER_AUTOMATION.md` を参照してください。

### Docker イメージビルド (マルチステージ)

API / Crawler はマルチステージDockerfileで軽量ランタイムイメージを生成できます。

```bash
# API
docker build -f worker/api/Dockerfile -t postal-api:multistage .

# Crawler
docker build -f worker/crawler/Dockerfile -t postal-crawler:multistage .
```

実行例:

```bash
docker run --rm -p 3202:3202 \
  -e DATABASE_TYPE=postgres \
  -e POSTGRES_DATABASE_URL=postgres://postgres:postgres_password@host.docker.internal:3205/zip_code_db \
  postal-api:multistage
```

## セットアップと実行

> **重要**: 以下の手順を**必ず順番通り**に実行してください。

### 1. データベースの起動

**最初に**Docker コンテナを起動してデータベースを初期化します：

```bash
# プロジェクトルートで実行
docker-compose up -d

# DBコンテナが起動したことを確認
docker ps
```

MySQL（ポート 3204）と PostgreSQL（ポート 3205）の両方が起動します。

デフォルトは **named volume**（推奨）です。  
ホストのディレクトリを直接マウントしたい場合のみ、`docker-compose.local.yml` を重ねます:

```bash
docker compose -f docker-compose.yml -f docker-compose.local.yml up -d
```

`binlog.index Permission denied` などでローカルDBが壊れた場合は、まず初期化してください:

```bash
docker compose down -v
```

Redis キャッシュを使う場合は、以下で Redis も起動できます（オプション）:

```bash
docker compose --profile cache up -d redis
```

### 2. 環境変数の設定

Crawler と API の `.env` ファイルを作成します：

```bash
# Crawler用
cp worker/crawler/.env.example worker/crawler/.env

# API用
cp worker/api/.env.example worker/api/.env
```

`.env` ファイルで `DATABASE_TYPE` を設定します（デフォルト: `postgres`）：

```bash
# PostgreSQLを使用する場合（デフォルト）
DATABASE_TYPE=postgres

# MySQLを使用する場合
DATABASE_TYPE=mysql

# SQLiteを使用する場合（read-only PoC）
DATABASE_TYPE=sqlite
SQLITE_DATABASE_PATH=storage/sqlite/postal_codes.sqlite3

# Redis キャッシュ（オプション）
REDIS_URL=redis://127.0.0.1:3206
REDIS_CACHE_TTL_SECONDS=300

# Readiness 厳密化（オプション）
# true: REDIS_URL が設定されている時、Redis疎通失敗で /ready=503
# false: Redis疎通失敗でも /ready=200（cache="error"）
READY_REQUIRE_CACHE=false

# IP制限（オプション）
# TRUST_PROXY_HEADERS=true の場合、X-Forwarded-For / X-Real-IP を優先して判定
TRUST_PROXY_HEADERS=false
# 例: IP_ALLOWLIST=203.0.113.10,10.0.0.0/24,2001:db8::/64
IP_ALLOWLIST=

# SSOヘッダ認証（最小構成）
# none: 認証なし（デフォルト）
# sso_header: IdP連携済みリバースプロキシが付与するヘッダを必須化
AUTH_MODE=none
AUTH_USER_HEADER=x-auth-request-email
# 任意（未設定可）
AUTH_GROUPS_HEADER=
# 認証をスキップするパス（prefix判定、カンマ区切り）
AUTH_ANONYMOUS_PATHS=/health,/ready,/openapi.json,/docs
```

> [!NOTE]
> `DATABASE_TYPE=sqlite` は API の read-only PoC 向けです。Crawler から SQLite への直接更新は未対応です。

### SQLite DB 生成（PoC）

PostgreSQL に取り込まれたデータから SQLite DB を生成できます:

```bash
nix develop --command bash -lc "./scripts/build_sqlite_from_postgres.sh"
```

SQLite 配布アーティファクト（DB + checksum + manifest）を作る場合:

```bash
nix develop --command bash -lc "./scripts/package_sqlite_release.sh"
```

`artifacts/sqlite/` に以下が生成されます。

- `postal_codes-YYYYMMDD.sqlite3`
- `checksums-YYYYMMDD.txt`
- `manifest-YYYYMMDD.txt`

### 3. Crawler の実行（郵便番号データの自動取得・更新）

**Nix 環境に入ってから**Crawler を起動します：

```bash
nix develop --command bash -lc "cd worker/crawler && cargo run --release --bin crawler"
```

1サイクルだけ実行して終了したい場合（CI/バッチ向け）:

```bash
nix develop --command bash -lc "cd worker/crawler && CRAWLER_RUN_ONCE=true cargo run --release --bin crawler"
```

初回実行時は以下の処理が行われます：

- 日本郵便から CSV データをダウンロード
- データベースへの初期データ投入（約 12 万件）

その後、設定された間隔（デフォルト 24 時間）で自動的にデータを更新し続けます。
`REDIS_URL` が設定されている場合、更新後に Redis キャッシュを自動失効します。

### 版指定ロールバック（最小CLI）

Crawler が保存した `data_version` を指定して、`postal_codes` をスナップショットから復元できます。

```bash
nix develop --command bash -lc "cd worker/crawler && cargo run --release --bin rollback -- --database-type postgres --data-version v20260213002038361"
```

MySQL の場合:

```bash
nix develop --command bash -lc "cd worker/crawler && cargo run --release --bin rollback -- --database-type mysql --data-version v20260213002038361"
```

`data_version` は `data_update_audits` テーブルで確認できます。

### 4. API サーバーの起動

**別のターミナルで**、Nix 環境に入ってから API を起動します：

```bash
nix develop --command bash -lc "cd worker/api && cargo run --release --bin api"
```

API サーバーは `http://localhost:3202` で起動します。

### 5. フロントエンドの起動

さらに**別のターミナルで**、Nix 環境経由でフロントエンドを起動します：

```bash
nix develop --command bash -lc "cd frontend && yarn install && yarn dev"
```

ブラウザで `http://localhost:3203` にアクセスすると、以下の導入サンプルを切り替えて確認できます。

- EC 配送先自動補完フォーム
- 会員登録フォーム（郵便番号検索 + 住所キーワード検索）
- コールセンター入力支援フォーム（通話中の候補提示）

SDK 実装サンプルは `frontend/src/lib/postal-sdk.ts` を参照してください。

## トラブルシューティング

👉 **トラブルシューティングについてはこちら:** [TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md)

## 開発者向け情報

👉 **API ドキュメント（OpenAPI JSON）:** `http://localhost:3202/openapi.json`

👉 **Swagger UI:** `http://localhost:3202/docs`

👉 **Readiness:** `http://localhost:3202/ready`

`/ready` の判定方針:

- `READY_REQUIRE_CACHE=false`（デフォルト）: DB 接続が正常なら Ready。Redis 障害時は `cache="error"` を返す
- `READY_REQUIRE_CACHE=true`: `REDIS_URL` が設定されている場合、Redis 障害時は `503`（`{"error":"cache not ready"}`）

IP制限（`IP_ALLOWLIST`）:

- 未設定: IP 制限なし
- 設定あり: 許可IP/CIDR 以外は `403 {"error":"forbidden"}`
- `TRUST_PROXY_HEADERS=true` の場合、`X-Forwarded-For` / `X-Real-IP` を優先して判定（Cloud Run 想定）

SSOヘッダ認証（`AUTH_MODE=sso_header`）:

- `AUTH_USER_HEADER` が存在しない場合は `401 {"error":"unauthorized"}`
- 最小構成は「SAML IdP -> 認証プロキシ（oauth2-proxy など）-> API」
- `/health` `/ready` `/openapi.json` `/docs` は既定で匿名アクセスを許可
- 匿名許可パスは `AUTH_ANONYMOUS_PATHS` で調整可能（prefix判定）

👉 **Metrics(JSON):** `http://localhost:3202/metrics`

### 参考ドキュメント

- 仕様書（補助）: [API_SPEC.md](./API_SPEC.md)
- SSO最小構成設計: [SAML_SSO_MINIMAL_DESIGN.md](./docs/SAML_SSO_MINIMAL_DESIGN.md)
- 開発者向け情報: [DEVELOPMENT.md](./docs/DEVELOPMENT.md)
- CI/CD 設計: [CI_DESIGN.md](./docs/CI_DESIGN.md)

### デプロイ / Terraform / Kubernetes

- デプロイ骨格（GitHub Actions + Terraform）: [DEPLOY.md](./docs/DEPLOY.md)
- Kubernetes 導入ガイド（v0.8.3）: [KUBERNETES_DEPLOYMENT.md](./docs/KUBERNETES_DEPLOYMENT.md)
- Kubernetes 導入設計図（既存運用向け）: [K8S_ADOPTION_BLUEPRINT.md](./docs/K8S_ADOPTION_BLUEPRINT.md)
- Kubernetes 最小雛形:
  - `deploy/helm/postal-converter-ja`（Helm / デフォルト）
  - `deploy/k8s/base`（Kustomize）
  - `deploy/argocd`（ArgoCD route）
- Kubernetes 導入リハーサル証跡（v0.8.3）: [K8S_REHEARSAL_EVIDENCE_v0.8.3.md](./docs/K8S_REHEARSAL_EVIDENCE_v0.8.3.md)
- ExternalSecret 運用の最小 Runbook: [EXTERNAL_SECRETS_RUNBOOK.md](./docs/EXTERNAL_SECRETS_RUNBOOK.md)
- v0.9 運用 Runbook ドラフト（SLO/SLI・一次対応）: [OPERATIONS_RUNBOOK_v0.9.md](./docs/OPERATIONS_RUNBOOK_v0.9.md)
- v0.8 offline plan 証跡: [TERRAFORM_OFFLINE_PLAN_EVIDENCE.md](./docs/TERRAFORM_OFFLINE_PLAN_EVIDENCE.md)
- v0.8 rollback 証跡: [TERRAFORM_ROLLBACK_REHEARSAL_EVIDENCE.md](./docs/TERRAFORM_ROLLBACK_REHEARSAL_EVIDENCE.md)

Terraform 関連の実行コマンド:

```bash
./scripts/setup_github_oidc_vars.sh
./scripts/run_terraform_workflow.sh
./scripts/run_terraform_workflow.sh --action destroy --environment dev --confirm-destroy DESTROY_AWS --ref v0.9.0
nix develop --command terraform version
```

`terraform` がローカルシェルで見えない場合は Nix dev shell 経由で実行してください（Nix では OpenTofu 互換の `terraform` コマンドを提供）。  
Homebrew 経由では `terraform` が 1.5.7 固定になることがあるため、バージョン差異を避ける目的でも Nix を推奨します。

Terraform/OpenTofu バージョン方針（v0.9）:

- 最小要件: `>= 1.6.0`（`infra/terraform/platforms/aws/main.tf`）
- CI 固定: `1.11.1`（`.github/workflows/terraform-multiplatform.yml`）
- ローカル推奨: `1.11+`（Nix dev shell 上で `terraform version` を確認）

```bash
nix develop --command terraform fmt -check -recursive infra/terraform
nix develop --command terraform -chdir=infra/terraform/platforms/aws validate
```

CI でも同等チェック（fmt/validate）を実行します。

👉 **SQLite read-only PoC についてはこちら:** [SQLITE_READONLY_POC.md](./docs/SQLITE_READONLY_POC.md)

👉 **SQLite 配布ワークフロー（GitHub Actions 手動実行）:** `.github/workflows/sqlite-release.yml`

## ライセンスと商用利用について

本プロジェクトは、**デュアルライセンス**（Dual Licensing）を採用する予定です。

1. **個人利用・非営利・オープンソース開発**:

    - **MIT License** の下、自由に利用・改変・再配布が可能です。
    - 学習目的や個人プロジェクトでぜひご活用ください。

2. **法人利用・商用サービスへの組み込み**:
    - 企業での業務利用や、商用製品への組み込みを行う場合は、**商用ライセンス**の契約、または**GitHub Sponsors**等による継続的な支援をお願いすることを想定しています。
    - （現在はプレビュー版のため、評価目的での利用は無償です。本格導入の際はご連絡ください）

このモデルにより、オープンソースとしての発展と、持続可能な開発体制の両立を目指しています。

## ロードマップ

実行順序とマイルストーンは `docs/V0_7_TO_V1_EXECUTION_PLAN.md` を参照してください。

- [x] **CI/CD パイプラインの構築**: GitHub Actions による自動テスト・ビルド
- [x] **ランチャーの UX 改善**: 実行順序の制御と視覚的フィードバック
- [x] **環境構築の自動化 (v0.6)**: `scripts/setup_nix_docker.sh` + `scripts/onboard.sh` で導入を標準化
- [ ] **マルチプラットフォーム デプロイ基盤**: GitHub Actions + Terraform による環境展開（クラウド別ターゲット対応）
- [x] **MySQL/PostgreSQL の自動テスト**: 両 DB でのインテグレーションテスト追加
- [x] **Docker イメージの軽量化**: マルチステージビルドの最適化（API/Crawler）
- [ ] **Kubernetes 連携**: コンテナ連携・オーケストレーション対応（Helm/Kustomize/ArgoCD 含む）
- [x] **API ドキュメントの拡充**: Swagger/OpenAPI による仕様書生成

### v0.9.1 フォーカス（非人間修正・整頓）

- [x] **v0.9.0 Release整合**: `v0.9.0` タグ範囲に合わせてリリースノートを更新（`#84`-`#91`）
- [x] **README整備**: 古い参照更新（`--ref v0.9.0`）と Markdown lint 指摘の解消
- [x] **Docs棚卸し**: 不要DOCの削除候補と `.gitignore` 化方針を整理（`docs/DOCS_INVENTORY_v0_9_1.md`）
- [x] **複雑度の高い箇所の改修**: `launcher/main.go` の `executeSelection` を分割して段階的リファクタを実施

### v0.8.x 実績（デプロイ基盤）

- [x] **AWS先行IaC運用**: GitHub Actions + Terraform の `validate/plan/apply/destroy` を `dev` で実行可能化
- [x] **環境分離**: `dev/stg/prod` の `aws.tfvars` を追加
- [x] **オフライン検証経路**: AWSシークレット未設定でも `plan` を実行できる導線を整備
- [x] **ロールバック運用**: `destroy` 手順を runbook 化し、実行証跡を追加
- [ ] **マルチクラウド再拡張**: GCP/Azure ターゲットの再導入
- [x] **Kubernetes最小構成**: Helm/Kustomize/ArgoCD 雛形の追加（`deploy/helm/postal-converter-ja`, `deploy/k8s/base`, `deploy/argocd`）

## バージョン

**v0.9.0 (Beta)** - Operations readiness package and launcher UX hardening

## 貢献について (Contributing)

Postal Converter JA はオープンソースプロジェクトであり、皆様からの貢献を歓迎します！
バグ報告、機能追加、ドキュメント改善など、どんな形でも構いません。

詳細なガイドラインについては、[CONTRIBUTING.md](CONTRIBUTING.md) をご覧ください。
また、コミュニティの行動規範として [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) を定めています。

> [!NOTE]
> 依存関係の更新は **Dependabot** によって毎週（Weekly）自動的にチェックされ、PR が作成されます。

## スポンサー募集

本プロジェクトの安定的かつ継続的な運用のために、スポンサー企業様を募集しています。

### 📮 日本郵便株式会社（Japan Post）様へ

本システムは、貴社の郵便番号データをより扱いやすく、現代的な Web 開発の現場で活用しやすくするために開発されました。
もし本プロジェクトの趣旨にご賛同いただけるようでしたら、公式なスポンサー、あるいは技術的なパートナーシップをご検討いただければ幸いです。
正確で使いやすい住所データインフラを、共に構築できることを願っております。
