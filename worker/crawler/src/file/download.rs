use futures_util::stream::StreamExt;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::{self, AsyncWriteExt, BufReader, BufWriter}; // インポート追加

async fn fetch_url(target_url: &str, tmp_path: &str) -> io::Result<()> {
    let response = reqwest::get(target_url)
        .await
        .expect("Failed to download the file");

    if response.status().is_success() {
        let mut file = File::create(tmp_path).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to read chunk: {:?}", e);
                    break;
                }
            };
            println!("Chunk size: {}", chunk.len());
            file.write_all(&chunk).await?;
            println!("Chunk written to file.");
        }

        println!("Download completed.");
    } else {
        eprintln!("Failed to download file: {:?}", response.status());
    }

    Ok(())
}

async fn process_large_file(tmp_path: &str, output_path: &str) -> io::Result<()> {
    println!("Processing large file start...: {}", tmp_path);
    let file = File::open(tmp_path).await?;
    println!("File open finish");
    let mut reader = BufReader::new(file);
    let mut writer = BufWriter::new(File::create(output_path).await?);
    println!("BufReader and BufWriter finish");

    let metadata = tokio::fs::metadata(tmp_path).await?;
    let file_size = metadata.len();
    println!("File size after download: {}", file_size);

    let mut buffer = [0; 1024];
    let mut total_read = 0;

    while total_read < file_size {
        let n = reader.read(&mut buffer).await?; // バッファにデータを読み込む
        if n == 0 {
            println!(" reader read finish");
            break;
        } else {
            println!(" {} bytes read from file", n); // 読み込んだバイト数を表示
        }

        writer.write_all(&buffer[..n]).await?; // 書き込む
        total_read += n as u64; // 読み込んだバイト数を更新
        println!(" writer write_all finish, {} bytes written", total_read);
    }

    writer.flush().await?;
    tokio::fs::remove_file(tmp_path).await?;
    println!("Async download completed.");
    Ok(())
}

pub async fn fetch_stream(in_path: &str, output_path: &str, target_url: &str) -> io::Result<()> {
    fetch_url(target_url, in_path).await?;
    process_large_file(in_path, output_path).await
}
