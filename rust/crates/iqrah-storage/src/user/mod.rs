pub mod repository;
mod models;

pub use repository::SqliteUserRepository;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use std::str::FromStr;

/// Initialize user database with migrations
pub async fn init_user_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations (from migrations/ directory at crate root)
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}

/// Get current schema version
pub async fn get_schema_version(pool: &SqlitePool) -> Result<i32, sqlx::Error> {
    let row = sqlx::query(
        "SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.get::<i64, _>("version") as i32).unwrap_or(0))
}

/// Check if a specific table exists
pub async fn table_exists(pool: &SqlitePool, table_name: &str) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?"
    )
    .bind(table_name)
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}
