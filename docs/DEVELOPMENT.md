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
