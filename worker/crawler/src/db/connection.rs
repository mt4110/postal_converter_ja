use deadpool_postgres::{Config, ManagerConfig, PoolConfig};
use dotenv::dotenv;
use mysql_async::{Opts, OptsBuilder};
use std::env;
use tokio_postgres::{Error as PgError, NoTls};

pub async fn mysql_connection() -> Result<mysql_async::Pool, mysql_async::Error> {
    dotenv().ok();
    let database_url = env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL not set");
    let tcp_keepalive_seconds = Some(10_u32);
    let connect_timeout = Some(30);

    let opts: mysql_async::Opts = OptsBuilder::from_opts(Opts::from_url(&database_url)?)
        .tcp_keepalive(tcp_keepalive_seconds)
        .wait_timeout(connect_timeout)
        .into();
    Ok(mysql_async::Pool::new(opts))
}

pub async fn postgres_connection() -> Result<deadpool_postgres::Pool, PgError> {
    dotenv().ok();
    let conn_str = env::var("POSTGRES_DATABASE_URL").expect("POSTGRES_DATABASE_URL not set");
    // construct managed pool
    let pool_config: PoolConfig = PoolConfig::new(5); // max pool size
    let mng_config: ManagerConfig = ManagerConfig::default();
    let config: Config = Config {
        url: Some(conn_str),
        manager: Some(mng_config),
        pool: Some(pool_config),
        ..Default::default()
    };
    let pool = config.builder(NoTls).unwrap().build().unwrap();
    Ok(pool)
}
