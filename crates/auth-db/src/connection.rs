//! Database connection management

use anyhow::Result;
use auth_config::DatabaseConfig;
use secrecy::ExposeSecret;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use sqlx::{MySql, MySqlPool, Pool, Sqlite, SqlitePool};
use std::time::Duration;

pub enum DatabasePool {
    MySql(MySqlPool),
    Sqlite(SqlitePool),
}

pub async fn create_mysql_pool(config: &DatabaseConfig) -> Result<Pool<MySql>> {
    let options = config
        .mysql_url
        .expose_secret()
        .parse::<MySqlConnectOptions>()?;

    let pool = MySqlPoolOptions::new()
        .max_connections(200) // Increased from 50 to support 1000 req/sec
        .min_connections(20) // Keep 10% warm
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Some(Duration::from_secs(300))) // Close idle connections after 5 min
        .max_lifetime(Some(Duration::from_secs(1800))) // Recycle connections after 30 min
        .test_before_acquire(true) // Verify connection health before use
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn create_sqlite_pool(database_url: &str) -> Result<Pool<Sqlite>> {
    let pool = SqlitePool::connect(database_url).await?;
    Ok(pool)
}
