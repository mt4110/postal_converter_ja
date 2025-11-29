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
    let mut in_parens = false;
    for c in s.chars() {
        if c == '(' {
            in_parens = true;
        } else if c == ')' {
            in_parens = false;
        } else if !in_parens {
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
        .and_then(|s| s.parse::<i32>().ok())
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
    let town = remove_parentheses(&raw_town);

    // Column 9 indicates if one zip code represents multiple town areas (1 = yes, 0 = no)
    // If 0, it means the zip code corresponds to a single town area (which might be split across lines)
    // If 1, it means the zip code covers multiple distinct towns, so we should NOT merge them.
    let multi_town_in_zip = record
        .get(8) // Index 8 is actually town name, wait. CSV index is 0-based.
        // 0: JIS code, 1: old zip, 2: zip, 3: pref kana, 4: city kana, 5: town kana
        // 6: pref, 7: city, 8: town
        // 9: multi-town flag (1=yes, 0=no)
        // 10: koaza flag
        // 11: chome flag
        // 12: multi-zip for one town flag
        // 13: update status
        // 14: update reason
        .and_then(|_| record.get(9)) // Access Index 9 correctly
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
        multi_town_in_zip,
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
    let mut prev_multi_town_in_zip = false;

    while let Some(result) = records.next().await {
        match result {
            Ok(record) => {
                let deque: VecDeque<String> = record.iter().map(|s| s.to_string()).collect();
                let (current, multi_town_in_zip) =
                    format_csv_record_with_cache(deque, &pref_cache, &replace_cache);

                if let Some(ref mut prev) = prev_record {
                    // Merge logic:
                    // 1. Same Zip Code and City ID
                    // 2. AND 'multi_town_in_zip' flag is 0 for BOTH records (Index 9)
                    //    If flag is 1, it means multiple distinct towns share the zip, so NO merge.
                    //    If flag is 0, it implies a single town entity, so split lines should be merged.
                    if prev.zip_code == current.zip_code
                        && prev.city_id == current.city_id
                        && !prev_multi_town_in_zip
                        && !multi_town_in_zip
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
                prev_multi_town_in_zip = multi_town_in_zip;
            }
            Err(e) => eprintln!("Error processing record: {:?}", e),
        }
    }

    // Push the last record if exists
    if let Some(last) = prev_record {
        records_vec.push(last);
    }

    Ok(records_vec)
}
