//! Storage errors.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database connection error: {0}")]
    Connection(#[source] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[source] sqlx::migrate::MigrateError),

    #[error("Query error: {0}")]
    Query(#[source] sqlx::Error),
}
