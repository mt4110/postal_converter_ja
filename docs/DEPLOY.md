# Deployment Guide (GCP) 🚀

本プロジェクトは、Google Cloud Platform (GCP) へのデプロイを推奨しています。
インフラ構成管理 (IaC) には、ランチャーと同じ **Go 言語** を使用できる **Pulumi** を採用予定です。

## 推奨アーキテクチャ

### 1. Compute (Cloud Run)

コンテナベースのサーバーレスプラットフォームである Cloud Run を使用します。

- **Frontend**: Next.js アプリケーション
- **API Server**: Rust API サーバー
- **Crawler**: バッチ処理として Cloud Run Jobs を使用（または定期実行トリガー）

### 2. Database (Cloud SQL)

マネージドなリレーショナルデータベースサービス。

- **PostgreSQL**: 本番環境での推奨 DB
- **MySQL**: オプションとして選択可能

### 3. Infrastructure as Code (IaC)

**Pulumi with Go** を使用して、インフラ全体をコードで定義・管理します。

```go
// 構成イメージ (Go)
func main() {
	pulumi.Run(func(ctx *pulumi.Context) error {
		// Cloud SQL インスタンス作成
		instance, err := sql.NewDatabaseInstance(ctx, "postal-db", &sql.DatabaseInstanceArgs{
			DatabaseVersion: pulumi.String("POSTGRES_15"),
			Settings: &sql.DatabaseInstanceSettingsArgs{
				Tier: pulumi.String("db-f1-micro"),
			},
		})

		// Cloud Run サービス作成 (API)
		apiService, err := cloudrun.NewService(ctx, "postal-api", &cloudrun.ServiceArgs{
			Template: &cloudrun.ServiceTemplateArgs{
				Spec: &cloudrun.ServiceTemplateSpecArgs{
					Containers: cloudrun.ServiceTemplateSpecContainerArray{
						&cloudrun.ServiceTemplateSpecContainerArgs{
							Image: pulumi.String("gcr.io/my-project/postal-api:latest"),
						},
					},
				},
			},
		})
		return nil
	})
}
```

## デプロイ手順 (予定)

1. **GCP プロジェクトの準備**: `gcloud auth login`
2. **IaC の実行**: `cd iac && go run main.go up`
3. **アプリケーションのデプロイ**: GitHub Actions から自動デプロイ

---

_Note: このドキュメントは v0.2.1 に向けて随時更新されます。_
