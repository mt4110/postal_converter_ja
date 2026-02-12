use axum::{
    extract::{Path, Query, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{Html, Response},
    routing::get,
    Json, Router,
};
use common::{db, models::PostalCode};
use deadpool_postgres::Pool as PgPool;
use mysql_async::Pool as MySqlPool;
use redis::{aio::ConnectionManager as RedisConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use unicode_normalization::UnicodeNormalization;
use utoipa::{OpenApi, ToSchema};

enum DbPool {
    Postgres(PgPool),
    MySql(MySqlPool),
    Sqlite(String),
}

struct AppState {
    pool: DbPool,
    cache: Option<RedisConnectionManager>,
    cache_ttl_seconds: u64,
    ready_require_cache: bool,
    metrics: ApiMetrics,
}

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize, ToSchema)]
struct ReadyResponse {
    status: String,
    database: String,
    cache: String,
}

#[derive(Serialize, ToSchema)]
struct MetricsResponse {
    requests_total: u64,
    errors_total: u64,
    not_found_total: u64,
    error_rate: f64,
    average_latency_ms: f64,
}

#[derive(Default)]
struct ApiMetrics {
    requests_total: AtomicU64,
    errors_total: AtomicU64,
    not_found_total: AtomicU64,
    latency_total_micros: AtomicU64,
}

impl ApiMetrics {
    fn record(&self, status: StatusCode, latency: std::time::Duration) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        if status == StatusCode::NOT_FOUND {
            self.not_found_total.fetch_add(1, Ordering::Relaxed);
        }
        if status.is_server_error() {
            self.errors_total.fetch_add(1, Ordering::Relaxed);
        }
        let latency_micros = latency.as_micros().min(u128::from(u64::MAX)) as u64;
        self.latency_total_micros
            .fetch_add(latency_micros, Ordering::Relaxed);
    }

    fn snapshot(&self) -> MetricsResponse {
        let requests_total = self.requests_total.load(Ordering::Relaxed);
        let errors_total = self.errors_total.load(Ordering::Relaxed);
        let not_found_total = self.not_found_total.load(Ordering::Relaxed);
        let latency_total_micros = self.latency_total_micros.load(Ordering::Relaxed);

        let average_latency_ms = if requests_total == 0 {
            0.0
        } else {
            latency_total_micros as f64 / requests_total as f64 / 1000.0
        };
        let error_rate = if requests_total == 0 {
            0.0
        } else {
            errors_total as f64 / requests_total as f64
        };

        MetricsResponse {
            requests_total,
            errors_total,
            not_found_total,
            error_rate,
            average_latency_ms,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, ToSchema)]
struct PrefectureResponse {
    prefecture_id: i16,
    prefecture: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, ToSchema)]
struct CityResponse {
    city_id: String,
    city: String,
}

#[derive(Deserialize)]
struct SearchParams {
    address: String,
    limit: Option<u32>,
    mode: Option<SearchMode>,
}

#[derive(Deserialize)]
struct CityParams {
    prefecture_id: i16,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
enum SearchMode {
    Exact,
    Prefix,
    Partial,
}

impl Default for SearchMode {
    fn default() -> Self {
        Self::Partial
    }
}

impl SearchMode {
    fn as_cache_key(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Prefix => "prefix",
            Self::Partial => "partial",
        }
    }

    fn needs_like(self) -> bool {
        !matches!(self, Self::Exact)
    }
}

fn build_search_term(mode: SearchMode, address: &str) -> String {
    match mode {
        SearchMode::Exact => address.to_string(),
        SearchMode::Prefix => format!("{address}%"),
        SearchMode::Partial => format!("%{address}%"),
    }
}

fn normalize_search_input(input: &str) -> String {
    input
        .nfkc()
        .collect::<String>()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
}

