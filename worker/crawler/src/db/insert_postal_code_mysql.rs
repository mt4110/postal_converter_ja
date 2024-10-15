use crate::constants::PostalCode;
use mysql_async::{params, prelude::Queryable, Pool};
use tokio::task;
use tokio::time::{sleep, Duration};

async fn retry_transaction<F, Fut, T>(
    max_retries: usize,
    delay: Duration,
    mut f: F,
) -> Result<T, mysql_async::Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, mysql_async::Error>>,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= max_retries {
                    return Err(e);
                }
                if let mysql_async::Error::Server(ref err) = e {
                    if err.code == 1213 {
                        // Deadlock
                        attempt += 1;
                        sleep(delay).await;
                        continue;
                    }
                }
                return Err(e);
            }
        }
    }
}

pub async fn bulk_insert(pool: &Pool, data: &[PostalCode]) -> Result<(), mysql_async::Error> {
    let chunk_size = 200;
    println!("Inserting {} records", data.len());

    let mut handles = Vec::new();
    for chunk in data.chunks(chunk_size) {
        let pool_clone = pool.clone();
        let chunk_data = chunk.to_vec();

        let handle = task::spawn(async move {
            println!("Chunk size: {}", chunk_data.len());

            let query = r"INSERT INTO postal_codes (zip_code, prefecture_id, prefecture, city, town)
              VALUES (:zip_code, :prefecture_id, :prefecture, :city, :town)
              ON DUPLICATE KEY UPDATE
              prefecture_id = VALUES(prefecture_id),
              prefecture = VALUES(prefecture),
              city = VALUES(city),
              town = VALUES(town)";

            retry_transaction(3, Duration::from_millis(500), || {
                let params: Vec<_> = chunk_data
                    .iter()
                    .map(|d| {
                        params! {
                            "zip_code" => &d.zip_code,
                            "prefecture_id" => &d.prefecture_id,
                            "prefecture" => &d.prefecture.trim(),
                            "city" => &d.city.trim(),
                            "town" => d.town.as_deref().unwrap_or("").trim(),
                        }
                    })
                    .collect();
                let conn_clone = pool_clone.clone();
                async move {
                    let mut conn = conn_clone.get_conn().await?;
                    let mut tx = conn.start_transaction(Default::default()).await?;
                    tx.exec_batch(query, params.clone()).await?;
                    tx.commit().await
                }
            })
            .await
            .map_err(|e| {
                eprintln!("Transaction failed: {:?}", e);
                e
            })?;

            sleep(Duration::from_millis(500)).await; // 次のタスクの前に少し待機
            println!(
                "Transaction committed for chunk of size: {}",
                chunk_data.len()
            );

            Ok::<_, mysql_async::Error>(())
        });

        handles.push(handle);
    }

    for handle in handles {
        match handle.await {
            Ok(result) => match result {
                Ok(()) => {} // 成功
                Err(e) => return Err(e),
            },
            Err(join_error) => {
                eprintln!("Task failed: {:?}", join_error);
                return Err(mysql_async::Error::Io(mysql_async::IoError::Io(
                    std::io::Error::new(std::io::ErrorKind::Other, join_error.to_string()),
                )));
            }
        }
    }

    Ok(())
}
