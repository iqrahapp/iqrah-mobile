pub mod repository;
mod models;

pub use repository::SqliteContentRepository;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

/// Initialize content database
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations for content database
    sqlx::migrate!("./migrations_content")
        .run(&pool)
        .await?;

    Ok(pool)
}