fn hiragana_to_katakana(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            let code = c as u32;
            if (0x3041..=0x3096).contains(&code) {
                char::from_u32(code + 0x60).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

fn katakana_to_hiragana(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            let code = c as u32;
            if (0x30A1..=0x30F6).contains(&code) {
                char::from_u32(code - 0x60).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

fn push_unique_candidate(candidates: &mut Vec<String>, candidate: String) {
    if !candidate.is_empty() && !candidates.contains(&candidate) {
        candidates.push(candidate);
    }
}

fn build_search_candidates(normalized_address: &str) -> Vec<String> {
    let mut candidates = Vec::with_capacity(3);
    push_unique_candidate(&mut candidates, normalized_address.to_string());
    push_unique_candidate(&mut candidates, hiragana_to_katakana(normalized_address));
    push_unique_candidate(&mut candidates, katakana_to_hiragana(normalized_address));
    candidates
}

fn append_unique_with_limit(
    acc: &mut Vec<PostalCode>,
    seen: &mut HashSet<PostalCode>,
    chunk: Vec<PostalCode>,
    limit: usize,
) {
    for item in chunk {
        if seen.insert(item.clone()) {
            acc.push(item);
            if acc.len() >= limit {
                break;
            }
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_postal_code,
        search_postal_code,
        get_prefectures,
        get_cities,
        health,
        ready,
        metrics
    ),
    components(schemas(
        PostalCode,
        PrefectureResponse,
        CityResponse,
        HealthResponse,
        ReadyResponse,
        MetricsResponse,
        ErrorResponse
    )),
    tags((name = "postal_codes", description = "Postal Converter JA API"))
)]
struct ApiDoc;

type ApiError = (StatusCode, Json<ErrorResponse>);

fn internal_error() -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "internal server error".to_string(),
        }),
    )
}

fn not_found_error() -> ApiError {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "not found".to_string(),
        }),
    )
}

