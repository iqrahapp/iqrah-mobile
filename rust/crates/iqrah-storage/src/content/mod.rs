mod models;
pub mod repository;

pub use repository::SqliteContentRepository;

use sqlx::{query_scalar, sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

const EXPECTED_SCHEMA_VERSION: i32 = 2;

/// Get the content database schema version
pub async fn get_content_schema_version(pool: &SqlitePool) -> Result<i32, sqlx::Error> {
    query_scalar("SELECT version FROM schema_version")
        .fetch_one(pool)
        .await
}

/// Initialize content database
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_path)?.create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations for content database
    sqlx::migrate!("./migrations_content").run(&pool).await?;

    // Validate schema version
    let version = get_content_schema_version(&pool).await?;
    if version != EXPECTED_SCHEMA_VERSION {
        return Err(sqlx::Error::Configuration(
            format!(
                "Schema version mismatch: expected {}, found {}",
                EXPECTED_SCHEMA_VERSION, version
            )
            .into(),
        ));
    }

    Ok(pool)
}
