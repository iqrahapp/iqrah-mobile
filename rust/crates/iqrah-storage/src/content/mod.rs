mod models;
pub mod repository;

#[cfg(test)]
mod scheduler_tests;

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
    // Note: FK constraints are disabled during migrations to avoid AUTOINCREMENT ID issues.
    // The base schema migration uses AUTOINCREMENT for translators and words, then
    // references those IDs in verse_translations and word_translations.
    // We've improved this with subqueries (e.g., SELECT translator_id WHERE slug='...'),
    // but migrations still fail due to sqlx running each migration in separate transactions.
    // FK constraints are properly enforced after migrations complete.
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true)
        .foreign_keys(false); // Disabled during migrations only

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
