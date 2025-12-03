mod models;
mod node_registry;
pub mod repository;

#[cfg(test)]
mod scheduler_tests;

pub use node_registry::NodeRegistry;
pub use repository::SqliteContentRepository;

use crate::error::{Result, StorageError};
use crate::version::{get_schema_version, is_compatible};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

const EXPECTED_CONTENT_VERSION: &str = "2.0.0";

/// Initialize content database
/// Initialize content database
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool> {
    // FK constraints enabled - all migrations properly handle foreign key references
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations for content database
    sqlx::migrate!("./migrations_content").run(&pool).await?;

    // Verify schema version compatibility
    let db_version = get_schema_version(&pool).await?;

    if !is_compatible(&db_version, EXPECTED_CONTENT_VERSION) {
        return Err(StorageError::IncompatibleSchema {
            db_version,
            app_version: EXPECTED_CONTENT_VERSION.to_string(),
            message: "Content database schema is incompatible with this app version".to_string(),
        });
    }

    tracing::info!(
        "Content DB initialized: schema v{}, expected v{}",
        db_version,
        EXPECTED_CONTENT_VERSION
    );

    Ok(pool)
}

/// Create a content repository with NodeRegistry
/// This is a convenience function that creates the necessary Arc-wrapped dependencies
pub fn create_content_repository(pool: SqlitePool) -> SqliteContentRepository {
    let registry = std::sync::Arc::new(NodeRegistry::new(pool.clone()));
    SqliteContentRepository::new(pool, registry)
}
