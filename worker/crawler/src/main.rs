mod constants;
mod db;
mod file;
mod utils;
use chrono::Timelike;
use constants::temp_dir;
use db::audit::{build_data_version, DataUpdateAuditRecord};
use redis::AsyncCommands;
use tokio::time::{sleep, Duration};

async fn invalidate_redis_cache() {
    let Ok(redis_url) = std::env::var("REDIS_URL") else {
        return;
    };

    let client = match redis::Client::open(redis_url) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Invalid REDIS_URL for crawler cache invalidation: {e}");
            return;
        }
    };

    let mut conn = match client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to connect Redis for cache invalidation: {e}");
            return;
        }
    };

    let mut cursor: u64 = 0;
    let mut deleted = 0usize;

    loop {
        let scan_result: redis::RedisResult<(u64, Vec<String>)> = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg("postal:*")
            .arg("COUNT")
            .arg(500)
            .query_async(&mut conn)
            .await;

        let (next_cursor, keys) = match scan_result {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to scan Redis cache keys: {e}");
                return;
            }
        };

        if !keys.is_empty() {
            let result: redis::RedisResult<()> = conn.del(&keys).await;
            if let Err(e) = result {
                eprintln!("Failed to delete Redis cache keys: {e}");
                return;
            }
            deleted += keys.len();
        }

        if next_cursor == 0 {
            break;
        }
        cursor = next_cursor;
    }

    tlog!(
        "Redis cache invalidated for postal:* (deleted {} keys).",
        deleted
    );
}

