# Setup Automation (Nix + Docker)

最終更新: 2026-02-12

`scripts/setup_nix_docker.sh` は、Nix + Docker 前提でローカル環境を一括セットアップするスクリプトです。

## できること

- Nix / Docker コマンドの存在チェック
- Docker daemon の起動確認 (Colima があれば自動起動を試行)
- 必要ディレクトリ作成 (`storage/*`, `artifacts/sqlite`)
- `.env` ファイル自動生成 (未作成時のみ)
- `nix develop` 動作確認
- そのまま `onboard.sh` を実行

## 使い方

```bash
# 通常セットアップ + dev起動
./scripts/setup_nix_docker.sh --profile dev

# demo起動まで
./scripts/setup_nix_docker.sh --profile demo

# セットアップのみ (起動しない)
./scripts/setup_nix_docker.sh --profile dev --skip-onboard
```

## オプション

- `--profile dev|demo|sqlite-release`
- `--install-missing`
- `--skip-onboard`

## install-missing について

- `--install-missing` は現在 macOS 向けです
- Homebrew が必要です
- Nix (Determinate installer), Docker CLI, Colima の導入を試行します

```bash
./scripts/setup_nix_docker.sh --install-missing --profile dev
```

## 補足

- 本番運用向けの OS 差分対応 (Linux distro 別) は今後拡張予定
- 起動済みサービス停止は `./scripts/onboard.sh --stop`
