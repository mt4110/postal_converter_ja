use deadpool_postgres::Pool as PgPool;
use mysql_async::{params, prelude::Queryable, Pool as MySqlPool};
use tokio_postgres::Error as PgError;

#[derive(Debug, Clone)]
pub struct DataUpdateAuditRecord {
    pub data_version: String,
    pub source_url: String,
    pub run_started_at: chrono::DateTime<chrono::Utc>,
    pub run_finished_at: chrono::DateTime<chrono::Utc>,
    pub batch_timestamp: chrono::NaiveDateTime,
    pub records_in_feed: i64,
    pub inserted_count: i64,
    pub updated_count: i64,
    pub deleted_count: i64,
    pub total_count: i64,
    pub status: String,
    pub error_message: Option<String>,
}

pub fn build_data_version(batch_timestamp: chrono::NaiveDateTime) -> String {
    format!(
        "v{}{:03}",
        batch_timestamp.format("%Y%m%d%H%M%S"),
        batch_timestamp.and_utc().timestamp_subsec_millis()
    )
}

fn naive_utc_to_utc(batch_timestamp: chrono::NaiveDateTime) -> chrono::DateTime<chrono::Utc> {
    batch_timestamp.and_utc()
}

pub async fn ensure_audit_table_postgres(pool: &PgPool) -> Result<(), PgError> {
    let client = pool.get().await.expect("Failed to get client");
    client
        .batch_execute(
            r#"
            CREATE TABLE IF NOT EXISTS data_update_audits (
                id BIGSERIAL PRIMARY KEY,
                data_version VARCHAR(32) NOT NULL UNIQUE,
                database_type VARCHAR(16) NOT NULL,
                source_url TEXT NOT NULL,
                run_started_at TIMESTAMPTZ NOT NULL,
                run_finished_at TIMESTAMPTZ NOT NULL,
                batch_timestamp TIMESTAMPTZ NOT NULL,
                records_in_feed BIGINT NOT NULL,
                inserted_count BIGINT NOT NULL,
                updated_count BIGINT NOT NULL,
                deleted_count BIGINT NOT NULL,
                total_count BIGINT NOT NULL,
                status VARCHAR(16) NOT NULL,
                error_message TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_data_update_audits_created_at
                ON data_update_audits (created_at DESC);
        "#,
        )
        .await?;
    Ok(())
}

pub async fn ensure_snapshot_table_postgres(pool: &PgPool) -> Result<(), PgError> {
    let client = pool.get().await.expect("Failed to get client");
    client
        .batch_execute(
            r#"
            CREATE TABLE IF NOT EXISTS postal_codes_snapshots (
                data_version VARCHAR(32) NOT NULL,
                zip_code CHAR(7) NOT NULL,
                prefecture_id SMALLINT NOT NULL,
                city_id VARCHAR(10) NOT NULL,
                prefecture VARCHAR(32) NOT NULL,
                city VARCHAR(50) NOT NULL,
                town VARCHAR(500),
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                snapshot_created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (data_version, zip_code, prefecture_id, city, town)
            );
            CREATE INDEX IF NOT EXISTS idx_postal_codes_snapshots_version
                ON postal_codes_snapshots (data_version);
        "#,
        )
        .await?;
    Ok(())
}

pub async fn create_postgres_snapshot(pool: &PgPool, data_version: &str) -> Result<u64, PgError> {
    let client = pool.get().await.expect("Failed to get client");
    client
        .execute(
            "INSERT INTO postal_codes_snapshots (
                data_version, zip_code, prefecture_id, city_id, prefecture, city, town, created_at, updated_at
            )
            SELECT
                $1, zip_code, prefecture_id, city_id, prefecture, city, town, created_at, updated_at
            FROM postal_codes
            ON CONFLICT (data_version, zip_code, prefecture_id, city, town) DO NOTHING",
            &[&data_version],
        )
        .await
}

pub async fn compute_postgres_diff_counts(
    pool: &PgPool,
    batch_timestamp: chrono::NaiveDateTime,
) -> Result<(i64, i64, i64), PgError> {
    let client = pool.get().await.expect("Failed to get client");
    let batch_timestamp_utc = naive_utc_to_utc(batch_timestamp);

    let touched_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::BIGINT FROM postal_codes WHERE updated_at = $1",
            &[&batch_timestamp_utc],
        )
        .await?
        .get(0);

    let inserted_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::BIGINT
             FROM postal_codes
             WHERE updated_at = $1 AND created_at = $1",
            &[&batch_timestamp_utc],
        )
        .await?
        .get(0);

    let total_count: i64 = client
        .query_one("SELECT COUNT(*)::BIGINT FROM postal_codes", &[])
        .await?
        .get(0);

    let updated_count = (touched_count - inserted_count).max(0);
    Ok((inserted_count, updated_count, total_count))
}

pub async fn insert_audit_postgres(
    pool: &PgPool,
    record: &DataUpdateAuditRecord,
) -> Result<(), PgError> {
    let client = pool.get().await.expect("Failed to get client");
    let batch_timestamp_utc = naive_utc_to_utc(record.batch_timestamp);
    client
        .execute(
            "INSERT INTO data_update_audits (
                data_version, database_type, source_url,
                run_started_at, run_finished_at, batch_timestamp,
                records_in_feed, inserted_count, updated_count, deleted_count, total_count,
                status, error_message
            ) VALUES (
                $1, 'postgres', $2,
                $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12
            )",
            &[
                &record.data_version,
                &record.source_url,
                &record.run_started_at,
                &record.run_finished_at,
                &batch_timestamp_utc,
                &record.records_in_feed,
                &record.inserted_count,
                &record.updated_count,
                &record.deleted_count,
                &record.total_count,
                &record.status,
                &record.error_message,
            ],
        )
        .await?;
    Ok(())
}

