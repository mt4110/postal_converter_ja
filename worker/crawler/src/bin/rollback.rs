use chrono::Timelike;
use crawler_service::db::audit::{
    build_data_version, ensure_audit_table_mysql, ensure_audit_table_postgres,
    ensure_snapshot_table_mysql, ensure_snapshot_table_postgres, insert_audit_mysql,
    insert_audit_postgres, DataUpdateAuditRecord,
};
use mysql_async::{params, prelude::Queryable};
use std::env;

fn usage() {
    eprintln!(
        "Usage: rollback --data-version <VERSION> [--database-type postgres|mysql]\n\
         Example: rollback --database-type postgres --data-version v20260213001549224"
    );
}

fn parse_args() -> Result<(String, String), String> {
    let mut db_type: Option<String> = None;
    let mut data_version: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--database-type" => {
                let v = args
                    .next()
                    .ok_or_else(|| "--database-type requires a value".to_string())?;
                db_type = Some(v);
            }
            "--data-version" => {
                let v = args
                    .next()
                    .ok_or_else(|| "--data-version requires a value".to_string())?;
                data_version = Some(v);
            }
            "--help" | "-h" => {
                usage();
                std::process::exit(0);
            }
            other => {
                return Err(format!("Unknown argument: {other}"));
            }
        }
    }

    let database_type = db_type
        .unwrap_or_else(|| env::var("DATABASE_TYPE").unwrap_or_else(|_| "postgres".to_string()));
    let data_version = data_version.ok_or_else(|| "--data-version is required".to_string())?;
    Ok((database_type, data_version))
}

fn make_rollback_audit_record(
    target_data_version: &str,
    restored_count: i64,
) -> DataUpdateAuditRecord {
    let now_utc = chrono::Utc::now();
    let now_local = chrono::Local::now().naive_local();
    let batch_timestamp = now_local
        .with_nanosecond(0)
        .expect("failed to normalize rollback batch timestamp");
    let rollback_data_version = format!("r{}", build_data_version(now_local));

    DataUpdateAuditRecord {
        data_version: rollback_data_version,
        source_url: format!("rollback_cli:{target_data_version}"),
        run_started_at: now_utc,
        run_finished_at: now_utc,
        batch_timestamp,
        records_in_feed: restored_count,
        inserted_count: restored_count,
        updated_count: 0,
        deleted_count: 0,
        total_count: restored_count,
        status: "rollback".to_string(),
        error_message: None,
    }
}

async fn rollback_postgres(target_data_version: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let pool = common::db::postgres_connection().await?;
    ensure_audit_table_postgres(&pool).await?;
    ensure_snapshot_table_postgres(&pool).await?;

    let mut client = pool.get().await?;
    let snapshot_count: i64 = client
        .query_one(
            "SELECT COUNT(*)::BIGINT FROM postal_codes_snapshots WHERE data_version = $1",
            &[&target_data_version],
        )
        .await?
        .get(0);

    if snapshot_count == 0 {
        return Err(
            format!("No snapshot rows found for data_version={target_data_version}").into(),
        );
    }

    let tx = client.transaction().await?;
    tx.execute("DELETE FROM postal_codes", &[]).await?;
    let restored = tx
        .execute(
            "INSERT INTO postal_codes (
                zip_code, prefecture_id, city_id, prefecture, city, town, created_at, updated_at
            )
            SELECT
                zip_code, prefecture_id, city_id, prefecture, city, town, created_at, updated_at
            FROM postal_codes_snapshots
            WHERE data_version = $1",
            &[&target_data_version],
        )
        .await?;
    tx.commit().await?;

    let audit_record = make_rollback_audit_record(target_data_version, restored as i64);
    insert_audit_postgres(&pool, &audit_record).await?;

    Ok(restored)
}

async fn rollback_mysql(target_data_version: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let pool = common::db::mysql_connection().await?;
    ensure_audit_table_mysql(&pool).await?;
    ensure_snapshot_table_mysql(&pool).await?;

    let mut conn = pool.get_conn().await?;
    let snapshot_count = conn
        .exec_first::<i64, _, _>(
            "SELECT COUNT(*) FROM postal_codes_snapshots WHERE data_version = :data_version",
            params! { "data_version" => target_data_version },
        )
        .await?
        .unwrap_or(0);

    if snapshot_count == 0 {
        return Err(
            format!("No snapshot rows found for data_version={target_data_version}").into(),
        );
    }

    let mut tx = conn.start_transaction(Default::default()).await?;
    tx.query_drop("DELETE FROM postal_codes").await?;
    tx.exec_drop(
        "INSERT INTO postal_codes (
            zip_code, prefecture_id, city_id, prefecture, city, town, created_at, updated_at
        )
        SELECT
            zip_code, prefecture_id, city_id, prefecture, city, town, created_at, updated_at
        FROM postal_codes_snapshots
        WHERE data_version = :data_version",
        params! { "data_version" => target_data_version },
    )
    .await?;
    let restored = tx.affected_rows();
    tx.commit().await?;

    let audit_record = make_rollback_audit_record(target_data_version, restored as i64);
    insert_audit_mysql(&pool, &audit_record).await?;

    Ok(restored)
}

#[tokio::main]
async fn main() {
    if dotenv::from_filename(".env").is_err() {
        dotenv::from_filename("crawler/.env").ok();
    }

    let (database_type, data_version) = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            usage();
            std::process::exit(2);
        }
    };

    let result = match database_type.as_str() {
        "postgres" => rollback_postgres(&data_version).await,
        "mysql" => rollback_mysql(&data_version).await,
        other => Err(format!("Unsupported database_type: {other}").into()),
    };

    match result {
        Ok(restored) => {
            println!(
                "Rollback completed. database_type={}, target_data_version={}, restored_rows={}",
                database_type, data_version, restored
            );
        }
        Err(e) => {
            eprintln!(
                "Rollback failed. database_type={}, target_data_version={}, error={}",
                database_type, data_version, e
            );
            std::process::exit(1);
        }
    }
}
