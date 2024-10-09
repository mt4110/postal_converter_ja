use crate::constants;
use crate::file;
use constants::common_path;
use csv_async::AsyncReaderBuilder;
use futures::io::AllowStdIo;
use futures::stream::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::BufReader;

// CSVファイルのレコードをフォーマット
fn format_csv_record(
    record: VecDeque<String>,
    pref_cache: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut formatted_record = HashMap::new();
    let zip_code = record.get(2).unwrap();
    formatted_record.insert("zip_code".to_string(), format_zip_code(zip_code));
    let prefecture = record.get(6).unwrap();
    let prefecture_id = pref_cache.get(prefecture).cloned().unwrap_or_default();
    formatted_record.insert("prefecture_id".to_string(), prefecture_id);
    formatted_record.insert("prefecture".to_string(), prefecture.to_string());
    formatted_record.insert("city".to_string(), record.get(7).unwrap().to_string());
    formatted_record.insert("town".to_string(), record.get(8).unwrap().to_string());

    formatted_record
}

// 郵便番号のフォーマット関数
fn format_zip_code(zip_code: &str) -> String {
    format!("{}-{}", &zip_code[..3], &zip_code[3..])
}

// 都道府県 ID キャッシュを事前に作成
pub async fn build_prefecture_cache() -> HashMap<String, String> {
    let mut pref_map = HashMap::new();
    let pref_json = format!("{}/{}", common_path().to_str().unwrap(), "pref.json");
    let pref_json_map = file::parse::json::json_parse(&pref_json).await.unwrap();

    if let Some(pref_list) = pref_json_map.as_array() {
        for pref in pref_list {
            if let Some(label) = pref.get("label").and_then(|v| v.as_str()) {
                if let Some(id) = pref.get("id").map(|id| id.to_string()) {
                    pref_map.insert(label.to_string(), id);
                }
            }
        }
    }
    pref_map
}

// get prefecture id
pub async fn get_prefecture_id(prefecture: &str) -> Option<String> {
    let pref_json = format!("{}/{}", common_path().to_str().unwrap(), "pref.json");
    let pref_json_map = file::parse::json::json_parse(&pref_json).await.ok()?;
    pref_json_map
        .as_array()?
        .iter()
        .find(|&pref| pref.get("label").and_then(|v| v.as_str()).unwrap_or("") == prefecture)
        .and_then(|pref| pref.get("id").map(|id| id.to_string()))
}

// CSVファイルをパースし、フォーマットを適用
pub async fn csv_stream_format(
    file_path: &str,
    is_header: bool,
) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut csv_reader = AsyncReaderBuilder::new()
        .has_headers(is_header)
        .create_reader(AllowStdIo::new(reader));

    // キャッシュを事前に作成
    let pref_cache = build_prefecture_cache().await;

    let mut records_vec: Vec<HashMap<String, String>> = Vec::new();
    let mut records = csv_reader.into_records();
    while let Some(result) = records.next().await {
        let record = result?;
        let deque: VecDeque<String> = record.iter().map(|s| s.to_string()).collect();
        let formatted_record = format_csv_record(deque, &pref_cache);
        records_vec.push(formatted_record);
    }
    Ok(records_vec)
}
