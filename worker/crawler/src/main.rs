mod constants;
mod db;
mod file;
mod utils;
use constants::temp_dir;

#[tokio::main]
async fn main() {
    dotenv::from_filename(".env").ok();
    let zip_code_url = std::env::var("ZIP_CODE_URL").expect("ZIP_CODE_URL not set");
    let tmp_path_name = format!("{}/{}", temp_dir().to_str().unwrap(), "utf_ken_all.zip");
    let in_optimize_file_path_name = format!(
        "{}/{}",
        temp_dir().to_str().unwrap(),
        "utf_ken_all_optimize.zip"
    );
    let out_file_path = format!("{}/{}", temp_dir().to_str().unwrap(), "utf_ken_all.csv");

    // file download
    tlog!("{}", &zip_code_url);

    if let Err(e) =
        file::download::fetch_stream(&tmp_path_name, &in_optimize_file_path_name, &zip_code_url)
            .await
    {
        eprintln!("Failed to download: {:?}", e);
        return;
    }

    // file unfreeze
    if let Err(e) = file::unfreeze::unzip(&in_optimize_file_path_name, &out_file_path) {
        eprintln!("Failed to unzip: {:?}", e);
        return;
    }
    // postal code csv file format
    let csv_map = match file::parse::csv::csv_stream_format(&out_file_path, false).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading CSV file: {:?}", e);
            return;
        }
    };

    // MySQL connection and error check
    let mysql_pool = match db::connection::mysql_connection().await {
        Ok(pool) => {
            println!("MySQL connected");
            pool
        }
        Err(e) => {
            eprintln!("Error connecting to MySQL: {:?}", e);
            return;
        }
    };

    // PostgreSQL connection and error check
    let postgres_pool = match db::connection::postgres_connection().await {
        Ok(pool) => {
            tlog!("PostgreSQL connected");
            pool
        }
        Err(e) => {
            eprintln!("Error connecting to PostgreSQL: {:?}", e);
            return;
        }
    };

    // Mysql insert data and error check
    if let Err(e) = db::insert_postal_code_mysql::bulk_insert(&mysql_pool, &csv_map).await {
        eprintln!("Error inserting data into MySQL: {:?}", e);
    } else {
        tlog!("Data inserted into MySQL successfully.");
    }

    // PostgreSQL insert data and error check
    if let Err(e) =
        db::insert_postal_code_postgres::bulk_insert_async(&postgres_pool, &csv_map).await
    {
        eprintln!("Error inserting data into PostgreSQL: {:?}", e);
    } else {
        tlog!("Data inserted into PostgreSQL successfully.");
    }
}
