# 郵便番号自動最新化システム (Postal Converter JA)

![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)
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
- **Mise** (オプション): Node.js/Yarn のバージョン管理に使用（推奨）
- **Nix**: 開発環境の構築に使用します（Rust ツールチェーン、ビルドツールなど）
- **Docker**: データベースの実行に使用します
- **Mise** (オプション): Node.js/Yarn のバージョン管理に使用（推奨）

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
go run main.go
```

または、ビルドして実行：

```bash
cd launcher
go build -o postal-launcher
./postal-launcher
```

ランチャーから以下の操作が可能です：

- データベースの起動/停止 (Docker)
- Crawler, API, Frontend の一括起動 (新しいターミナルで開きます)

---

## 🛠 手動セットアップ & 実行ール後、シェルを再起動してください。

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
```

### 3. Crawler の実行（郵便番号データの自動取得・更新）

**Nix 環境に入ってから**Crawler を起動します：

```bash
cd worker/crawler

# Nix環境に入る（重要！）
nix develop

# Crawlerを起動
cargo run --release --bin crawler
```

初回実行時は以下の処理が行われます：

- 日本郵便から CSV データをダウンロード
- データベースへの初期データ投入（約 12 万件）

その後、設定された間隔（デフォルト 24 時間）で自動的にデータを更新し続けます。

### 4. API サーバーの起動

**別のターミナルで**、Nix 環境に入ってから API を起動します：

```bash
cd worker/api

# Nix環境に入る（重要！）
nix develop

# APIサーバーを起動
cargo run --release --bin api
```

API サーバーは `http://localhost:3202` で起動します。

### 5. フロントエンドの起動

さらに**別のターミナルで**フロントエンドを起動します：

```bash
cd frontend

# 依存関係のインストール（初回のみ）
yarn install

# 開発サーバーの起動
yarn dev
```

ブラウザで `http://localhost:3203` にアクセスすると、郵便番号検索のデモ画面が表示されます。

## トラブルシューティング

👉 **トラブルシューティングについてはこちら:** [TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md)

## 開発者向け情報

👉 **API ドキュメントはこちら:** [API_SPEC.md](./API_SPEC.md)

👉 **開発者向け情報についてはこちら:** [DEVELOPMENT.md](./docs/DEVELOPMENT.md)

## ライセンスと商用利用について

本プロジェクトは、**デュアルライセンス**（Dual Licensing）を採用する予定です。

1.  **個人利用・非営利・オープンソース開発**:

    - **MIT License** の下、自由に利用・改変・再配布が可能です。
    - 学習目的や個人プロジェクトでぜひご活用ください。

2.  **法人利用・商用サービスへの組み込み**:
    - 企業での業務利用や、商用製品への組み込みを行う場合は、**商用ライセンス**の契約、または**GitHub Sponsors**等による継続的な支援をお願いすることを想定しています。
    - （現在はプレビュー版のため、評価目的での利用は無償です。本格導入の際はご連絡ください）

このモデルにより、オープンソースとしての発展と、持続可能な開発体制の両立を目指しています。

## ロードマップ (TODO)

- [x] **CI/CD パイプラインの構築**: GitHub Actions による自動テスト・ビルド
- [x] **ランチャーの UX 改善**: 実行順序の制御と視覚的フィードバック
- [ ] **環境構築の完全自動化**: Nix, Docker, Mise のインストールとセットアップ (`mise trust` 等)
- [ ] **GCP デプロイ (v0.2.1)**: Cloud Run + Cloud SQL へのデプロイ構成
- [ ] **IaC (Infrastructure as Code)**: Go 言語 (Pulumi/Terraform CDK) によるインフラ管理
- [ ] **MySQL/PostgreSQL の自動テスト**: 両 DB でのインテグレーションテスト追加
- [ ] **Docker イメージの軽量化**: マルチステージビルドの最適化
- [ ] **API ドキュメントの拡充**: Swagger/OpenAPI による仕様書生成

## バージョン

**v0.2.0 (Beta)** - Enhanced Developer Experience & Robustness

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
