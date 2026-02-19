//! Storage layer for Iqrah backend.

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub mod error;
pub mod pack_repository;
pub mod sync_repository;
pub mod user_repository;

pub use error::StorageError;
pub use pack_repository::{PackInfo, PackRepository};
pub use sync_repository::{ConflictLogEntry, SyncRepository};
pub use user_repository::{UserRepository, UserRow};

/// Create a PostgreSQL connection pool.
pub async fn create_pool(database_url: &str) -> Result<PgPool, StorageError> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(StorageError::Connection)
}

/// Run database migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<(), StorageError> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(StorageError::Migration)
}

/// Check database connectivity.
pub async fn check_connection(pool: &PgPool) -> Result<(), StorageError> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(StorageError::Query)?;
    Ok(())
}
