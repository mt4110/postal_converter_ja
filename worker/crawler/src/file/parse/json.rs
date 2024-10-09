use common::serde_json;
use serde_json::Value;
use tokio::io::{self, AsyncReadExt};

// Read JSON file asynchronously
pub async fn json_parse(file_path: &str) -> io::Result<serde_json::Value> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    let json: Value = serde_json::from_slice(&contents)?;
    Ok(json)
}