pub async fn ensure_audit_table_mysql(pool: &MySqlPool) -> Result<(), mysql_async::Error> {
    let mut conn = pool.get_conn().await?;
    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS data_update_audits (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            data_version VARCHAR(32) NOT NULL UNIQUE,
            database_type VARCHAR(16) NOT NULL,
            source_url TEXT NOT NULL,
            run_started_at DATETIME NOT NULL,
            run_finished_at DATETIME NOT NULL,
            batch_timestamp DATETIME NOT NULL,
            records_in_feed BIGINT NOT NULL,
            inserted_count BIGINT NOT NULL,
            updated_count BIGINT NOT NULL,
            deleted_count BIGINT NOT NULL,
            total_count BIGINT NOT NULL,
            status VARCHAR(16) NOT NULL,
            error_message TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_data_update_audits_created_at (created_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci",
    )
    .await?;
    Ok(())
}

pub async fn ensure_snapshot_table_mysql(pool: &MySqlPool) -> Result<(), mysql_async::Error> {
    let mut conn = pool.get_conn().await?;
    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS postal_codes_snapshots (
            data_version VARCHAR(32) NOT NULL,
            zip_code CHAR(7) NOT NULL,
            prefecture_id SMALLINT NOT NULL,
            city_id VARCHAR(10) NOT NULL,
            prefecture VARCHAR(32) NOT NULL,
            city VARCHAR(50) NOT NULL,
            town VARCHAR(500),
            created_at TIMESTAMP NOT NULL,
            updated_at TIMESTAMP NOT NULL,
            snapshot_created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (data_version, zip_code, prefecture_id, city, town),
            INDEX idx_postal_codes_snapshots_version (data_version)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci",
    )
    .await?;
    Ok(())
}

pub async fn create_mysql_snapshot(
    pool: &MySqlPool,
    data_version: &str,
) -> Result<u64, mysql_async::Error> {
    let mut conn = pool.get_conn().await?;
    conn.exec_drop(
        "INSERT IGNORE INTO postal_codes_snapshots (
            data_version, zip_code, prefecture_id, city_id, prefecture, city, town, created_at, updated_at
        )
        SELECT
            :data_version, zip_code, prefecture_id, city_id, prefecture, city, town, created_at, updated_at
        FROM postal_codes",
        params! {
            "data_version" => data_version,
        },
    )
    .await?;
    Ok(conn.affected_rows())
}

pub async fn compute_mysql_diff_counts(
    pool: &MySqlPool,
    batch_timestamp: chrono::NaiveDateTime,
) -> Result<(i64, i64, i64), mysql_async::Error> {
    let mut conn = pool.get_conn().await?;

    let touched_count = conn
        .exec_first::<i64, _, _>(
            "SELECT COUNT(*) FROM postal_codes WHERE updated_at = :batch_timestamp",
            params! {
                "batch_timestamp" => batch_timestamp,
            },
        )
        .await?
        .unwrap_or(0);

    let inserted_count = conn
        .exec_first::<i64, _, _>(
            "SELECT COUNT(*)
             FROM postal_codes
             WHERE updated_at = :batch_timestamp AND created_at = :batch_timestamp",
            params! {
                "batch_timestamp" => batch_timestamp,
            },
        )
        .await?
        .unwrap_or(0);

    let total_count = conn
        .query_first::<i64, _>("SELECT COUNT(*) FROM postal_codes")
        .await?
        .unwrap_or(0);

    let updated_count = (touched_count - inserted_count).max(0);
    Ok((inserted_count, updated_count, total_count))
}

pub async fn insert_audit_mysql(
    pool: &MySqlPool,
    record: &DataUpdateAuditRecord,
) -> Result<(), mysql_async::Error> {
    let mut conn = pool.get_conn().await?;
    conn.exec_drop(
        "INSERT INTO data_update_audits (
            data_version, database_type, source_url,
            run_started_at, run_finished_at, batch_timestamp,
            records_in_feed, inserted_count, updated_count, deleted_count, total_count,
            status, error_message
        ) VALUES (
            :data_version, 'mysql', :source_url,
            :run_started_at, :run_finished_at, :batch_timestamp,
            :records_in_feed, :inserted_count, :updated_count, :deleted_count, :total_count,
            :status, :error_message
        )",
        params! {
            "data_version" => &record.data_version,
            "source_url" => &record.source_url,
            "run_started_at" => record.run_started_at.naive_utc(),
            "run_finished_at" => record.run_finished_at.naive_utc(),
            "batch_timestamp" => record.batch_timestamp,
            "records_in_feed" => record.records_in_feed,
            "inserted_count" => record.inserted_count,
            "updated_count" => record.updated_count,
            "deleted_count" => record.deleted_count,
            "total_count" => record.total_count,
            "status" => &record.status,
            "error_message" => &record.error_message,
        },
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_data_version;

    #[test]
    fn build_data_version_has_fixed_prefix_and_length() {
        let ts = chrono::NaiveDate::from_ymd_opt(2026, 2, 12)
            .expect("valid date")
            .and_hms_milli_opt(21, 37, 5, 123)
            .expect("valid time");
        let v = build_data_version(ts);
        assert_eq!(v, "v20260212213705123");
    }
}
