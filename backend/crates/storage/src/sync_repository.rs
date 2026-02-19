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

    /// Apply sync changes with LWW in a single transaction.
    ///
    /// Returns `(applied, skipped)` counts:
    /// - `applied`: rows written (new inserts or LWW-winning updates)
    /// - `skipped`: rows the server already had a newer version of (LWW rejected)
    pub async fn apply_changes(
        &self,
        user_id: Uuid,
        device_id: Uuid,
        changes: &SyncChanges,
    ) -> Result<(u64, u64), StorageError> {
        // Begin transaction to ensure atomicity
        let mut tx = self.pool.begin().await.map_err(StorageError::Query)?;
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
        let query_limit = (limit + 1) as i64; // Fetch one extra per entity to detect if there are more
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

        let mut settings_index = 0usize;
        let mut memory_states_index = 0usize;
        let mut sessions_index = 0usize;
        let mut session_items_index = 0usize;

        let mut settings = Vec::new();
        let mut memory_states = Vec::new();
        let mut sessions = Vec::new();
        let mut session_items = Vec::new();

        while settings.len() + memory_states.len() + sessions.len() + session_items.len() < limit {
            enum NextEntity {
                Settings,
                MemoryStates,
                Sessions,
                SessionItems,
            }

            let mut next_entity: Option<(i64, i32, NextEntity)> = None;

            if let Some(row) = settings_raw.get(settings_index) {
                next_entity = Some((row.updated_at.timestamp_millis(), 0, NextEntity::Settings));
            }
            if let Some(row) = memory_states_raw.get(memory_states_index) {
                let candidate = (
                    row.updated_at.timestamp_millis(),
                    1,
                    NextEntity::MemoryStates,
                );
                if next_entity
                    .as_ref()
                    .map(|current| {
                        candidate.0 < current.0
                            || (candidate.0 == current.0 && candidate.1 < current.1)
                    })
                    .unwrap_or(true)
                {
                    next_entity = Some(candidate);
                }
            }
            if let Some(row) = sessions_raw.get(sessions_index) {
                let candidate = (row.updated_at.timestamp_millis(), 2, NextEntity::Sessions);
                if next_entity
                    .as_ref()
                    .map(|current| {
                        candidate.0 < current.0
                            || (candidate.0 == current.0 && candidate.1 < current.1)
                    })
                    .unwrap_or(true)
                {
                    next_entity = Some(candidate);
                }
            }
            if let Some(row) = session_items_raw.get(session_items_index) {
                let candidate = (
                    row.updated_at.timestamp_millis(),
                    3,
                    NextEntity::SessionItems,
                );
                if next_entity
                    .as_ref()
                    .map(|current| {
                        candidate.0 < current.0
                            || (candidate.0 == current.0 && candidate.1 < current.1)
                    })
                    .unwrap_or(true)
                {
                    next_entity = Some(candidate);
                }
            }

            let Some((_, _, entity)) = next_entity else {
                break;
            };

            match entity {
                NextEntity::Settings => {
                    let row = &settings_raw[settings_index];
                    settings.push(SettingChange {
                        key: row.key.clone(),
                        value: row.value.clone(),
                        client_updated_at: row.updated_at.timestamp_millis(),
                    });
                    settings_index += 1;
                }
                NextEntity::MemoryStates => {
                    let row = &memory_states_raw[memory_states_index];
                    memory_states.push(MemoryStateChange {
                        node_id: row.node_id,
                        energy: row.energy,
                        fsrs_stability: row.fsrs_stability,
                        fsrs_difficulty: row.fsrs_difficulty,
                        last_reviewed_at: row.last_reviewed_at.map(|t| t.timestamp_millis()),
                        next_review_at: row.next_review_at.map(|t| t.timestamp_millis()),
                        client_updated_at: row.updated_at.timestamp_millis(),
                    });
                    memory_states_index += 1;
                }
                NextEntity::Sessions => {
                    let row = &sessions_raw[sessions_index];
                    sessions.push(SessionChange {
                        id: row.id,
                        goal_id: row.goal_id.clone(),
                        started_at: row.started_at.timestamp_millis(),
                        completed_at: row.completed_at.map(|t| t.timestamp_millis()),
                        items_completed: row.items_completed,
                        client_updated_at: row.updated_at.timestamp_millis(),
                    });
                    sessions_index += 1;
                }
                NextEntity::SessionItems => {
                    let row = &session_items_raw[session_items_index];
                    session_items.push(SessionItemChange {
                        id: row.id,
                        session_id: row.session_id,
                        node_id: row.node_id,
                        exercise_type: row.exercise_type.clone(),
                        grade: row.grade,
                        duration_ms: row.duration_ms,
                        client_updated_at: row.updated_at.timestamp_millis(),
                    });
                    session_items_index += 1;
                }
            }
        }

        let has_more = settings_index < settings_raw.len()
            || memory_states_index < memory_states_raw.len()
            || sessions_index < sessions_raw.len()
            || session_items_index < session_items_raw.len();

        let settings_cursor = settings.last().map(|setting| SyncCursorSetting {
            updated_at: setting.client_updated_at,
            key: setting.key.clone(),
        });
        let memory_states_cursor = memory_states.last().map(|state| SyncCursorMemoryState {
            updated_at: state.client_updated_at,
            node_id: state.node_id,
        });
        let sessions_cursor = sessions.last().map(|session| SyncCursorSession {
            updated_at: session.client_updated_at,
            id: session.id,
        });
        let session_items_cursor = session_items.last().map(|item| SyncCursorSessionItem {
            updated_at: item.client_updated_at,
            id: item.id,
        });

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
