use common::serde_json;
use csv_async::AsyncReaderBuilder;
use futures::io::AllowStdIo;
use futures::stream::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::BufReader;

// CSVファイルのレコードをフォーマット
async fn format_csv_record(record: VecDeque<String>) -> HashMap<String, String> {
    let mut formatted_record = HashMap::new();

    // 郵便番号のフォーマットを追加
    let zip_code = record.get(2).unwrap(); // 仮に3番目が郵便番号とする
    formatted_record.insert("zip_code".to_string(), format_zip_code(zip_code));

    // 他のフィールドも必要に応じてフォーマットして追加
    formatted_record.insert("city".to_string(), record.get(3).unwrap().to_string());

    // 都道府県IDの取得（仮の処理）
    let prefecture = record.get(1).unwrap(); // 仮に2番目が都道府県名とする
    formatted_record.insert("pref_id".to_string(), get_pref_id(prefecture));

    formatted_record
}

// 郵便番号のフォーマット関数
fn format_zip_code(zip_code: &str) -> String {
    format!("{}-{}", &zip_code[..3], &zip_code[3..])
}

// 都道府県IDを取得する（適宜実装）
fn get_pref_id(prefecture: &str) -> String {
    // 紐付け処理を書く
    prefecture.to_string() // 仮の処理
}

// CSVファイルをパースし、フォーマットを適用
pub async fn csv_stream_format(
    file_path: &str,
    pref_obj: tokio::io::Result<serde_json::Value>,
    is_header: bool,
) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    println!("CSV Stream Formatting start... {:?}", pref_obj);
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut csv_reader = AsyncReaderBuilder::new()
        .has_headers(is_header)
        .create_reader(AllowStdIo::new(reader));

    let mut records_vec: Vec<HashMap<String, String>> = Vec::new();

    let mut records = csv_reader.into_records();
    while let Some(result) = records.next().await {
        let record = result?;
        // StringRecord を VecDeque<String> に変換
        let deque: VecDeque<String> = record.iter().map(|s| s.to_string()).collect();
        let formatted_record = format_csv_record(deque).await; // VecDeque<String>を渡す
        records_vec.push(formatted_record);
    }

    Ok(records_vec)
}
