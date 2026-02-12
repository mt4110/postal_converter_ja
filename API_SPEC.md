# Postal Converter JA – API Specification

**Version:** v0.4.0-beta  
**Base URL (Local API Server):**
http://localhost:3202

## 1. エンドポイント一覧

| Method | Path                        | Description                    |
| ------ | --------------------------- | ------------------------------ |
| GET    | `/postal_codes/:zip_code`   | 郵便番号 → 住所検索            |
| GET    | `/postal_codes/search`      | 住所（部分一致）→ 郵便番号検索 |
| GET    | `/postal_codes/prefectures` | 都道府県一覧取得               |
| GET    | `/postal_codes/cities`      | 指定都道府県の市区町村一覧     |
| GET    | `/health`                   | API の状態チェック             |
| GET    | `/ready`                    | API の準備状態チェック         |
| GET    | `/metrics`                  | 最低限メトリクス出力           |
| GET    | `/openapi.json`             | OpenAPI 仕様(JSON)             |
| GET    | `/docs`                     | Swagger UI                     |

## 2. 詳細仕様

### GET /postal_codes/:zip_code

Example

GET http://localhost:3202/postal_codes/1000001

```
Response
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

```
Error
{
  "error": "not found"
}

```

### GET /postal_codes/search

Query Parameters

| Name      | Required | Description                             |
| --------- | -------- | --------------------------------------- |
| `address` | true     | 住所キーワード（かな揺れ正規化を適用） |
| `limit`   | false    | デフォルト 50                           |
| `mode`    | false    | `exact` / `prefix` / `partial`(default) |

Example

GET http://localhost:3202/postal_codes/search?address=新宿&mode=partial&limit=20

補足:

- 住所キーワードは NFKC 正規化 + 空白除去を行う
- かな揺れとして「ひらがな/カタカナ/半角カナ」の差分を吸収して検索する

Example Response

```
[
  {
    "zip_code": "1600023",
    "prefecture": "東京都",
    "city": "新宿区",
    "town": "西新宿"
  }
]
```

### GET /postal_codes/prefectures

Example

GET http://localhost:3202/postal_codes/prefectures

Example Response

```
[
  { "prefecture_id": 1, "prefecture": "北海道" },
  { "prefecture_id": 13, "prefecture": "東京都" }
]
```

### GET /postal_codes/cities?prefecture_id=13

Example

GET http://localhost:3202/postal_codes/cities?prefecture_id=13

Example Response

```
[
  { "city_id": "13101", "city": "千代田区" },
  { "city_id": "13102", "city": "中央区" }
]
```

### GET /health

Example

GET http://localhost:3202/health

Example Response

```
{ "status": "ok" }
```

### GET /ready

Example

GET http://localhost:3202/ready

Example Response

```
{
  "status": "ready",
  "database": "postgres",
  "cache": "ok"
}
```

Error Response (DB未接続時など)

```
{
  "error": "database not ready"
}
```

運用ポリシー (`READY_REQUIRE_CACHE`):

- `false`（デフォルト）: Redis 障害時でも `200` を返し、`cache` は `error`
- `true`: `REDIS_URL` が設定されている時、Redis 障害で `503`（`{"error":"cache not ready"}`）

### GET /metrics

Example

GET http://localhost:3202/metrics

Example Response

```
{
  "requests_total": 245,
  "errors_total": 2,
  "not_found_total": 14,
  "error_rate": 0.00816326530612245,
  "average_latency_ms": 6.42
}
```

### エラーフォーマット（統一）

```
{
  "error": "not found"
}
```

### 4. 認証

現状は認証機構なし（ネットワーク境界で制御）。

### 5. Versioning

API は SemVer に従い version upgrade で破壊的変更を管理。

現在は beta のため Breaking Changes は許容される。

### 6. Database 切り替え仕様

```
DATABASE_TYPE=postgres
```

または

```
DATABASE_TYPE=mysql
```

または

```
DATABASE_TYPE=sqlite
SQLITE_DATABASE_PATH=storage/sqlite/postal_codes.sqlite3
```

Readiness 厳密化オプション:

```
READY_REQUIRE_CACHE=false
```

※`postgres` / `mysql` 運用時は Crawler / API ともに同じ値で運用すること。

> [!NOTE]
> `sqlite` は API の read-only PoC 用です。Crawler の SQLite 更新は未対応です。

## 7. 更新データの元（Japan Post）

日本郵便公式 CSV（最新データ）

Crawler が毎日自動取得

差分更新（削除 / 変更 / 新規）

## 実行環境

| Component  | Port     |
| ---------- | -------- |
| API        | **3202** |
| Frontend   | **3203** |
| MySQL      | **3204** |
| PostgreSQL | **3205** |
