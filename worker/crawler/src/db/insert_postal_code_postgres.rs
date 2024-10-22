use crate::constants::PostalCode;
use crate::db::query_builder::build_pg_bulk_insert_query;
use crate::tlog;
use crate::utils::thread::determine_thread_num;
use deadpool_postgres::{Pool as PgPool, PoolError};
use futures::future::join_all;
use tokio::time::{sleep, Duration};
use tokio_postgres::Error as PgError;

const MAX_RETRIES: usize = 3;

async fn db_client(pool: &PgPool) -> Result<deadpool_postgres::Client, PoolError> {
    pool.get().await
}

fn to_sql_param<T: tokio_postgres::types::ToSql + Sync>(
    value: &T,
) -> &(dyn tokio_postgres::types::ToSql + Sync) {
    value as &(dyn tokio_postgres::types::ToSql + Sync)
}

async fn bulk_insert(pool: &PgPool, data: &[PostalCode]) -> Result<(), PgError> {
    let chunk_size = 200;
    let columns: &[&str] = &["zip_code", "prefecture_id", "prefecture", "city", "town"];
    let mut client = db_client(pool)
        .await
        .expect("Failed to get a client from the pool");

    tlog!("Data length: {}", data.len());
    for chunk in data.chunks(chunk_size) {
        let mut retries = 0;

        let tx = client.transaction().await?;

        // Collect the parameters for each postal code
        let insert_data: Vec<Vec<&(dyn tokio_postgres::types::ToSql + Sync)>> = chunk
            .iter()
            .map(|d| {
                vec![
                    to_sql_param(&d.zip_code),
                    to_sql_param(&d.prefecture_id),
                    to_sql_param(&d.prefecture),
                    to_sql_param(&d.city),
                    to_sql_param(&d.town),
                ]
            })
            .collect();

        let (query, params) = build_pg_bulk_insert_query("postal_codes", columns, &insert_data);
        // tlog!("Params: {:?}", params);

        // Execute the transaction and asynchronously wait for the result
        if let Err(e) = tx.execute(&query, &params).await {
            if retries == MAX_RETRIES {
                eprintln!("Max retries reached. Rolling back transaction. {:?}", e);
                tx.rollback().await?;
                return Err(e);
            }
            if let Some(pg_err) = e.as_db_error() {
                if pg_err.code().code() == "40P01" && retries < MAX_RETRIES {
                    eprintln!("Deadlock detected, retrying... Attempt {}", retries + 1,);
                    retries += 1;
                    tlog!("Retrying... Current attempt: {}", retries + 1);
                    tlog!("Retrying query: {}", query);
                    sleep(Duration::from_millis(200)).await;
                    continue;
                }
            }
            tx.rollback().await?;
            return Err(e);
        }
        tlog!(
            "Attempting to commit transaction for chunk size: {}",
            chunk.len(),
        );
        tx.commit().await?;
        tlog!("Transaction committed successfully");
        sleep(Duration::from_millis(200)).await;
    }
    Ok(())
}

pub async fn bulk_insert_async(pool: &PgPool, data: &[PostalCode]) -> Result<(), PgError> {
    let thread_num = determine_thread_num();
    let chunk_size = data.len() / thread_num;
    tlog!("Using {} threads for bulk insert", thread_num);

    let tasks: Vec<_> = (0..thread_num)
        .map(|i| {
            let start_index = i * chunk_size;
            let end_index = if i == thread_num - 1 {
                data.len()
            } else {
                (i + 1) * chunk_size
            };

            let chunk = &data[start_index..end_index];
            tlog!(
                "Task {} processing range {} to {}",
                i,
                start_index,
                end_index
            );

            async move {
                tlog!("Task {} running", i);
                bulk_insert(pool, chunk).await
            }
        })
        .collect();

    let results = join_all(tasks).await;

    for result in results {
        if let Err(e) = result {
            eprintln!("Task failed with error: {:?}", e);
            return Err(e);
        }
    }

    Ok(())
}