fn not_ready_error(message: &str) -> ApiError {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn parse_bool_env(var_name: &str, default: bool) -> bool {
    std::env::var(var_name)
        .ok()
        .map(|v| is_truthy(&v))
        .unwrap_or(default)
}

fn resolve_cache_state(
    cache_enabled: bool,
    cache_ping_ok: bool,
    ready_require_cache: bool,
) -> Result<&'static str, &'static str> {
    if !cache_enabled {
        return Ok("disabled");
    }
    if cache_ping_ok {
        return Ok("ok");
    }
    if ready_require_cache {
        return Err("cache not ready");
    }
    Ok("error")
}

async fn cache_get<T: DeserializeOwned>(
    cache: &Option<RedisConnectionManager>,
    key: &str,
) -> Option<T> {
    let manager = cache.as_ref()?;
    let mut conn = manager.clone();
    let payload: Option<String> = conn.get(key).await.ok()?;
    let payload = payload?;
    serde_json::from_str(&payload).ok()
}

async fn cache_set<T: Serialize>(
    cache: &Option<RedisConnectionManager>,
    key: &str,
    value: &T,
    ttl_seconds: u64,
) {
    let Some(manager) = cache.as_ref() else {
        return;
    };
    let Ok(payload) = serde_json::to_string(value) else {
        return;
    };
    let mut conn = manager.clone();
    let _: Result<(), redis::RedisError> = conn.set_ex(key, payload, ttl_seconds).await;
}

async fn request_metrics_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let started = Instant::now();
    let response = next.run(request).await;
    let latency = started.elapsed();
    let status = response.status();

    state.metrics.record(status, latency);
    println!(
        r#"{{"event":"api_request","method":"{}","path":"{}","status":{},"latency_ms":{}}}"#,
        method,
        path,
        status.as_u16(),
        latency.as_millis()
    );

    response
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let database_type = std::env::var("DATABASE_TYPE").unwrap_or_else(|_| "postgres".to_string());
    println!("Using database type: {}", database_type);

    let pool = match database_type.as_str() {
        "sqlite" => {
            let sqlite_path = std::env::var("SQLITE_DATABASE_PATH")
                .unwrap_or_else(|_| "storage/sqlite/postal_codes.sqlite3".to_string());

            if let Err(e) = rusqlite::Connection::open(&sqlite_path) {
                eprintln!("Failed to open SQLite database at '{}': {e}", sqlite_path);
                return;
            }
            println!("Using SQLite database at: {}", sqlite_path);
            DbPool::Sqlite(sqlite_path)
        }
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
        _ => {
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

    let cache_ttl_seconds: u64 = std::env::var("REDIS_CACHE_TTL_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(300);

    let redis_cache = match std::env::var("REDIS_URL") {
        Ok(redis_url) => match redis::Client::open(redis_url) {
            Ok(client) => match RedisConnectionManager::new(client).await {
                Ok(manager) => {
                    println!("Redis cache enabled (ttl={}s).", cache_ttl_seconds);
                    Some(manager)
                }
                Err(e) => {
                    eprintln!("Failed to create Redis connection manager: {e}");
                    None
                }
            },
            Err(e) => {
                eprintln!("Invalid REDIS_URL: {e}");
                None
            }
        },
        Err(_) => None,
    };

    let ready_require_cache = parse_bool_env("READY_REQUIRE_CACHE", false);
    if ready_require_cache {
        println!("Readiness strict mode enabled: cache must be available when REDIS_URL is set.");
    }

    let shared_state = Arc::new(AppState {
        pool,
        cache: redis_cache,
        cache_ttl_seconds,
        ready_require_cache,
        metrics: ApiMetrics::default(),
    });

    let app = Router::new()
        .route("/postal_codes/{zip_code}", get(get_postal_code))
        .route("/postal_codes/search", get(search_postal_code))
        .route("/postal_codes/prefectures", get(get_prefectures))
        .route("/postal_codes/cities", get(get_cities))
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .route("/openapi.json", get(openapi_json))
        .route("/docs", get(swagger_ui))
        .route("/docs/", get(swagger_ui))
        .layer(axum::middleware::from_fn_with_state(
            shared_state.clone(),
            request_metrics_middleware,
        ))
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:3202").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[utoipa::path(
    get,
    path = "/postal_codes/{zip_code}",
    params(
        ("zip_code" = String, Path, description = "7-digit postal code")
    ),
    responses(
        (status = 200, description = "Postal code lookup result", body = [PostalCode]),
        (status = 404, description = "Postal code not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn get_postal_code(
    State(state): State<Arc<AppState>>,
    Path(zip_code): Path<String>,
) -> Result<Json<Vec<PostalCode>>, ApiError> {
    let cache_key = format!("postal:zip:{zip_code}");
    if let Some(cached) = cache_get::<Vec<PostalCode>>(&state.cache, &cache_key).await {
        return Ok(Json(cached));
    }

    match &state.pool {
        DbPool::Postgres(pool) => {
            let client = pool.get().await.map_err(|_| internal_error())?;
            let rows = client
                .query(
                    "SELECT zip_code, prefecture_id, city_id, prefecture, city, town FROM postal_codes WHERE zip_code = $1",
                    &[&zip_code],
                )
                .await
                .map_err(|_| internal_error())?;

            if rows.is_empty() {
                return Err(not_found_error());
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
            cache_set(&state.cache, &cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
        DbPool::MySql(pool) => {
            use mysql_async::prelude::*;
            let mut conn = pool.get_conn().await.map_err(|_| internal_error())?;
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
                .map_err(|_| internal_error())?;

            if result.is_empty() {
                return Err(not_found_error());
            }
            cache_set(&state.cache, &cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
        DbPool::Sqlite(path) => {
            let result: Vec<PostalCode> = {
                let conn = rusqlite::Connection::open(path).map_err(|_| internal_error())?;
                let mut stmt = conn
                    .prepare(
                        "SELECT zip_code, prefecture_id, city_id, prefecture, city, COALESCE(town, '')
                         FROM postal_codes WHERE zip_code = ?1",
                    )
                    .map_err(|_| internal_error())?;

                let rows = stmt
                    .query_map([zip_code], |row| {
                        Ok(PostalCode {
                            zip_code: row.get(0)?,
                            prefecture_id: row.get(1)?,
                            city_id: row.get(2)?,
                            prefecture: row.get(3)?,
                            city: row.get(4)?,
                            town: row.get(5)?,
                        })
                    })
                    .map_err(|_| internal_error())?;

                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|_| internal_error())?
            };

            if result.is_empty() {
                return Err(not_found_error());
            }
            cache_set(&state.cache, &cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
    }
}

#[utoipa::path(
    get,
    path = "/postal_codes/search",
    params(
        ("address" = String, Query, description = "Address keyword (kana normalization is applied)"),
        ("limit" = Option<u32>, Query, description = "Result size, default=50, max=200"),
        ("mode" = Option<String>, Query, description = "Search mode: exact | prefix | partial (default)")
    ),
    responses(
        (status = 200, description = "Address search result", body = [PostalCode]),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn search_postal_code(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<PostalCode>>, ApiError> {
    let normalized_address = normalize_search_input(&params.address);
    if normalized_address.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let candidates = build_search_candidates(&normalized_address);
    let mode = params.mode.unwrap_or_default();
    let limit_u32 = params.limit.unwrap_or(50).clamp(1, 200);
    let limit = i64::from(limit_u32);
    let limit_usize = limit_u32 as usize;
    let cache_key = format!(
        "postal:search:{}:{}:{limit}",
        mode.as_cache_key(),
        normalized_address
    );
    if let Some(cached) = cache_get::<Vec<PostalCode>>(&state.cache, &cache_key).await {
        return Ok(Json(cached));
    }

    let search_terms: Vec<String> = candidates
        .iter()
        .map(|candidate| build_search_term(mode, candidate))
        .collect();

    match &state.pool {
        DbPool::Postgres(pool) => {
            const QUERY_LIKE: &str =
                "SELECT zip_code, prefecture_id, city_id, prefecture, city, town
                FROM postal_codes WHERE
                prefecture LIKE $1 OR
                city LIKE $1 OR
                town LIKE $1
                LIMIT $2";
            const QUERY_EXACT: &str =
                "SELECT zip_code, prefecture_id, city_id, prefecture, city, town
                FROM postal_codes WHERE
                prefecture = $1 OR
                city = $1 OR
                town = $1
                LIMIT $2";

            let client = pool.get().await.map_err(|_| internal_error())?;
            let mut result: Vec<PostalCode> = Vec::new();
            let mut seen: HashSet<PostalCode> = HashSet::new();

            for search_term in &search_terms {
                if result.len() >= limit_usize {
                    break;
                }
                let remaining = (limit_usize - result.len()) as i64;
                let rows = client
                    .query(
                        if mode.needs_like() {
                            QUERY_LIKE
                        } else {
                            QUERY_EXACT
                        },
                        &[search_term, &remaining],
                    )
                    .await
                    .map_err(|_| internal_error())?;

                let chunk: Vec<PostalCode> = rows
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
                append_unique_with_limit(&mut result, &mut seen, chunk, limit_usize);
            }

            cache_set(&state.cache, &cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
        DbPool::MySql(pool) => {
            const QUERY_LIKE: &str =
                "SELECT zip_code, prefecture_id, city_id, prefecture, city, town
                FROM postal_codes WHERE
                prefecture LIKE :search OR
                city LIKE :search OR
                town LIKE :search
                LIMIT :limit";
            const QUERY_EXACT: &str =
                "SELECT zip_code, prefecture_id, city_id, prefecture, city, town
                FROM postal_codes WHERE
                prefecture = :search OR
                city = :search OR
                town = :search
                LIMIT :limit";

            use mysql_async::prelude::*;
            let mut conn = pool.get_conn().await.map_err(|_| internal_error())?;
            let mut result: Vec<PostalCode> = Vec::new();
            let mut seen: HashSet<PostalCode> = HashSet::new();

            for search_term in &search_terms {
                if result.len() >= limit_usize {
                    break;
                }
                let remaining = (limit_usize - result.len()) as i64;
                let chunk: Vec<PostalCode> = conn
                    .exec_map(
                        if mode.needs_like() {
                            QUERY_LIKE
                        } else {
                            QUERY_EXACT
                        },
                        mysql_async::params! {
                            "search" => search_term,
                            "limit" => remaining,
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
                    .map_err(|_| internal_error())?;
                append_unique_with_limit(&mut result, &mut seen, chunk, limit_usize);
            }

            cache_set(&state.cache, &cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
        DbPool::Sqlite(path) => {
            const QUERY_LIKE: &str =
                "SELECT zip_code, prefecture_id, city_id, prefecture, city, COALESCE(town, '')
                FROM postal_codes
                WHERE prefecture LIKE ?1 OR city LIKE ?1 OR town LIKE ?1
                LIMIT ?2";
            const QUERY_EXACT: &str =
                "SELECT zip_code, prefecture_id, city_id, prefecture, city, COALESCE(town, '')
                FROM postal_codes
                WHERE prefecture = ?1 OR city = ?1 OR town = ?1
                LIMIT ?2";

            let conn = rusqlite::Connection::open(path).map_err(|_| internal_error())?;
            let mut result: Vec<PostalCode> = Vec::new();
            let mut seen: HashSet<PostalCode> = HashSet::new();

            for search_term in &search_terms {
                if result.len() >= limit_usize {
                    break;
                }
                let remaining = (limit_usize - result.len()) as i64;
                let mut stmt = conn
                    .prepare(if mode.needs_like() {
                        QUERY_LIKE
                    } else {
                        QUERY_EXACT
                    })
                    .map_err(|_| internal_error())?;

                let rows = stmt
                    .query_map(rusqlite::params![search_term, remaining], |row| {
                        Ok(PostalCode {
                            zip_code: row.get(0)?,
                            prefecture_id: row.get(1)?,
                            city_id: row.get(2)?,
                            prefecture: row.get(3)?,
                            city: row.get(4)?,
                            town: row.get(5)?,
                        })
                    })
                    .map_err(|_| internal_error())?;

                let chunk = rows
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| internal_error())?;
                append_unique_with_limit(&mut result, &mut seen, chunk, limit_usize);
            }

            cache_set(&state.cache, &cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
    }
}

#[utoipa::path(
    get,
    path = "/postal_codes/prefectures",
    responses(
        (status = 200, description = "Prefecture list", body = [PrefectureResponse]),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn get_prefectures(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PrefectureResponse>>, ApiError> {
    let cache_key = "postal:prefectures";
    if let Some(cached) = cache_get::<Vec<PrefectureResponse>>(&state.cache, cache_key).await {
        return Ok(Json(cached));
    }

    match &state.pool {
        DbPool::Postgres(pool) => {
            let client = pool.get().await.map_err(|_| internal_error())?;
            let rows = client
                .query(
                    "SELECT DISTINCT prefecture_id, prefecture FROM postal_codes ORDER BY prefecture_id",
                    &[],
                )
                .await
                .map_err(|_| internal_error())?;

            let result: Vec<PrefectureResponse> = rows
                .iter()
                .map(|row| PrefectureResponse {
                    prefecture_id: row.get(0),
                    prefecture: row.get(1),
                })
                .collect();
            cache_set(&state.cache, cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
        DbPool::MySql(pool) => {
            use mysql_async::prelude::*;
            let mut conn = pool.get_conn().await.map_err(|_| internal_error())?;
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
                .map_err(|_| internal_error())?;
            cache_set(&state.cache, cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
        DbPool::Sqlite(path) => {
            let result: Vec<PrefectureResponse> = {
                let conn = rusqlite::Connection::open(path).map_err(|_| internal_error())?;
                let mut stmt = conn
                    .prepare(
                        "SELECT DISTINCT prefecture_id, prefecture
                         FROM postal_codes
                         ORDER BY prefecture_id",
                    )
                    .map_err(|_| internal_error())?;

                let rows = stmt
                    .query_map([], |row| {
                        Ok(PrefectureResponse {
                            prefecture_id: row.get(0)?,
                            prefecture: row.get(1)?,
                        })
                    })
                    .map_err(|_| internal_error())?;

                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|_| internal_error())?
            };

            cache_set(&state.cache, cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
    }
}

#[utoipa::path(
    get,
    path = "/postal_codes/cities",
    params(
        ("prefecture_id" = i16, Query, description = "Prefecture id")
    ),
    responses(
        (status = 200, description = "City list", body = [CityResponse]),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn get_cities(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CityParams>,
) -> Result<Json<Vec<CityResponse>>, ApiError> {
    let cache_key = format!("postal:cities:{}", params.prefecture_id);
    if let Some(cached) = cache_get::<Vec<CityResponse>>(&state.cache, &cache_key).await {
        return Ok(Json(cached));
    }

    match &state.pool {
        DbPool::Postgres(pool) => {
            let client = pool.get().await.map_err(|_| internal_error())?;
            let rows = client
                .query(
                    "SELECT DISTINCT city_id, city FROM postal_codes WHERE prefecture_id = $1 ORDER BY city_id",
                    &[&params.prefecture_id],
                )
                .await
                .map_err(|_| internal_error())?;

            let result: Vec<CityResponse> = rows
                .iter()
                .map(|row| CityResponse {
                    city_id: row.get(0),
                    city: row.get(1),
                })
                .collect();
            cache_set(&state.cache, &cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
        DbPool::MySql(pool) => {
            use mysql_async::prelude::*;
            let mut conn = pool.get_conn().await.map_err(|_| internal_error())?;
            let result: Vec<CityResponse> = conn
                .exec_map(
                    "SELECT DISTINCT city_id, city FROM postal_codes WHERE prefecture_id = :prefecture_id ORDER BY city_id",
                    mysql_async::params! {
                        "prefecture_id" => params.prefecture_id,
                    },
                    |(city_id, city)| CityResponse { city_id, city },
                )
                .await
                .map_err(|_| internal_error())?;
            cache_set(&state.cache, &cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
        DbPool::Sqlite(path) => {
            let result: Vec<CityResponse> = {
                let conn = rusqlite::Connection::open(path).map_err(|_| internal_error())?;
                let mut stmt = conn
                    .prepare(
                        "SELECT DISTINCT city_id, city
                         FROM postal_codes
                         WHERE prefecture_id = ?1
                         ORDER BY city_id",
                    )
                    .map_err(|_| internal_error())?;

                let rows = stmt
                    .query_map([params.prefecture_id], |row| {
                        Ok(CityResponse {
                            city_id: row.get(0)?,
                            city: row.get(1)?,
                        })
                    })
                    .map_err(|_| internal_error())?;

                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|_| internal_error())?
            };

            cache_set(&state.cache, &cache_key, &result, state.cache_ttl_seconds).await;
            Ok(Json(result))
        }
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check", body = HealthResponse)
    )
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/ready",
    responses(
        (status = 200, description = "Readiness check", body = ReadyResponse),
        (status = 503, description = "Service not ready", body = ErrorResponse)
    )
)]
async fn ready(State(state): State<Arc<AppState>>) -> Result<Json<ReadyResponse>, ApiError> {
    let database = match &state.pool {
        DbPool::Postgres(pool) => {
            let client = pool
                .get()
                .await
                .map_err(|_| not_ready_error("database not ready"))?;
            client
                .query_one("SELECT 1", &[])
                .await
                .map_err(|_| not_ready_error("database not ready"))?;
            "postgres"
        }
        DbPool::MySql(pool) => {
            use mysql_async::prelude::Queryable;
            let mut conn = pool
                .get_conn()
                .await
                .map_err(|_| not_ready_error("database not ready"))?;
            conn.query_first::<i8, _>("SELECT 1")
                .await
                .map_err(|_| not_ready_error("database not ready"))?;
            "mysql"
        }
        DbPool::Sqlite(path) => {
            let conn = rusqlite::Connection::open(path)
                .map_err(|_| not_ready_error("database not ready"))?;
            let _: i64 = conn
                .query_row("SELECT 1", [], |row| row.get(0))
                .map_err(|_| not_ready_error("database not ready"))?;
            "sqlite"
        }
    };

    let (cache_enabled, cache_ping_ok) = if let Some(cache) = &state.cache {
        let mut conn = cache.clone();
        let ping: Result<String, redis::RedisError> =
            redis::cmd("PING").query_async(&mut conn).await;
        (true, ping.is_ok())
    } else {
        (false, false)
    };

    let cache_state = resolve_cache_state(cache_enabled, cache_ping_ok, state.ready_require_cache)
        .map_err(not_ready_error)?;

    Ok(Json(ReadyResponse {
        status: "ready".to_string(),
        database: database.to_string(),
        cache: cache_state.to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Application metrics", body = MetricsResponse)
    )
)]
async fn metrics(State(state): State<Arc<AppState>>) -> Json<MetricsResponse> {
    Json(state.metrics.snapshot())
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

async fn swagger_ui() -> Html<&'static str> {
    Html(
        r##"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>Postal Converter JA API Docs</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
      window.onload = () => {
        window.ui = SwaggerUIBundle({
          url: "/openapi.json",
          dom_id: "#swagger-ui",
          deepLinking: true,
        });
      };
    </script>
  </body>
</html>"##,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_search_candidates, build_search_term, hiragana_to_katakana, is_truthy,
        katakana_to_hiragana, normalize_search_input, resolve_cache_state, ApiMetrics, SearchMode,
    };
    use axum::http::StatusCode;
    use std::time::Duration;

    #[test]
    fn build_search_term_exact() {
        assert_eq!(build_search_term(SearchMode::Exact, "新宿"), "新宿");
    }

    #[test]
    fn build_search_term_prefix() {
        assert_eq!(build_search_term(SearchMode::Prefix, "新宿"), "新宿%");
    }

    #[test]
    fn build_search_term_partial() {
        assert_eq!(build_search_term(SearchMode::Partial, "新宿"), "%新宿%");
    }

    #[test]
    fn normalize_search_input_nfkc_and_trim_spaces() {
        assert_eq!(normalize_search_input("  ｼﾝ ｼﾞｭｸ  "), "シンジュク");
    }

    #[test]
    fn kana_conversion_hiragana_to_katakana() {
        assert_eq!(hiragana_to_katakana("しんじゅく"), "シンジュク");
    }

    #[test]
    fn kana_conversion_hiragana_to_katakana_voiced_vu() {
        assert_eq!(hiragana_to_katakana("ゔ"), "ヴ");
    }

    #[test]
    fn kana_conversion_katakana_to_hiragana() {
        assert_eq!(katakana_to_hiragana("シンジュク"), "しんじゅく");
    }

    #[test]
    fn kana_conversion_katakana_to_hiragana_voiced_vu() {
        assert_eq!(katakana_to_hiragana("ヴ"), "ゔ");
    }

    #[test]
    fn build_search_candidates_keeps_unique_variants() {
        let c = build_search_candidates("しんじゅく");
        assert_eq!(c, vec!["しんじゅく", "シンジュク"]);
    }

    #[test]
    fn metrics_snapshot_aggregates_values() {
        let metrics = ApiMetrics::default();
        metrics.record(StatusCode::OK, Duration::from_millis(10));
        metrics.record(StatusCode::NOT_FOUND, Duration::from_millis(30));
        metrics.record(StatusCode::INTERNAL_SERVER_ERROR, Duration::from_millis(20));

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.requests_total, 3);
        assert_eq!(snapshot.not_found_total, 1);
        assert_eq!(snapshot.errors_total, 1);
        assert!((snapshot.error_rate - (1.0 / 3.0)).abs() < f64::EPSILON);
        assert!((snapshot.average_latency_ms - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn truthy_parser_accepts_common_true_values() {
        for value in ["1", "true", "TRUE", " yes ", "On"] {
            assert!(is_truthy(value), "expected `{value}` to be treated as true");
        }
    }

    #[test]
    fn truthy_parser_rejects_non_true_values() {
        for value in ["0", "false", "", "disabled", "no"] {
            assert!(
                !is_truthy(value),
                "expected `{value}` to be treated as false"
            );
        }
    }

    #[test]
    fn cache_state_disabled_when_cache_not_enabled() {
        assert_eq!(resolve_cache_state(false, false, true), Ok("disabled"));
    }

    #[test]
    fn cache_state_ready_when_cache_ping_succeeds() {
        assert_eq!(resolve_cache_state(true, true, true), Ok("ok"));
    }

    #[test]
    fn cache_state_error_when_strict_mode_disabled() {
        assert_eq!(resolve_cache_state(true, false, false), Ok("error"));
    }

    #[test]
    fn cache_state_fails_when_strict_mode_enabled() {
        assert_eq!(
            resolve_cache_state(true, false, true),
            Err("cache not ready")
        );
    }
}
