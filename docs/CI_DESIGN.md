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
