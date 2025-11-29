# 郵便番号自動最新化システム (Postal Converter JA)

![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)
![Status](https://img.shields.io/badge/status-beta-orange.svg)

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

### Nix のインストール

まだ Nix をインストールしていない場合は、以下のコマンドでインストールしてください：

```bash
# 公式インストールスクリプト（macOS/Linux）
sh <(curl -L https://nixos.org/nix/install)

# または、Determinate Systems版（推奨）
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

インストール後、シェルを再起動してください。

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
cargo run --release
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
cargo run --release
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

### ポート競合エラー（Address already in use）

API を起動時に以下のエラーが表示される場合があります：

```
called `Result::unwrap()` on an `Err` value: Os { code: 48, kind: AddrInUse, message: "Address already in use" }
```

これは既に別のプロセスがポート 3202 を使用している場合に発生します。以下の手順で解決してください：

```bash
# ポート3202を使用しているプロセスを特定
lsof -i:3202

# 出力例:
# COMMAND   PID           USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
# api     36659 masakitakemura   10u  IPv4 0x1e7f9d9246fd07f9      0t0  TCP *:intraintra (LISTEN)

# プロセスを終了（PIDは実際の値に置き換えてください）
kill -9 36659

# APIを再起動
cargo run --release
```

### データベース接続エラー

Crawler や api が「Failed to connect to database」のようなエラーを出す場合：

1. Docker コンテナが起動しているか確認：

   ```bash
   docker ps
   ```

2. コンテナが起動していない場合は起動：

   ```bash
   docker-compose up -d
   ```

3. `.env` ファイルのデータベース URL が正しいか確認

### Nix 環境について

Rust のコンパイルやビルドは、**必ず `nix develop` を実行してから**行ってください。
Nix 環境に入らずに実行すると、依存関係やツールチェーンが見つからないエラーが発生します。

## API エンドポイント

### `GET /postal_codes/:zip_code`

指定された郵便番号に対応する住所情報を返します。

```json
// GET /postal_codes/1000001
[
  {
    "zip_code": "1000001",
    "prefecture_id": 13,
    "city_id": "13101",
    "prefecture": "東京都",
    "city": "千代田区",
    "town": "千代田"
  }
]
```

### `GET /postal_codes/search?address=...`

住所の一部から郵便番号を検索します。

### `GET /postal_codes/prefectures`

都道府県の一覧を返します。

### `GET /postal_codes/cities?prefecture_id=...`

指定された都道府県の市区町村一覧を返します。

## 開発者向け情報

### Lint & Format

- **Frontend**: `yarn lint` (ESLint 8 + Prettier)
- **Backend**: `cargo fmt`, `cargo clippy`

### ディレクトリ構成

- `frontend/`: Next.js アプリケーション
- `worker/api/`: Rust API サーバー
- `worker/crawler/`: Rust データ更新クローラー
- `worker/common/`: 共有 Rust モジュール (DB 接続, モデル定義)

### データベースの切り替え

環境変数 `DATABASE_TYPE` を設定することで、使用するデータベースを切り替えられます：

```bash
# .env ファイル（worker/crawler/.env と worker/api/.env）

# PostgreSQLを使用（デフォルト）
DATABASE_TYPE=postgres
POSTGRES_DATABASE_URL=postgres://postgres:postgres_password@127.0.0.1:3205/zip_code_db

# MySQLを使用
DATABASE_TYPE=mysql
MYSQL_DATABASE_URL=mysql://mysql_user:u_password@127.0.0.1:3204/zip_code_db
```

Crawler と API で同じ `DATABASE_TYPE` を設定してください。

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

- [ ] **CI/CD パイプラインの構築**: GitHub Actions による自動テスト・ビルド
- [ ] **MySQL/PostgreSQL の自動テスト**: 両 DB でのインテグレーションテスト追加
- [ ] **Docker イメージの軽量化**: マルチステージビルドの最適化
- [ ] **API ドキュメントの拡充**: Swagger/OpenAPI による仕様書生成

## スポンサー募集

本プロジェクトの安定的かつ継続的な運用のために、スポンサー企業様を募集しています。

### 📮 日本郵便株式会社（Japan Post）様へ

本システムは、貴社の郵便番号データをより扱いやすく、現代的な Web 開発の現場で活用しやすくするために開発されました。
もし本プロジェクトの趣旨にご賛同いただけるようでしたら、公式なスポンサー、あるいは技術的なパートナーシップをご検討いただければ幸いです。
正確で使いやすい住所データインフラを、共に構築できることを願っております。
