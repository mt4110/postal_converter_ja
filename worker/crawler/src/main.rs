mod constants;
mod db;
mod file;
mod utils;
use constants::temp_dir;
use tokio::time::{sleep, Duration};

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

    let tmp_path_name = format!("{}/{}", temp_dir().to_str().unwrap(), "utf_ken_all.zip");
    let in_optimize_file_path_name = format!(
        "{}/{}",
        temp_dir().to_str().unwrap(),
        "utf_ken_all_optimize.zip"
    );
    let out_file_path = format!("{}/{}", temp_dir().to_str().unwrap(), "utf_ken_all.csv");

    loop {
        tlog!("Starting crawler cycle...");
        let batch_timestamp = chrono::Local::now().naive_local();
        tlog!("Batch timestamp: {:?}", batch_timestamp);

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

            if let Err(e) =
                db::insert_postal_code_mysql::bulk_insert(&mysql_pool, &csv_map, batch_timestamp)
                    .await
            {
                eprintln!("Error inserting data into MySQL: {:?}", e);
            } else {
                tlog!("Data inserted into MySQL successfully.");
                if let Err(e) = db::insert_postal_code_mysql::delete_old_records_mysql(
                    &mysql_pool,
                    batch_timestamp,
                )
                .await
                {
                    eprintln!("Error deleting old records from MySQL: {:?}", e);
                }
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

            if let Err(e) = db::insert_postal_code_postgres::bulk_insert_async(
                &postgres_pool,
                &csv_map,
                batch_timestamp,
            )
            .await
            {
                eprintln!("Error inserting data into PostgreSQL: {:?}", e);
            } else {
                tlog!("Data inserted into PostgreSQL successfully.");
                if let Err(e) = db::insert_postal_code_postgres::delete_old_records_postgres(
                    &postgres_pool,
                    batch_timestamp,
                )
                .await
                {
                    eprintln!("Error deleting old records from PostgreSQL: {:?}", e);
                }
            }
        }

        tlog!(
            "Crawler cycle completed. Sleeping for {} seconds...",
            sleep_seconds
        );
        sleep(Duration::from_secs(sleep_seconds)).await;
    }
}
