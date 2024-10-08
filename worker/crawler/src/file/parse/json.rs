use common::serde_json;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

// Read JSON file asynchronously
pub async fn json_parse(file_path: &str) -> io::Result<serde_json::Value> {
    let metadata = tokio::fs::metadata(file_path).await?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "The path is not a file.",
        ));
    }

    let mut file = File::open(file_path).await?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;

    let value: serde_json::Value = serde_json::from_slice(&contents).expect("Failed to parse JSON");

    println!("Parse JSON completed.");
    Ok(value)
}
