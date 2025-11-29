use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use common::{db, models::PostalCode};
use deadpool_postgres::Pool as PgPool;
use mysql_async::Pool as MySqlPool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

enum DbPool {
    Postgres(PgPool),
    MySql(MySqlPool),
}

struct AppState {
    pool: DbPool,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Read DATABASE_TYPE from environment (default: postgres)
    let database_type = std::env::var("DATABASE_TYPE").unwrap_or_else(|_| "postgres".to_string());
    println!("Using database type: {}", database_type);

    let pool = match database_type.as_str() {
        "mysql" => {
            let mysql_pool = match db::mysql_connection().await {
                Ok(pool) => pool,
                Err(e) => {
                    eprintln!("Failed to connect to MySQL: {:?}", e);
                    return;
                }
            };
            DbPool::MySql(mysql_pool)
        }
        "postgres" | _ => {
            let pg_pool = match db::postgres_connection().await {
                Ok(pool) => pool,
                Err(e) => {
                    eprintln!("Failed to connect to PostgreSQL: {:?}", e);
                    return;
                }
            };
            DbPool::Postgres(pg_pool)
        }
    };

    let shared_state = Arc::new(AppState { pool });

    // Build our application with a route
    let app = Router::new()
        .route("/postal_codes/:zip_code", get(get_postal_code))
        .route("/postal_codes/search", get(search_postal_code))
        .route("/postal_codes/prefectures", get(get_prefectures))
        .route("/postal_codes/cities", get(get_cities))
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

    // Run it
    let listener = TcpListener::bind("0.0.0.0:3202").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn get_postal_code(
    State(state): State<Arc<AppState>>,
    Path(zip_code): Path<String>,
) -> Result<Json<Vec<PostalCode>>, StatusCode> {
    match &state.pool {
        DbPool::Postgres(pool) => {
            let client = pool
                .get()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let rows = client
                .query(
                    "SELECT zip_code, prefecture_id, city_id, prefecture, city, town FROM postal_codes WHERE zip_code = $1",
                    &[&zip_code],
                )
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if rows.is_empty() {
                return Err(StatusCode::NOT_FOUND);
            }

            let result: Vec<PostalCode> = rows
                .iter()
                .map(|row| PostalCode {
                    zip_code: row.get(0),
                    prefecture_id: row.get(1),
                    city_id: row.get(2),
                    prefecture: row.get(3),
                    city: row.get(4),
                    town: row.get(5),
                })
                .collect();
            Ok(Json(result))
        }
        DbPool::MySql(pool) => {
            use mysql_async::prelude::*;
            let mut conn = pool
                .get_conn()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let result: Vec<PostalCode> = conn
                .exec_map(
                    "SELECT zip_code, prefecture_id, city_id, prefecture, city, town FROM postal_codes WHERE zip_code = :zip_code",
                    mysql_async::params! {
                        "zip_code" => zip_code,
                    },
                    |(zip_code, prefecture_id, city_id, prefecture, city, town)| PostalCode {
                        zip_code,
                        prefecture_id,
                        city_id,
                        prefecture,
                        city,
                        town,
                    },
                )
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if result.is_empty() {
                return Err(StatusCode::NOT_FOUND);
            }
            Ok(Json(result))
        }
    }
}

#[derive(Deserialize)]
struct SearchParams {
    address: String,
}

async fn search_postal_code(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<PostalCode>>, StatusCode> {
    let search_term = format!("%{}%", params.address);

    match &state.pool {
        DbPool::Postgres(pool) => {
            let client = pool
                .get()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let rows = client
                .query(
                    "SELECT zip_code, prefecture_id, city_id, prefecture, city, town FROM postal_codes WHERE 
                    prefecture LIKE $1 OR 
                    city LIKE $1 OR 
                    town LIKE $1",
                    &[&search_term],
                )
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let result: Vec<PostalCode> = rows
                .iter()
                .map(|row| PostalCode {
                    zip_code: row.get(0),
                    prefecture_id: row.get(1),
                    city_id: row.get(2),
                    prefecture: row.get(3),
                    city: row.get(4),
                    town: row.get(5),
                })
                .collect();
            Ok(Json(result))
        }
        DbPool::MySql(pool) => {
            use mysql_async::prelude::*;
            let mut conn = pool
                .get_conn()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let result: Vec<PostalCode> = conn
                .exec_map(
                    "SELECT zip_code, prefecture_id, city_id, prefecture, city, town FROM postal_codes WHERE 
                    prefecture LIKE :search OR 
                    city LIKE :search OR 
                    town LIKE :search",
                    mysql_async::params! {
                        "search" => search_term,
                    },
                    |(zip_code, prefecture_id, city_id, prefecture, city, town)| PostalCode {
                        zip_code,
                        prefecture_id,
                        city_id,
                        prefecture,
                        city,
                        town,
                    },
                )
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(result))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
struct PrefectureResponse {
    prefecture_id: i32,
    prefecture: String,
}

async fn get_prefectures(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PrefectureResponse>>, StatusCode> {
    match &state.pool {
        DbPool::Postgres(pool) => {
            let client = pool
                .get()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let rows = client
                .query(
                    "SELECT DISTINCT prefecture_id, prefecture FROM postal_codes ORDER BY prefecture_id",
                    &[],
                )
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let result: Vec<PrefectureResponse> = rows
                .iter()
                .map(|row| PrefectureResponse {
                    prefecture_id: row.get(0),
                    prefecture: row.get(1),
                })
                .collect();
            Ok(Json(result))
        }
        DbPool::MySql(pool) => {
            use mysql_async::prelude::*;
            let mut conn = pool
                .get_conn()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let result: Vec<PrefectureResponse> = conn
                .exec_map(
                    "SELECT DISTINCT prefecture_id, prefecture FROM postal_codes ORDER BY prefecture_id",
                    (),
                    |(prefecture_id, prefecture)| PrefectureResponse {
                        prefecture_id,
                        prefecture,
                    },
                )
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(result))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
struct CityResponse {
    city_id: String,
    city: String,
}

#[derive(Deserialize)]
struct CityParams {
    prefecture_id: i32,
}

async fn get_cities(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CityParams>,
) -> Result<Json<Vec<CityResponse>>, StatusCode> {
    match &state.pool {
        DbPool::Postgres(pool) => {
            let client = pool
                .get()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let rows = client
                .query(
                    "SELECT DISTINCT city_id, city FROM postal_codes WHERE prefecture_id = $1 ORDER BY city_id",
                    &[&params.prefecture_id],
                )
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let result: Vec<CityResponse> = rows
                .iter()
                .map(|row| CityResponse {
                    city_id: row.get(0),
                    city: row.get(1),
                })
                .collect();
            Ok(Json(result))
        }
        DbPool::MySql(pool) => {
            use mysql_async::prelude::*;
            let mut conn = pool
                .get_conn()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let result: Vec<CityResponse> = conn
                .exec_map(
                    "SELECT DISTINCT city_id, city FROM postal_codes WHERE prefecture_id = :prefecture_id ORDER BY city_id",
                    mysql_async::params! {
                        "prefecture_id" => params.prefecture_id,
                    },
                    |(city_id, city)| CityResponse {
                        city_id,
                        city,
                    },
                )
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(result))
        }
    }
}
