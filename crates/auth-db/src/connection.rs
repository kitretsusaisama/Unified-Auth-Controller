//! Database connection management

use auth_config::DatabaseConfig;
use sqlx::{MySqlPool, SqlitePool, Pool, MySql, Sqlite};
use anyhow::Result;
use secrecy::ExposeSecret;

pub enum DatabasePool {
    MySql(MySqlPool),
    Sqlite(SqlitePool),
}

pub async fn create_mysql_pool(config: &DatabaseConfig) -> Result<Pool<MySql>> {
    let pool = MySqlPool::connect(config.mysql_url.expose_secret()).await?;
    Ok(pool)
}

pub async fn create_sqlite_pool(database_url: &str) -> Result<Pool<Sqlite>> {
    let pool = SqlitePool::connect(database_url).await?;
    Ok(pool)
}