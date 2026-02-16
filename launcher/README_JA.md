# Postal Converter JA Launcher 🚀

Postal Converter JA の各コンポーネント（Docker, Crawler, API, Frontend）を一括管理・起動するための CLI ツールです。
Go 言語と Bubbletea を使用し、リッチな TUI（テキストユーザーインターフェース）を提供します。

## 必要要件

- Go 1.21 以上 (Nix 環境に含まれています)
- Docker & Docker Compose

## 使い方

プロジェクトルートで以下のコマンドを実行してください：

```bash
nix develop --command go run ./launcher/main.go
```

`launcher` ディレクトリ内で実行する場合は次です。

```bash
nix develop --command go run main.go
```

または、ビルドして実行することも可能です：

```bash
cd launcher
nix develop --command go build -o postal-launcher
./postal-launcher
```

> [!NOTE]
> ランチャーには実行順序の制御があります。
> **Databases -> Crawler/API -> Frontend** の順に起動してください。
> 前提となるサービスが起動していない場合、次のステップはロックされています。

## 機能

- **Start Databases**: `docker-compose up -d` を実行し、MySQL/PostgreSQL を起動します。
- **Start Crawler**: 新しいターミナルウィンドウを開き、Crawler を実行します。
- **Start API Server**: 新しいターミナルウィンドウを開き、API サーバーを実行します。
- **Start Frontend**: 新しいターミナルウィンドウを開き、Next.js フロントエンドを実行します。
- **Stop Databases**: `docker-compose down` を実行し、データベースを停止します。

## 開発者向け

このランチャーは `bubbletea` フレームワークを使用しています。
新しいコマンドを追加したい場合は、`main.go` の `choices` と `executeSelection` 関数を編集してください。