#[tokio::main]
async fn main() {
    // Load .env file
    if dotenv::from_filename(".env").is_err() {
        // Try loading from crawler directory if running from workspace root
        dotenv::from_filename("crawler/.env").ok();
    }
    let zip_code_url = std::env::var("ZIP_CODE_URL").expect("ZIP_CODE_URL not set");
    // Default sleep duration: 24 hours (in seconds)
    let sleep_seconds: u64 = std::env::var("CRAWLER_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "86400".to_string())
        .parse()
        .expect("CRAWLER_INTERVAL_SECONDS must be a number");
    let run_once = std::env::var("CRAWLER_RUN_ONCE")
        .map(|v| {
            let value = v.to_ascii_lowercase();
            value == "1" || value == "true" || value == "yes"
        })
        .unwrap_or(false);

    let tmp_path_name = format!("{}/{}", temp_dir().to_str().unwrap(), "utf_ken_all.zip");
    let in_optimize_file_path_name = format!(
        "{}/{}",
        temp_dir().to_str().unwrap(),
        "utf_ken_all_optimize.zip"
    );
    let out_file_path = format!("{}/{}", temp_dir().to_str().unwrap(), "utf_ken_all.csv");

    loop {
        tlog!("Starting crawler cycle...");
        let run_started_at = chrono::Utc::now();
        let batch_now = chrono::Utc::now().naive_utc();
        let batch_timestamp = batch_now
            .with_nanosecond(0)
            .expect("failed to normalize batch timestamp");
        let data_version = build_data_version(batch_now);
        tlog!("Batch timestamp: {:?}", batch_timestamp);
        tlog!("Data version: {}", data_version);

        // file download
        tlog!("{}", &zip_code_url);

        if let Err(e) =
            file::download::fetch_stream(&tmp_path_name, &in_optimize_file_path_name, &zip_code_url)
                .await
        {
            eprintln!("Failed to download: {:?}", e);
            tlog!("Retrying in {} seconds...", sleep_seconds);
            sleep(Duration::from_secs(sleep_seconds)).await;
            continue;
        }

        // file unfreeze
        if let Err(e) = file::unfreeze::unzip(&in_optimize_file_path_name, &out_file_path) {
            eprintln!("Failed to unzip: {:?}", e);
            tlog!("Retrying in {} seconds...", sleep_seconds);
            sleep(Duration::from_secs(sleep_seconds)).await;
            continue;
        }
        // postal code csv file format
        let csv_map = match file::parse::csv::csv_stream_format(&out_file_path, false).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error reading CSV file: {:?}", e);
                tlog!("Retrying in {} seconds...", sleep_seconds);
                sleep(Duration::from_secs(sleep_seconds)).await;
                continue;
            }
        };

        // Read DATABASE_TYPE from environment (default: postgres)
        let database_type =
            std::env::var("DATABASE_TYPE").unwrap_or_else(|_| "postgres".to_string());
        tlog!("Using database type: {}", database_type);
        let records_in_feed = csv_map.len() as i64;
        let mut data_updated = false;

        // MySQL connection and insertion (only if DATABASE_TYPE is mysql)
        if database_type == "mysql" {
            let mysql_pool = match db::connection::mysql_connection().await {
                Ok(pool) => {
                    println!("MySQL connected");
                    pool
                }
                Err(e) => {
                    eprintln!("Error connecting to MySQL: {:?}", e);
                    tlog!("Retrying in {} seconds...", sleep_seconds);
                    sleep(Duration::from_secs(sleep_seconds)).await;
                    continue;
                }
            };

            if let Err(e) = db::audit::ensure_audit_table_mysql(&mysql_pool).await {
                eprintln!("Error preparing MySQL audit table: {:?}", e);
            }
            if let Err(e) = db::audit::ensure_snapshot_table_mysql(&mysql_pool).await {
                eprintln!("Error preparing MySQL snapshot table: {:?}", e);
            }

            let mut audit_record = DataUpdateAuditRecord {
                data_version: data_version.clone(),
                source_url: zip_code_url.clone(),
                run_started_at,
                run_finished_at: chrono::Utc::now(),
                batch_timestamp,
                records_in_feed,
                inserted_count: 0,
                updated_count: 0,
                deleted_count: 0,
                total_count: 0,
                status: "failed".to_string(),
                error_message: None,
            };

            match db::insert_postal_code_mysql::bulk_insert(&mysql_pool, &csv_map, batch_timestamp)
                .await
            {
                Err(e) => {
                    eprintln!("Error inserting data into MySQL: {:?}", e);
                    audit_record.error_message = Some(format!("bulk_insert: {e}"));
                }
                Ok(()) => {
                    tlog!("Data inserted into MySQL successfully.");
                    data_updated = true;

                    let deleted_count =
                        match db::insert_postal_code_mysql::delete_old_records_mysql(
                            &mysql_pool,
                            batch_timestamp,
                        )
                        .await
                        {
                            Ok(count) => count as i64,
                            Err(e) => {
                                eprintln!("Error deleting old records from MySQL: {:?}", e);
                                audit_record.error_message =
                                    Some(format!("delete_old_records: {e}"));
                                audit_record.run_finished_at = chrono::Utc::now();
                                if let Err(log_err) =
                                    db::audit::insert_audit_mysql(&mysql_pool, &audit_record).await
                                {
                                    eprintln!("Error inserting MySQL audit log: {:?}", log_err);
                                }
                                continue;
                            }
                        };

                    match db::audit::compute_mysql_diff_counts(&mysql_pool, batch_timestamp).await {
                        Ok((inserted_count, updated_count, total_count)) => {
                            audit_record.inserted_count = inserted_count;
                            audit_record.updated_count = updated_count;
                            audit_record.deleted_count = deleted_count;
                            audit_record.total_count = total_count;
                            audit_record.status = "success".to_string();
                            if let Err(e) =
                                db::audit::create_mysql_snapshot(&mysql_pool, &data_version).await
                            {
                                eprintln!("Error creating MySQL snapshot: {:?}", e);
                                audit_record.status = "failed".to_string();
                                audit_record.error_message = Some(format!("create_snapshot: {e}"));
                            }
                            tlog!(
                                "MySQL audit summary: inserted={}, updated={}, deleted={}, total={}",
                                inserted_count,
                                updated_count,
                                deleted_count,
                                total_count
                            );
                        }
                        Err(e) => {
                            eprintln!("Error computing MySQL audit counts: {:?}", e);
                            audit_record.error_message = Some(format!("compute_diff_counts: {e}"));
                        }
                    }
                }
            }

            audit_record.run_finished_at = chrono::Utc::now();
            if let Err(e) = db::audit::insert_audit_mysql(&mysql_pool, &audit_record).await {
                eprintln!("Error inserting MySQL audit log: {:?}", e);
            }
        }

        // PostgreSQL connection and insertion (only if DATABASE_TYPE is postgres)
        if database_type == "postgres" {
            let postgres_pool = match db::connection::postgres_connection().await {
                Ok(pool) => {
                    tlog!("PostgreSQL connected");
                    pool
                }
                Err(e) => {
                    eprintln!("Error connecting to PostgreSQL: {:?}", e);
                    tlog!("Retrying in {} seconds...", sleep_seconds);
                    sleep(Duration::from_secs(sleep_seconds)).await;
                    continue;
                }
            };

            if let Err(e) = db::audit::ensure_audit_table_postgres(&postgres_pool).await {
                eprintln!("Error preparing PostgreSQL audit table: {:?}", e);
            }
            if let Err(e) = db::audit::ensure_snapshot_table_postgres(&postgres_pool).await {
                eprintln!("Error preparing PostgreSQL snapshot table: {:?}", e);
            }

            let mut audit_record = DataUpdateAuditRecord {
                data_version: data_version.clone(),
                source_url: zip_code_url.clone(),
                run_started_at,
                run_finished_at: chrono::Utc::now(),
                batch_timestamp,
                records_in_feed,
                inserted_count: 0,
                updated_count: 0,
                deleted_count: 0,
                total_count: 0,
                status: "failed".to_string(),
                error_message: None,
            };

            match db::insert_postal_code_postgres::bulk_insert_async(
                &postgres_pool,
                &csv_map,
                batch_timestamp,
            )
            .await
            {
                Err(e) => {
                    eprintln!("Error inserting data into PostgreSQL: {:?}", e);
                    audit_record.error_message = Some(format!("bulk_insert: {e}"));
                }
                Ok(()) => {
                    tlog!("Data inserted into PostgreSQL successfully.");
                    data_updated = true;

                    let deleted_count =
                        match db::insert_postal_code_postgres::delete_old_records_postgres(
                            &postgres_pool,
                            batch_timestamp,
                        )
                        .await
                        {
                            Ok(count) => count as i64,
                            Err(e) => {
                                eprintln!("Error deleting old records from PostgreSQL: {:?}", e);
                                audit_record.error_message =
                                    Some(format!("delete_old_records: {e}"));
                                audit_record.run_finished_at = chrono::Utc::now();
                                if let Err(log_err) =
                                    db::audit::insert_audit_postgres(&postgres_pool, &audit_record)
                                        .await
                                {
                                    eprintln!(
                                        "Error inserting PostgreSQL audit log: {:?}",
                                        log_err
                                    );
                                }
                                continue;
                            }
                        };

                    match db::audit::compute_postgres_diff_counts(&postgres_pool, batch_timestamp)
                        .await
                    {
                        Ok((inserted_count, updated_count, total_count)) => {
                            audit_record.inserted_count = inserted_count;
                            audit_record.updated_count = updated_count;
                            audit_record.deleted_count = deleted_count;
                            audit_record.total_count = total_count;
                            audit_record.status = "success".to_string();
                            if let Err(e) =
                                db::audit::create_postgres_snapshot(&postgres_pool, &data_version)
                                    .await
                            {
                                eprintln!("Error creating PostgreSQL snapshot: {:?}", e);
                                audit_record.status = "failed".to_string();
                                audit_record.error_message = Some(format!("create_snapshot: {e}"));
                            }
                            tlog!(
                                "PostgreSQL audit summary: inserted={}, updated={}, deleted={}, total={}",
                                inserted_count,
                                updated_count,
                                deleted_count,
                                total_count
                            );
                        }
                        Err(e) => {
                            eprintln!("Error computing PostgreSQL audit counts: {:?}", e);
                            audit_record.error_message = Some(format!("compute_diff_counts: {e}"));
                        }
                    }
                }
            }

            audit_record.run_finished_at = chrono::Utc::now();
            if let Err(e) = db::audit::insert_audit_postgres(&postgres_pool, &audit_record).await {
                eprintln!("Error inserting PostgreSQL audit log: {:?}", e);
            }
        }

        if data_updated {
            invalidate_redis_cache().await;
        }

        if run_once {
            tlog!("CRAWLER_RUN_ONCE enabled. Exiting after one completed cycle.");
            break;
        }

        tlog!(
            "Crawler cycle completed. Sleeping for {} seconds...",
            sleep_seconds
        );
        sleep(Duration::from_secs(sleep_seconds)).await;
    }
}
