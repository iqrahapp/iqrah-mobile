//! Sync repository for LWW sync operations.

use chrono::{DateTime, TimeZone, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use iqrah_backend_domain::{
    MemoryStateChange, SessionChange, SessionItemChange, SettingChange, SyncChanges,
    SyncCursorMemoryState, SyncCursorSession, SyncCursorSessionItem, SyncCursorSetting,
    SyncPullCursor,
};

use crate::StorageError;

/// Sync repository.
#[derive(Clone)]
pub struct SyncRepository {
    pool: PgPool,
}

impl SyncRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Register or update device metadata and last_seen_at.
    ///
    /// **Transaction Note**: This method does NOT require transaction wrapping because
    /// it performs a single INSERT...ON CONFLICT DO UPDATE statement, which is atomic
    /// by nature in PostgreSQL. The database guarantees that this operation either
    /// fully succeeds or fully fails, with no partial state possible.
    pub async fn touch_device(
        &self,
        user_id: Uuid,
        device_id: Uuid,
        device_os: Option<&str>,
        device_model: Option<&str>,
        app_version: Option<&str>,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO devices (id, user_id, os, device_model, app_version, last_seen_at)
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (id) DO UPDATE SET
                os = COALESCE(EXCLUDED.os, devices.os),
                device_model = COALESCE(EXCLUDED.device_model, devices.device_model),
                app_version = COALESCE(EXCLUDED.app_version, devices.app_version),
                last_seen_at = now()
            "#,
        )
        .bind(device_id)
        .bind(user_id)
        .bind(device_os)
        .bind(device_model)
        .bind(app_version)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Query)?;

        Ok(())
    }

    /// Apply sync changes with server-timestamp LWW in a single transaction.
    ///
    /// Returns `(applied, skipped)` counts:
    /// - `applied`: rows written (new inserts or LWW-winning updates)
    /// - `skipped`: rows rejected because `existing.updated_at >= incoming.updated_at`
    pub async fn apply_changes(
        &self,
        user_id: Uuid,
        device_id: Uuid,
        changes: &SyncChanges,
    ) -> Result<(u64, u64), StorageError> {
        // Begin transaction to ensure atomicity
        let mut tx = self.pool.begin().await.map_err(StorageError::Query)?;
        // Conflict policy: server-assigned timestamp decides LWW ordering.
        // Client-provided logical timestamps are ignored by the repository.
        let now = Utc::now();
        let mut applied: u64 = 0;
        let mut skipped: u64 = 0;

        // Apply settings
        for setting in &changes.settings {
            let rows = self
                .upsert_setting_tx(&mut tx, user_id, device_id, setting, now)
                .await?;
            if rows > 0 {
                applied += 1;
            } else {
                skipped += 1;
            }
        }

        // Apply memory states
        for state in &changes.memory_states {
            let rows = self
                .upsert_memory_state_tx(&mut tx, user_id, device_id, state, now)
                .await?;
            if rows > 0 {
                applied += 1;
            } else {
                skipped += 1;
            }
        }

        // Apply sessions
        for session in &changes.sessions {
            let rows = self
                .upsert_session_tx(&mut tx, user_id, device_id, session, now)
                .await?;
            if rows > 0 {
                applied += 1;
            } else {
                skipped += 1;
            }
        }

        // Apply session items
        for item in &changes.session_items {
            let rows = self
                .upsert_session_item_tx(&mut tx, user_id, device_id, item, now)
                .await?;
            if rows > 0 {
                applied += 1;
            } else {
                skipped += 1;
            }
        }

        // Commit transaction - if this fails, transaction auto-rolls back
        tx.commit().await.map_err(StorageError::Query)?;

        Ok((applied, skipped))
    }

    async fn upsert_setting_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        device_id: Uuid,
        setting: &SettingChange,
        now: DateTime<Utc>,
    ) -> Result<u64, StorageError> {
        let result = sqlx::query(
            r#"
            INSERT INTO user_settings (user_id, key, value, updated_at, updated_by_device)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, key) DO UPDATE SET
                value = EXCLUDED.value,
                updated_at = EXCLUDED.updated_at,
                updated_by_device = EXCLUDED.updated_by_device
            WHERE user_settings.updated_at < EXCLUDED.updated_at
            "#,
        )
        .bind(user_id)
        .bind(&setting.key)
        .bind(&setting.value)
        .bind(now)
        .bind(device_id)
        .execute(&mut **tx)
        .await
        .map_err(StorageError::Query)?;

        Ok(result.rows_affected())
    }

    async fn upsert_memory_state_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        device_id: Uuid,
        state: &MemoryStateChange,
        now: DateTime<Utc>,
    ) -> Result<u64, StorageError> {
        let last_reviewed = state
            .last_reviewed_at
            .map(|ts| Utc.timestamp_millis_opt(ts).unwrap());
        let next_review = state
            .next_review_at
            .map(|ts| Utc.timestamp_millis_opt(ts).unwrap());

        let result = sqlx::query(
            r#"
            INSERT INTO memory_states (user_id, node_id, energy, fsrs_stability, fsrs_difficulty,
                                       last_reviewed_at, next_review_at, updated_at, updated_by_device)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id, node_id) DO UPDATE SET
                energy = EXCLUDED.energy,
                fsrs_stability = EXCLUDED.fsrs_stability,
                fsrs_difficulty = EXCLUDED.fsrs_difficulty,
                last_reviewed_at = EXCLUDED.last_reviewed_at,
                next_review_at = EXCLUDED.next_review_at,
                updated_at = EXCLUDED.updated_at,
                updated_by_device = EXCLUDED.updated_by_device
            WHERE memory_states.updated_at < EXCLUDED.updated_at
            "#,
        )
        .bind(user_id)
        .bind(state.node_id)
        .bind(state.energy)
        .bind(state.fsrs_stability)
        .bind(state.fsrs_difficulty)
        .bind(last_reviewed)
        .bind(next_review)
        .bind(now)
        .bind(device_id)
        .execute(&mut **tx)
        .await
        .map_err(StorageError::Query)?;

        Ok(result.rows_affected())
    }

    async fn upsert_session_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        device_id: Uuid,
        session: &SessionChange,
        now: DateTime<Utc>,
    ) -> Result<u64, StorageError> {
        let started = Utc.timestamp_millis_opt(session.started_at).unwrap();
        let completed = session
            .completed_at
            .map(|ts| Utc.timestamp_millis_opt(ts).unwrap());

        let result = sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, goal_id, started_at, completed_at, items_completed, updated_at, updated_by_device)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                completed_at = EXCLUDED.completed_at,
                items_completed = EXCLUDED.items_completed,
                updated_at = EXCLUDED.updated_at,
                updated_by_device = EXCLUDED.updated_by_device
            WHERE sessions.updated_at < EXCLUDED.updated_at
            "#,
        )
        .bind(session.id)
        .bind(user_id)
        .bind(&session.goal_id)
        .bind(started)
        .bind(completed)
        .bind(session.items_completed)
        .bind(now)
        .bind(device_id)
        .execute(&mut **tx)
        .await
        .map_err(StorageError::Query)?;

        Ok(result.rows_affected())
    }

    async fn upsert_session_item_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: Uuid,
        device_id: Uuid,
        item: &SessionItemChange,
        now: DateTime<Utc>,
    ) -> Result<u64, StorageError> {
        let result = sqlx::query(
            r#"
            INSERT INTO session_items (id, session_id, user_id, node_id, exercise_type, grade, duration_ms, updated_at, updated_by_device)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                grade = EXCLUDED.grade,
                duration_ms = EXCLUDED.duration_ms,
                updated_at = EXCLUDED.updated_at,
                updated_by_device = EXCLUDED.updated_by_device
            WHERE session_items.updated_at < EXCLUDED.updated_at
            "#,
        )
        .bind(item.id)
        .bind(item.session_id)
        .bind(user_id)
        .bind(item.node_id)
        .bind(&item.exercise_type)
        .bind(item.grade)
        .bind(item.duration_ms)
        .bind(now)
        .bind(device_id)
        .execute(&mut **tx)
        .await
        .map_err(StorageError::Query)?;

        Ok(result.rows_affected())
    }

    /// Get changes since timestamp with pagination.
    /// Returns (SyncChanges, has_more, next_cursor).
    ///
    /// **Transaction Note**: This method does NOT require transaction wrapping because
    /// it only performs read-only SELECT queries. Multiple SELECT statements without
    /// transaction isolation is acceptable here because:
    /// 1. We're using timestamps for change tracking, not row versions
    /// 2. Slight inconsistencies between entity types are acceptable (will sync next round)
    /// 3. READ COMMITTED isolation (PostgreSQL default) provides sufficient consistency
    ///    for each individual SELECT query
    pub async fn get_changes_since(
        &self,
        user_id: Uuid,
        since_millis: i64,
        limit: usize,
        cursor: Option<&SyncPullCursor>,
    ) -> Result<(SyncChanges, bool, Option<SyncPullCursor>), StorageError> {
        let since = Utc.timestamp_millis_opt(since_millis).unwrap();
        let query_limit = (limit + 1) as i64; // Fetch one extra to detect if there are more
        let previous_cursor = cursor.cloned();

        // Get settings
        let settings_raw = if let Some(cursor) = cursor.as_ref().and_then(|c| c.settings.as_ref()) {
            let cursor_time = Utc.timestamp_millis_opt(cursor.updated_at).unwrap();
            sqlx::query_as::<_, SettingRow>(
                "SELECT key, value, updated_at FROM user_settings
                 WHERE user_id = $1 AND (updated_at, key) > ($2, $3)
                 ORDER BY updated_at, key LIMIT $4",
            )
            .bind(user_id)
            .bind(cursor_time)
            .bind(&cursor.key)
            .bind(query_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Query)?
        } else {
            sqlx::query_as::<_, SettingRow>(
                "SELECT key, value, updated_at FROM user_settings
                 WHERE user_id = $1 AND updated_at > $2
                 ORDER BY updated_at, key LIMIT $3",
            )
            .bind(user_id)
            .bind(since)
            .bind(query_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Query)?
        };

        let settings_has_more = settings_raw.len() > limit;
        let settings_cursor =
            settings_raw
                .get(limit.saturating_sub(1))
                .map(|row| SyncCursorSetting {
                    updated_at: row.updated_at.timestamp_millis(),
                    key: row.key.clone(),
                });
        let settings: Vec<SettingChange> = settings_raw
            .into_iter()
            .take(limit)
            .map(|r| SettingChange {
                key: r.key,
                value: r.value,
            })
            .collect();

        // Get memory states
        let memory_states_raw = if let Some(cursor) =
            cursor.as_ref().and_then(|c| c.memory_states.as_ref())
        {
            let cursor_time = Utc.timestamp_millis_opt(cursor.updated_at).unwrap();
            sqlx::query_as::<_, MemoryStateRow>(
                    "SELECT node_id, energy, fsrs_stability, fsrs_difficulty, last_reviewed_at, next_review_at, updated_at
                     FROM memory_states
                     WHERE user_id = $1 AND (updated_at, node_id) > ($2, $3)
                     ORDER BY updated_at, node_id LIMIT $4",
                )
                .bind(user_id)
                .bind(cursor_time)
                .bind(cursor.node_id)
                .bind(query_limit)
                .fetch_all(&self.pool)
                .await
                .map_err(StorageError::Query)?
        } else {
            sqlx::query_as::<_, MemoryStateRow>(
                    "SELECT node_id, energy, fsrs_stability, fsrs_difficulty, last_reviewed_at, next_review_at, updated_at
                     FROM memory_states WHERE user_id = $1 AND updated_at > $2
                     ORDER BY updated_at, node_id LIMIT $3",
                )
                .bind(user_id)
                .bind(since)
                .bind(query_limit)
                .fetch_all(&self.pool)
                .await
                .map_err(StorageError::Query)?
        };

        let memory_states_has_more = memory_states_raw.len() > limit;
        let memory_states_cursor =
            memory_states_raw
                .get(limit.saturating_sub(1))
                .map(|row| SyncCursorMemoryState {
                    updated_at: row.updated_at.timestamp_millis(),
                    node_id: row.node_id,
                });
        let memory_states: Vec<MemoryStateChange> = memory_states_raw
            .into_iter()
            .take(limit)
            .map(|r| MemoryStateChange {
                node_id: r.node_id,
                energy: r.energy,
                fsrs_stability: r.fsrs_stability,
                fsrs_difficulty: r.fsrs_difficulty,
                last_reviewed_at: r.last_reviewed_at.map(|t| t.timestamp_millis()),
                next_review_at: r.next_review_at.map(|t| t.timestamp_millis()),
            })
            .collect();

        // Get sessions
        let sessions_raw = if let Some(cursor) = cursor.as_ref().and_then(|c| c.sessions.as_ref()) {
            let cursor_time = Utc.timestamp_millis_opt(cursor.updated_at).unwrap();
            sqlx::query_as::<_, SessionRow>(
                "SELECT id, goal_id, started_at, completed_at, items_completed, updated_at
                 FROM sessions
                 WHERE user_id = $1 AND (updated_at, id) > ($2, $3)
                 ORDER BY updated_at, id LIMIT $4",
            )
            .bind(user_id)
            .bind(cursor_time)
            .bind(cursor.id)
            .bind(query_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Query)?
        } else {
            sqlx::query_as::<_, SessionRow>(
                "SELECT id, goal_id, started_at, completed_at, items_completed, updated_at
                 FROM sessions WHERE user_id = $1 AND updated_at > $2
                 ORDER BY updated_at, id LIMIT $3",
            )
            .bind(user_id)
            .bind(since)
            .bind(query_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Query)?
        };

        let sessions_has_more = sessions_raw.len() > limit;
        let sessions_cursor =
            sessions_raw
                .get(limit.saturating_sub(1))
                .map(|row| SyncCursorSession {
                    updated_at: row.updated_at.timestamp_millis(),
                    id: row.id,
                });
        let sessions: Vec<SessionChange> = sessions_raw
            .into_iter()
            .take(limit)
            .map(|r| SessionChange {
                id: r.id,
                goal_id: r.goal_id,
                started_at: r.started_at.timestamp_millis(),
                completed_at: r.completed_at.map(|t| t.timestamp_millis()),
                items_completed: r.items_completed,
            })
            .collect();

        // Get session items (direct query via user_id — no JOIN needed)
        let session_items_raw =
            if let Some(cursor) = cursor.as_ref().and_then(|c| c.session_items.as_ref()) {
                let cursor_time = Utc.timestamp_millis_opt(cursor.updated_at).unwrap();
                sqlx::query_as::<_, SessionItemRow>(
                    "SELECT id, session_id, node_id, exercise_type, grade, duration_ms, updated_at
                     FROM session_items
                     WHERE user_id = $1 AND (updated_at, id) > ($2, $3)
                     ORDER BY updated_at, id LIMIT $4",
                )
                .bind(user_id)
                .bind(cursor_time)
                .bind(cursor.id)
                .bind(query_limit)
                .fetch_all(&self.pool)
                .await
                .map_err(StorageError::Query)?
            } else {
                sqlx::query_as::<_, SessionItemRow>(
                    "SELECT id, session_id, node_id, exercise_type, grade, duration_ms, updated_at
                     FROM session_items
                     WHERE user_id = $1 AND updated_at > $2
                     ORDER BY updated_at, id LIMIT $3",
                )
                .bind(user_id)
                .bind(since)
                .bind(query_limit)
                .fetch_all(&self.pool)
                .await
                .map_err(StorageError::Query)?
            };

        let session_items_has_more = session_items_raw.len() > limit;
        let session_items_cursor =
            session_items_raw
                .get(limit.saturating_sub(1))
                .map(|row| SyncCursorSessionItem {
                    updated_at: row.updated_at.timestamp_millis(),
                    id: row.id,
                });
        let session_items: Vec<SessionItemChange> = session_items_raw
            .into_iter()
            .take(limit)
            .map(|r| SessionItemChange {
                id: r.id,
                session_id: r.session_id,
                node_id: r.node_id,
                exercise_type: r.exercise_type,
                grade: r.grade,
                duration_ms: r.duration_ms,
            })
            .collect();

        // Check if any category hit the limit
        let has_more = settings_has_more
            || memory_states_has_more
            || sessions_has_more
            || session_items_has_more;
        let next_cursor = if has_more {
            Some(SyncPullCursor {
                settings: settings_cursor.or_else(|| {
                    previous_cursor
                        .as_ref()
                        .and_then(|cursor| cursor.settings.clone())
                }),
                memory_states: memory_states_cursor.or_else(|| {
                    previous_cursor
                        .as_ref()
                        .and_then(|cursor| cursor.memory_states.clone())
                }),
                sessions: sessions_cursor.or_else(|| {
                    previous_cursor
                        .as_ref()
                        .and_then(|cursor| cursor.sessions.clone())
                }),
                session_items: session_items_cursor.or_else(|| {
                    previous_cursor
                        .as_ref()
                        .and_then(|cursor| cursor.session_items.clone())
                }),
            })
        } else {
            None
        };

        Ok((
            SyncChanges {
                settings,
                memory_states,
                sessions,
                session_items,
            },
            has_more,
            next_cursor,
        ))
    }
}

// Query result types
#[derive(sqlx::FromRow)]
struct SettingRow {
    key: String,
    value: serde_json::Value,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct MemoryStateRow {
    node_id: i64,
    energy: f32,
    fsrs_stability: Option<f32>,
    fsrs_difficulty: Option<f32>,
    last_reviewed_at: Option<DateTime<Utc>>,
    next_review_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    goal_id: Option<String>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    items_completed: i32,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct SessionItemRow {
    id: Uuid,
    session_id: Uuid,
    node_id: i64,
    exercise_type: String,
    grade: Option<i32>,
    duration_ms: Option<i32>,
    updated_at: DateTime<Utc>,
}
