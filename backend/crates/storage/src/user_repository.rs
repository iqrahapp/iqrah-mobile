//! User repository.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::StorageError;

/// User record from database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub oauth_sub: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// User repository.
#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find user by OAuth sub, or create if not exists.
    ///
    /// Uses a single atomic upsert to avoid TOCTOU race conditions on concurrent
    /// first-time logins from the same Google account.
    pub async fn find_or_create(&self, oauth_sub: &str) -> Result<UserRow, StorageError> {
        let user = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (oauth_sub, last_seen_at)
            VALUES ($1, now())
            ON CONFLICT (oauth_sub) DO UPDATE SET last_seen_at = now()
            RETURNING id, oauth_sub, created_at, last_seen_at
            "#,
        )
        .bind(oauth_sub)
        .fetch_one(&self.pool)
        .await
        .map_err(StorageError::Query)?;

        Ok(user)
    }

    /// Get user by ID.
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<UserRow>, StorageError> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, oauth_sub, created_at, last_seen_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StorageError::Query)
    }
}
