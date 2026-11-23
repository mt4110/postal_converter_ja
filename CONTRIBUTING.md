# Contributing to Postal Converter JA

First off, thanks for taking the time to contribute! 🎉

The following is a set of guidelines for contributing to Postal Converter JA. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## Code of Conduct

This project and everyone participating in it is governed by the [Postal Converter JA Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

This section guides you through submitting a bug report for Postal Converter JA. Following these guidelines helps maintainers and the community understand your report, reproduce the behavior, and find related reports.

- **Use the Bug Report Template**: When you create a new issue, please select the "Bug report" template.
- **Provide Reproducible Steps**: Include clear steps to reproduce the issue.
- **Check for Duplicates**: Before creating a new issue, please search existing issues to see if the problem has already been reported.

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for Postal Converter JA, including completely new features and minor improvements to existing functionality.

- **Use the Feature Request Template**: When you create a new issue, please select the "Feature request" template.
- **Be Specific**: Describe the behavior you want and why.

### Pull Requests

1.  **Fork the repo** and create your branch from `develop`.
2.  **Ensure CI passes**: Run tests and linting locally before pushing.
    - Backend: `cargo test`
    - Frontend: `yarn lint`
    - Launcher: `go build ./...`
3.  **Use the PR Template**: Fill out the Pull Request template to provide context for your changes.
4.  **Link Issues**: If your PR fixes an issue, link it in the description (e.g., `Fixes #123`).

## Styleguides

### Git Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line
- Consider using [Conventional Commits](https://www.conventionalcommits.org/) (e.g., `feat:`, `fix:`, `docs:`, `chore:`)

### Rust Styleguide

- Run `cargo fmt` before committing.
- Ensure `cargo clippy` passes without warnings.

### TypeScript/React Styleguide

- Run `yarn lint` to check for style issues.
- Use functional components and hooks.

Thank you for your contribution!

## コミット前の脆弱性検査

OSV-Scanner v2、Trivy、`jq`、`curl`、Docker（Compose と Buildx を含む）を用意し、
リポジトリのルートで `git config --local core.hooksPath .githooks` を実行します。
既に独自フックを使っている場合は、既存のフックから `.githooks/pre-commit` を呼び出してください。

フックは Git インデックスを一時ディレクトリに展開し、コミットに入る依存定義と
Dockerfile を検査します。ロックファイルと Go ツールチェーンの既知アドバイザリ、
Nix とフロントエンド CI が共有する Node.js、および Nix と Docker の Rust バージョンの NVD 検査、
Compose イメージ（無効化されているプロファイルも含む）および API・クローラーのベース、
ビルド、実行イメージを確認します。Trivy は Oracle Linux などの OS パッケージ検査を補完します。
検出・検査失敗のどちらでもコミットは中断します。
OSV、NVD、Trivy のデータベースへのネットワーク接続、および Docker イメージを取得・ビルドするための容量が必要です。

個別調査は `bash scripts/security-scan.sh source` または
`bash scripts/security-scan.sh images` で実行できます。GitHub Actions は両方を実行します。
検査範囲は、データベースが収録する既知のアドバイザリと、その実行環境向けにビルドした
イメージです。Docker Engine 自体はホスト側で更新・管理してください。

### Compose の PostgreSQL ボリュームを移行する

既存の Debian 版 PostgreSQL 16 のボリュームは Alpine 版に直接マウントせず、
論理バックアップを新しいボリュームに復元します。アプリからの書き込みを止め、
旧コンテナが動いている間にバックアップを取得します。追加のロールがある環境では、
復元先に同じロールを作成してからデータを復元してください。

```sh
migration_dir=$(mktemp -d)
docker exec postgres_container sh -ec '
  PGPASSWORD="$POSTGRES_PASSWORD" pg_dump -h 127.0.0.1 -U "$POSTGRES_USER" -Fc "$POSTGRES_DB"
' > "$migration_dir/postgres.dump"
test -s "$migration_dir/postgres.dump"
```

バックアップ取得に成功したら、旧コンテナを停止し、新規の名前付きボリュームで起動します。
既存スキーマとの競合を避けるため、初回だけ空の SQL を指定します。
ホストのディレクトリへバインドするオーバーライドを使っている場合、
復元時はそのオーバーライドを外し、復元後も新しいボリュームを指定します。

```sh
docker stop postgres_container
touch "$migration_dir/empty.sql"
export POSTAL_POSTGRES_VOLUME="postal_postgres_alpine_$(date +%Y%m%d%H%M%S)"
POSTAL_POSTGRES_INIT_SQL="$migration_dir/empty.sql" docker compose up -d postgres
docker compose exec -T postgres sh -ec '
  PGPASSWORD="$POSTGRES_PASSWORD" pg_isready -h 127.0.0.1 -U "$POSTGRES_USER" -d "$POSTGRES_DB"
'
```

起動確認後に復元し、件数と郵便番号の検索結果を確認します。
以降の Compose 起動でも `POSTAL_POSTGRES_VOLUME` を同じ値に設定します。
元のボリュームとバックアップは復旧用に保持します。

```sh
docker compose exec -T postgres sh -ec '
  PGPASSWORD="$POSTGRES_PASSWORD" pg_restore -h 127.0.0.1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" --exit-on-error
' < "$migration_dir/postgres.dump"
docker compose exec -T postgres sh -ec '
  PGPASSWORD="$POSTGRES_PASSWORD" psql -h 127.0.0.1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" -v ON_ERROR_STOP=1 -c "SELECT count(*) FROM postal_codes"
'
```
