# Postal Converter JA – API Specification

**Version:** v0.1.0-beta  
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
| `address` | true     | 住所の部分一致（例:「新宿」「千代田」） |
| `limit`   | false    | デフォルト 50                           |

Example

GET http://localhost:3202/postal_codes/search?address=新宿&limit=20

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
  { "id": 1, "name": "北海道" },
  { "id": 13, "name": "東京都" }
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

### エラーフォーマット（統一）

```
{
  "error": {
    "message": "postal code not found",
    "code": "NOT_FOUND"
  }
}
```

### 認証（現状なし）

将来的には Optional API Key に対応可能。

### Versioning

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

※Crawler / API ともに同じ値で運用すること。

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
