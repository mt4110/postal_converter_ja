use common::db;
use mysql::{params, prelude::Queryable, Pool as MySqlPool};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

fn build_test_zip(seed: u64) -> String {
    let n = (seed % 9_000_000) + 1_000_000;
    format!("{n:07}")
}

fn build_city_id(prefix: char, seed: u64) -> String {
    format!("{prefix}{:09}", seed % 1_000_000_000)
}

#[tokio::test(flavor = "current_thread")]
async fn postgres_roundtrip_insert_and_query() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("POSTGRES_DATABASE_URL").is_err() {
        eprintln!("skip postgres test: POSTGRES_DATABASE_URL is not set");
        return Ok(());
    }

    let seed = unique_seed();
    let zip_code = build_test_zip(seed);
    let prefecture_id: i16 = 13;
    let city_id = build_city_id('T', seed);
    let city = format!("integration-city-{seed}");
    let town = format!("integration-town-{seed}");
    let prefecture = "東京都".to_string();

    let pool = db::postgres_connection().await?;
    let mut client = pool.get().await?;
    let tx = client.transaction().await?;

    tx.execute(
        "INSERT INTO postal_codes (zip_code, prefecture_id, city_id, prefecture, city, town)
         VALUES ($1, $2, $3, $4, $5, $6)",
        &[
            &zip_code,
            &prefecture_id,
            &city_id,
            &prefecture,
            &city,
            &town,
        ],
    )
    .await?;

    let row = tx
        .query_one(
            "SELECT zip_code, prefecture_id, city_id, prefecture, city, town
             FROM postal_codes
             WHERE zip_code = $1 AND prefecture_id = $2 AND city = $3 AND town = $4",
            &[&zip_code, &prefecture_id, &city, &town],
        )
        .await?;

    assert_eq!(row.get::<_, String>(0), zip_code);
    assert_eq!(row.get::<_, i16>(1), prefecture_id);
    assert_eq!(row.get::<_, String>(2), city_id);
    assert_eq!(row.get::<_, String>(3), prefecture);
    assert_eq!(row.get::<_, String>(4), city);
    assert_eq!(row.get::<_, String>(5), town);

    tx.rollback().await?;
    Ok(())
}

#[test]
fn mysql_roundtrip_insert_and_query() -> Result<(), Box<dyn std::error::Error>> {
    let mysql_url = match std::env::var("MYSQL_DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("skip mysql test: MYSQL_DATABASE_URL is not set");
            return Ok(());
        }
    };

    let opts = mysql::Opts::from_url(&mysql_url)?;
    let pool = MySqlPool::new(opts)?;
    let mut conn = pool.get_conn()?;

    let seed = unique_seed();
    let zip_code = build_test_zip(seed);
    let prefecture_id: i16 = 27;
    let city_id = build_city_id('M', seed);
    let city = format!("integration-city-{seed}");
    let town = format!("integration-town-{seed}");
    let prefecture = "大阪府".to_string();

    conn.exec_drop(
        "INSERT INTO postal_codes (zip_code, prefecture_id, city_id, prefecture, city, town)
         VALUES (:zip_code, :prefecture_id, :city_id, :prefecture, :city, :town)",
        params! {
            "zip_code" => &zip_code,
            "prefecture_id" => prefecture_id,
            "city_id" => &city_id,
            "prefecture" => &prefecture,
            "city" => &city,
            "town" => &town,
        },
    )?;

    let row: Option<(String, i16, String, String, String, String)> = conn.exec_first(
        "SELECT zip_code, prefecture_id, city_id, prefecture, city, town
         FROM postal_codes
         WHERE zip_code = :zip_code AND prefecture_id = :prefecture_id AND city = :city AND town = :town",
        params! {
            "zip_code" => &zip_code,
            "prefecture_id" => prefecture_id,
            "city" => &city,
            "town" => &town,
        },
    )?;

    let row = row.expect("inserted row was not found in mysql");
    assert_eq!(row.0, zip_code);
    assert_eq!(row.1, prefecture_id);
    assert_eq!(row.2, city_id);
    assert_eq!(row.3, prefecture);
    assert_eq!(row.4, city);
    assert_eq!(row.5, town);

    conn.exec_drop(
        "DELETE FROM postal_codes
         WHERE zip_code = :zip_code AND prefecture_id = :prefecture_id AND city = :city AND town = :town",
        params! {
            "zip_code" => &zip_code,
            "prefecture_id" => prefecture_id,
            "city" => &city,
            "town" => &town,
        },
    )?;

    Ok(())
}
