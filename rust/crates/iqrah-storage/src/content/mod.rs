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

    // Run schema creation
    let schema = include_str!("../content_schema.sql");
    sqlx::raw_sql(schema).execute(&pool).await?;

    Ok(pool)
}
