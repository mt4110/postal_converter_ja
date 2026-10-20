# CI/CD Integration Test Design

## 🎯 Objective

Prevent regressions such as:

- Database Schema vs Rust Struct type mismatches (e.g., `SMALLINT` vs `i32`).
- Runtime panics due to missing environment variables or files.
- Parsing errors for specific edge cases.

## ⚠️ Constraints & Risks

- **Japan Post Server Load**: Running a full download (120MB+) on every PR is **dangerous**. It risks IP blocking and puts unnecessary load on public infrastructure.
- **CI Execution Time**: Processing 120,000+ records takes time, slowing down the feedback loop.
- **Flakiness**: External network dependencies make CI unreliable.

## 💡 Proposed Solution: "Fixture Data Strategy"

Instead of downloading the real data from Japan Post every time, we use a **small, local sample file** for testing.

### 1. Test Data Preparation (`tests/fixtures/`)

Create a `sample_ken_all.zip` containing a CSV with ~10-20 representative records:

- Normal address
- Long address (to test `VARCHAR` limits)
- Address with parentheses (to test parsing logic)
- Address with `multi_town` flag (to test merging logic)

### 2. CI Workflow (GitHub Actions)

We will add a new job `integration-test` to `.github/workflows/ci.yml`.

#### Steps:

1.  **Service Containers**: Spin up MySQL and PostgreSQL (fresh containers).
2.  **Build**: Compile `crawler` and `api` binaries.
3.  **Setup**:
    - Place `sample_ken_all.zip` into `worker/crawler/temp_assets/`.
    - Set `ZIP_CODE_URL` to a dummy value (or file path if supported).
    - **Crucial**: Configure Crawler to skip download if file exists, OR use a "Local File Mode".
4.  **Run Crawler**:
    - Execute `cargo run --bin crawler`.
    - **Verification**: It should process the 10 records and exit successfully without panic.
5.  **Run API**:
    - Execute `cargo run --bin api` in background.
    - **Verification**: Use `curl` to query the API (e.g., `GET /postal_codes/1000001`).
    - Assert that the response matches the expected JSON from the fixture data.

### 3. Implementation Details

- **Crawler Modification**: Add a flag or env var (e.g., `CRAWLER_SOURCE=local`) to force using a local file instead of downloading.
- **Schedule**: Run this on every PR to `main` and `develop`. Since it uses local data, it finishes in seconds and generates **zero network traffic** to Japan Post.

## 📈 Future Expansion (Optional)

- **Weekly Full Test**: If we really want to test the _real_ Japan Post file (to catch format changes), we can create a separate Scheduled Workflow that runs once a week. This minimizes load while ensuring long-term compatibility.

## Kubernetes Manifest Validation Policy (v0.8.4)

Helm chart の回帰検知は `.github/workflows/ci.yml` の `helm` ジョブで行う。

- `helm lint` を実行する。
- `helm template` を `default/dev/stg/prod` の4パターンでレンダリングする。
- `kubeconform` は `-strict -summary` を必須にし、型不整合を fail させる。
- CRD（現時点では `ExternalSecret`）は標準スキーマ未提供ケースがあるため、`-ignore-missing-schemas` を併用する。

運用ルール:

- `-ignore-missing-schemas` は「CRD由来の未解決スキーマのみ」を許容するための例外として扱う。
- 新規 CRD を導入した場合は、PR に「なぜこの例外が必要か」を記載する。
- CRD の安定スキーマ配布元を固定できる段階になったら、`-ignore-missing-schemas` の解除を検討する。

## Veil Secret Scan

`.github/workflows/veil.yml` は、既存のアプリケーション検証と独立してmainへのpush・main向けPR・手動実行で動く。
Veil 0.22.1のLinux x86_64 musl配布ZIPとSHA-256を固定し、`scripts/ci_veil.py` からGit追跡ファイルの作業ツリーを検査する。
Git履歴や未追跡ファイルの検査は含まない。baseline自身とバイナリのスキップはレポートの対象数に影響する。

- `--fail-on-severity High` は、この版ではスコア70以上で終了1となる。JSONの重大度ラベルだけで判定しない。
- `veil.baseline.json` の12件は、公開設定例・テンプレートの確認済み候補。各例外に理由と元行のSHA-256を持ち、元行の変更・移動時は補助スクリプトが終了2で再確認を求める。
- 例外の自動再生成やルール全体の無効化はしない。更新時は対象の元行を再レビューする。
- 検査の終了0・1・2を維持し、エラーや不完全な検査を成功扱いにしない。
- runnerの一時ディレクトリへ、件数・ルール・パス・行番号・重大度・スコアだけを保存する。ソース断片を保存せず、GitHub artifactの保持は7日間。
- workflowの権限は `contents: read`。ブランチ保護やrulesetでの必須チェック指定は別の設定で、今回変更していない。

ローカルでも同じ判定を使う場合は、配布版Veilを展開したうえで次を実行する。レポートはリポジトリ外の新しい保存先へ出力する。

```bash
python3 -m unittest discover -s scripts/tests -p test_ci_veil.py
python3 scripts/ci_veil.py --veil-bin /path/to/veil --report /path/outside/repository/veil-report.json
```

実行後はGitHub ActionsのSummaryで終了コードと件数を確認し、`veil-report` artifact内のJSONからルールID・パス・行番号・スコアを確認する。
