mod models;
pub mod repository;

#[cfg(test)]
mod scheduler_tests;

pub use repository::SqliteUserRepository;

use crate::error::{Result, StorageError};
use crate::version::{get_schema_version, is_compatible};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

const EXPECTED_USER_VERSION: &str = "1.0.0";

/// Initialize user database with migrations
/// Initialize user database with migrations
pub async fn init_user_db(db_path: &str) -> Result<SqlitePool> {
    // FK constraints enabled - all migrations properly handle foreign key references
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations (from migrations_user/ directory at crate root)
    sqlx::migrate!("./migrations_user").run(&pool).await?;

    // Verify schema version compatibility
    let db_version = get_schema_version(&pool).await?;

    if !is_compatible(&db_version, EXPECTED_USER_VERSION) {
        return Err(StorageError::IncompatibleSchema {
            db_version,
            app_version: EXPECTED_USER_VERSION.to_string(),
            message: "User database schema is incompatible with this app version".to_string(),
        });
    }

    tracing::info!(
        "User DB initialized: schema v{}, expected v{}",
        db_version,
        EXPECTED_USER_VERSION
    );

    Ok(pool)
}

/// Check if a specific table exists
pub async fn table_exists(pool: &SqlitePool, table_name: &str) -> Result<bool> {
    let row = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name=?")
        .bind(table_name)
        .fetch_optional(pool)
        .await?;

    Ok(row.is_some())
}
