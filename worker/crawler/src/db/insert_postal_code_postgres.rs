use crate::constants::PostalCode;
use deadpool_postgres::Pool;
use tokio::time::{sleep, Duration};
use tokio_postgres::{Client, Error as PgError};

const MAX_RETRIES: usize = 3;

pub async fn bulk_insert(pool: &Pool, data: &[PostalCode]) -> Result<(), PgError> {
    let chunk_size = 200;
    for chunk in data.chunks(chunk_size) {
        let mut retries = 0;
        loop {
            let mut client = pool.get().await.unwrap();
            let tx = client.transaction().await?;
            let mut query = r#"
                INSERT INTO postal_codes (zip_code, prefecture_id, prefecture, city, town)
                VALUES "#
                .to_string();

            let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
            for (i, d) in chunk.iter().enumerate() {
                query.push_str(&format!(
                    "(${}, ${}, ${}, ${}, ${})",
                    i * 5 + 1,
                    i * 5 + 2,
                    i * 5 + 3,
                    i * 5 + 4,
                    i * 5 + 5
                ));
                if i < chunk.len() - 1 {
                    query.push_str(", ");
                }
                params.push(&d.zip_code);
                params.push(&d.prefecture_id);
                params.push(&d.prefecture);
                params.push(&d.city);
                params.push(&d.town);
            }

            query.push_str(
                " ON CONFLICT (zip_code) DO UPDATE SET \
                prefecture_id = EXCLUDED.prefecture_id, \
                prefecture = EXCLUDED.prefecture, \
                city = EXCLUDED.city, \
                town = EXCLUDED.town",
            );

            if let Err(e) = tx.execute(&query, &params).await {
                if let Some(pg_err) = e.as_db_error() {
                    if pg_err.code().code() == "40P01" && retries < MAX_RETRIES {
                        eprintln!("Deadlock detected, retrying... Attempt {}", retries + 1);
                        retries += 1;
                        sleep(Duration::from_millis(500)).await;
                        continue;
                    }
                }
                tx.rollback().await?;
                return Err(e);
            }

            tx.commit().await?;
            break;
        }
    }
    Ok(())
}
