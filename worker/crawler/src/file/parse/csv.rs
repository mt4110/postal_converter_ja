use crate::constants::{common_path, PostalCode};
use crate::file;
use csv_async::AsyncReaderBuilder;
use futures::io::AllowStdIo;
use futures::stream::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::BufReader;

// Register uppercase letters in advance in the cache map
fn build_replace_cache() -> HashMap<char, &'static str> {
    let mut map = HashMap::new();
    map.insert('（', "(");
    map.insert('）', ")");
    map.insert('ー', "-");
    map.insert('、', ",");
    map.insert('０', "0");
    map.insert('１', "1");
    map.insert('２', "2");
    map.insert('３', "3");
    map.insert('４', "4");
    map.insert('５', "5");
    map.insert('６', "6");
    map.insert('７', "7");
    map.insert('８', "8");
    map.insert('９', "9");
    map
}

fn replace_japanese_to_alphanumeric_with_cache(s: &str, cache: &HashMap<char, &str>) -> String {
    s.chars()
        .map(|c| cache.get(&c).unwrap_or(&c.to_string().as_str()).to_string())
        .collect()
}

//  Format CSV file records
fn format_csv_record_with_cache(
    record: VecDeque<String>,
    pref_cache: &HashMap<String, String>,
    replace_cache: &HashMap<char, &str>,
) -> PostalCode {
    let zip_code = record.get(2).cloned().unwrap_or_else(|| "".to_string());
    let prefecture = record.get(6).map_or_else(
        || "".to_string(),
        |s| replace_japanese_to_alphanumeric_with_cache(s, replace_cache),
    );
    let prefecture_id = pref_cache
        .get(&prefecture)
        .and_then(|id| id.parse().ok())
        .unwrap_or(0);
    let city = record.get(7).map_or_else(
        || "".to_string(),
        |s| replace_japanese_to_alphanumeric_with_cache(s, replace_cache),
    );
    let town = record
        .get(8)
        .map(|s| replace_japanese_to_alphanumeric_with_cache(s, replace_cache))
        .filter(|s| !s.is_empty());
    PostalCode {
        zip_code,
        prefecture_id,
        prefecture,
        city,
        town: match town.as_deref() {
            Some("以下に掲載がない場合") => Some("".to_string()),
            _ => town,
        },
    }
}

// make prefecture cache
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

// Parse CSV file and apply formatting
pub async fn csv_stream_format(
    file_path: &str,
    is_header: bool,
) -> Result<Vec<PostalCode>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let csv_reader = AsyncReaderBuilder::new()
        .has_headers(is_header)
        .create_reader(AllowStdIo::new(reader));

    let pref_cache = build_prefecture_cache().await;
    let replace_cache = build_replace_cache();

    let mut records_vec: Vec<PostalCode> = Vec::new();
    let mut records = csv_reader.into_records();

    while let Some(result) = records.next().await {
        match result {
            Ok(record) => {
                let deque: VecDeque<String> = record.iter().map(|s| s.to_string()).collect();
                let formatted_record =
                    format_csv_record_with_cache(deque, &pref_cache, &replace_cache);
                records_vec.push(formatted_record);
            }
            Err(e) => eprintln!("Error processing record: {:?}", e),
        }
    }
    Ok(records_vec)
}
