use crate::constants::common_path;
use crate::file;
use common::models::PostalCode;
use csv_async::AsyncReaderBuilder;
use futures::stream::StreamExt;
use std::collections::{HashMap, VecDeque};

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

fn remove_parentheses(s: &str) -> String {
    let mut result = String::new();
    let mut depth = 0;
    for c in s.chars() {
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth = std::cmp::max(0, depth - 1);
        } else if depth == 0 {
            result.push(c);
        }
    }
    result
}

//  Format CSV file records
fn format_csv_record_with_cache(
    record: VecDeque<String>,
    pref_cache: &HashMap<String, String>,
    replace_cache: &HashMap<char, &str>,
) -> (PostalCode, bool) {
    let city_id = record.front().cloned().unwrap_or_else(|| "".to_string());
    let zip_code = record.get(2).cloned().unwrap_or_else(|| "".to_string());
    let prefecture = record.get(6).map_or_else(
        || "".to_string(),
        |s| replace_japanese_to_alphanumeric_with_cache(s, replace_cache),
    );
    let prefecture_id = pref_cache
        .get(&prefecture)
        .and_then(|s| s.parse::<i16>().ok())
        .unwrap_or(0);
    let city = record.get(7).map_or_else(
        || "".to_string(),
        |s| replace_japanese_to_alphanumeric_with_cache(s, replace_cache),
    );
    let raw_town = record
        .get(8)
        .map(|s| replace_japanese_to_alphanumeric_with_cache(s, replace_cache))
        .unwrap_or_default();

    // Remove parentheses content (e.g., "銀座(1丁目)" -> "銀座")
    // Note: Full-width parentheses '（）' are already converted to half-width '()'
    // by replace_japanese_to_alphanumeric_with_cache above.
    let town = remove_parentheses(&raw_town);

    // Column 12 (Index 12, 13th column) indicates "One town has multiple zip codes" (1=yes, 0=no)
    // The user suggests using this flag to determine split lines.
    // If 0, it likely means the town is simple and if split across lines, it should be merged.
    let is_multi_town = record
        .get(12) // Access Index 12 directly
        .map(|s| s == "1")
        .unwrap_or(false);

    (
        PostalCode {
            zip_code,
            prefecture_id,
            city_id,
            prefecture,
            city,
            town: if town == "以下に掲載がない場合" {
                "".to_string()
            } else {
                town
            },
        },
        is_multi_town,
    )
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
    let content = tokio::fs::read(file_path).await?;
    let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&content);
    let decoded_string = decoded.into_owned();

    let csv_reader = AsyncReaderBuilder::new()
        .has_headers(is_header)
        .create_reader(decoded_string.as_bytes());

    let pref_cache = build_prefecture_cache().await;
    let replace_cache = build_replace_cache();

    let mut records_vec: Vec<PostalCode> = Vec::new();
    let mut records = csv_reader.into_records();
    let mut prev_record: Option<PostalCode> = None;
    let mut prev_is_multi_town = false;

    while let Some(result) = records.next().await {
        match result {
            Ok(record) => {
                let deque: VecDeque<String> = record.iter().map(|s| s.to_string()).collect();
                let (current, is_multi_town) =
                    format_csv_record_with_cache(deque, &pref_cache, &replace_cache);

                if let Some(ref mut prev) = prev_record {
                    // Merge logic:
                    // 1. Same Zip Code and City ID
                    // 2. AND 'is_multi_town' flag is 0 for BOTH records (Index 12)
                    if prev.zip_code == current.zip_code
                        && prev.city_id == current.city_id
                        && !prev_is_multi_town
                        && !is_multi_town
                    {
                        prev.town.push_str(&current.town);
                        // Continue to next record, keeping 'prev' as accumulator
                        continue;
                    } else {
                        // Different record or distinct towns, push previous
                        records_vec.push(prev.clone());
                    }
                }
                // Update prev_record
                prev_record = Some(current);
                prev_is_multi_town = is_multi_town;
            }
            Err(e) => eprintln!("Error processing record: {:?}", e),
        }
    }

    // Push the last record if exists
    if let Some(last) = prev_record {
        records_vec.push(last);
    }

    // Deduplicate records based on Primary Key (zip_code, prefecture_id, city, town)
    // to prevent "ON CONFLICT DO UPDATE command cannot affect row a second time" error.
    // This error occurs when a single batch contains multiple records with the same PK.
    let mut seen = std::collections::HashSet::new();
    records_vec.retain(|r| {
        let key = (
            r.zip_code.clone(),
            r.prefecture_id,
            r.city.clone(),
            r.town.clone(),
        );
        seen.insert(key)
    });

    Ok(records_vec)
}
