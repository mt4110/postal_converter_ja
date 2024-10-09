mod constants;
use constants::temp_dir;

// Declare the submodule `file` and the files inside it
mod file {
    pub mod download;
    pub mod unfreeze;
    pub mod parse {
        pub mod csv;
        pub mod json;
    }
}

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
    println!("{}", &zip_code_url);

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

    // file parse & formatted
    let csv_map = file::parse::csv::csv_stream_format(&out_file_path, false).await;
    println!("{:?}", csv_map);
}
