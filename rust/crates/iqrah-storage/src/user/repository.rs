use super::models::{
    BanditArmRow, MemoryBasicsRow, MemoryStateRow, ParentEnergyRow, SessionItemRow, SessionRow,
    SessionStateRow, UserStatRow,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use iqrah_core::{
    scheduler_v2::{BanditArmState, MemoryBasics},
    MemoryState, PropagationEvent, Session, SessionItem, SessionSummary, UserRepository,
};
use sqlx::{query, query_as, Sqlite, SqlitePool, Transaction};
use std::collections::HashMap;

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get the underlying pool for transaction creation
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    // ========================================================================
    // Transaction-aware methods (for atomic operations)
    // ========================================================================

    /// Save memory state within an existing transaction
    pub async fn save_memory_state_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        state: &MemoryState,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO user_memory_states
             (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(user_id, content_key) DO UPDATE SET
                stability = excluded.stability,
                difficulty = excluded.difficulty,
                energy = excluded.energy,
                last_reviewed = excluded.last_reviewed,
                due_at = excluded.due_at,
                review_count = excluded.review_count",
        )
        .bind(&state.user_id)
        .bind(state.node_id)
        .bind(state.stability)
        .bind(state.difficulty)
        .bind(state.energy)
        .bind(state.last_reviewed.timestamp_millis())
        .bind(state.due_at.timestamp_millis())
        .bind(state.review_count as i64)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Update energy within an existing transaction
    pub async fn update_energy_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        user_id: &str,
        node_id: i64,
        new_energy: f64,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE user_memory_states SET energy = ? WHERE user_id = ? AND content_key = ?",
        )
        .bind(new_energy)
        .bind(user_id)
        .bind(node_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Log propagation within an existing transaction
    pub async fn log_propagation_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        event: &PropagationEvent,
    ) -> anyhow::Result<()> {
        // Insert event
        let result = sqlx::query(
            "INSERT INTO propagation_events (source_content_key, event_timestamp)
             VALUES (?, ?)",
        )
        .bind(event.source_node_id)
        .bind(event.event_timestamp.timestamp_millis())
        .execute(&mut **tx)
        .await?;

        let event_id = result.last_insert_rowid();

        // Insert details
        for detail in &event.details {
            sqlx::query(
                "INSERT INTO propagation_details (event_id, target_content_key, energy_change, reason)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(event_id)
            .bind(detail.target_node_id)
            .bind(detail.energy_change)
            .bind(&detail.reason)
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }

    /// Get all unique node IDs from user memory states (for integrity checking)
    pub async fn get_all_node_ids(&self, user_id: &str) -> anyhow::Result<Vec<i64>> {
        let rows = query_as::<_, (i64,)>(
            "SELECT DISTINCT content_key FROM user_memory_states WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    // ========================================================================
    // Sync helpers (Phase 2)
    // ========================================================================

    pub async fn get_memory_states_since(
        &self,
        user_id: &str,
        since_millis: i64,
    ) -> anyhow::Result<Vec<MemoryState>> {
        let rows = query_as::<_, MemoryStateRow>(
            "SELECT user_id, content_key, stability, difficulty, energy,
                    last_reviewed, due_at, review_count
             FROM user_memory_states
             WHERE user_id = ? AND last_reviewed > ?
             ORDER BY last_reviewed ASC",
        )
        .bind(user_id)
        .bind(since_millis)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| MemoryState {
                user_id: r.user_id,
                node_id: r.content_key,
                stability: r.stability,
                difficulty: r.difficulty,
                energy: r.energy,
                last_reviewed: DateTime::from_timestamp_millis(r.last_reviewed)
                    .unwrap_or_else(Utc::now),
                due_at: DateTime::from_timestamp_millis(r.due_at).unwrap_or_else(Utc::now),
                review_count: r.review_count as u32,
            })
            .collect())
    }

    pub async fn get_sessions_since(
        &self,
        user_id: &str,
        since_millis: i64,
    ) -> anyhow::Result<Vec<Session>> {
        let rows = query_as::<_, SessionRow>(
            "SELECT id, user_id, goal_id, started_at, completed_at, items_count, items_completed
             FROM sessions
             WHERE user_id = ? AND (started_at > ? OR completed_at > ?)
             ORDER BY started_at ASC",
        )
        .bind(user_id)
        .bind(since_millis)
        .bind(since_millis)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Session {
                id: r.id,
                user_id: r.user_id,
                goal_id: r.goal_id,
                started_at: DateTime::from_timestamp_millis(r.started_at)
                    .unwrap_or_else(Utc::now),
                completed_at: r.completed_at.and_then(DateTime::from_timestamp_millis),
                items_count: r.items_count as i32,
                items_completed: r.items_completed as i32,
            })
            .collect())
    }

    pub async fn get_session_items_since(
        &self,
        user_id: &str,
        since_millis: i64,
    ) -> anyhow::Result<Vec<SessionItem>> {
        let rows = query_as::<_, SessionItemRow>(
            "SELECT si.id, si.session_id, si.node_id, si.exercise_type, si.grade, si.duration_ms, si.completed_at
             FROM session_items si
             JOIN sessions s ON s.id = si.session_id
             WHERE s.user_id = ? AND si.completed_at IS NOT NULL AND si.completed_at > ?
             ORDER BY si.completed_at ASC",
        )
        .bind(user_id)
        .bind(since_millis)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SessionItem {
                id: r.id,
                session_id: r.session_id,
                node_id: r.node_id,
                exercise_type: r.exercise_type,
                grade: r.grade as i32,
                duration_ms: r.duration_ms,
                completed_at: r.completed_at.and_then(DateTime::from_timestamp_millis),
            })
            .collect())
    }

    pub async fn upsert_memory_state_if_newer(&self, state: &MemoryState) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO user_memory_states
             (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(user_id, content_key) DO UPDATE SET
                stability = excluded.stability,
                difficulty = excluded.difficulty,
                energy = excluded.energy,
                last_reviewed = excluded.last_reviewed,
                due_at = excluded.due_at,
                review_count = user_memory_states.review_count
             WHERE excluded.last_reviewed > user_memory_states.last_reviewed",
        )
        .bind(&state.user_id)
        .bind(state.node_id)
        .bind(state.stability)
        .bind(state.difficulty)
        .bind(state.energy)
        .bind(state.last_reviewed.timestamp_millis())
        .bind(state.due_at.timestamp_millis())
        .bind(state.review_count as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_session_if_newer(&self, session: &Session) -> anyhow::Result<()> {
        let existing = self.get_session(&session.id).await?;

        let incoming_updated = session
            .completed_at
            .unwrap_or(session.started_at)
            .timestamp_millis();

        if let Some(existing) = existing {
            let existing_updated = existing
                .completed_at
                .unwrap_or(existing.started_at)
                .timestamp_millis();

            if incoming_updated <= existing_updated {
                return Ok(());
            }

            query(
                "UPDATE sessions SET goal_id = ?, started_at = ?, completed_at = ?, items_count = ?, items_completed = ?
                 WHERE id = ?",
            )
            .bind(&session.goal_id)
            .bind(session.started_at.timestamp_millis())
            .bind(session.completed_at.map(|t| t.timestamp_millis()))
            .bind(session.items_count)
            .bind(session.items_completed)
            .bind(&session.id)
            .execute(&self.pool)
            .await?;
        } else {
            self.create_session(session).await?;
        }

        Ok(())
    }

    pub async fn insert_session_item_if_absent(
        &self,
        item: &SessionItem,
    ) -> anyhow::Result<bool> {
        let completed_at = item.completed_at.map(|t| t.timestamp_millis());
        let existing = query_as::<_, (i64,)>(
            "SELECT id FROM session_items
             WHERE session_id = ? AND node_id = ? AND exercise_type = ? AND completed_at = ?
             LIMIT 1",
        )
        .bind(&item.session_id)
        .bind(item.node_id)
        .bind(&item.exercise_type)
        .bind(completed_at)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_none() {
            self.insert_session_item(item).await?;
            return Ok(true);
        }

        Ok(false)
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn get_memory_state(
        &self,
        user_id: &str,
        node_id: i64,
    ) -> anyhow::Result<Option<MemoryState>> {
        let row = query_as::<_, MemoryStateRow>(
            "SELECT user_id, content_key, stability, difficulty, energy,
                    last_reviewed, due_at, review_count
             FROM user_memory_states
             WHERE user_id = ? AND content_key = ?",
        )
        .bind(user_id)
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| MemoryState {
            user_id: r.user_id,
            node_id: r.content_key,
            stability: r.stability,
            difficulty: r.difficulty,
            energy: r.energy,
            last_reviewed: DateTime::from_timestamp_millis(r.last_reviewed)
                .unwrap_or_else(Utc::now),
            due_at: DateTime::from_timestamp_millis(r.due_at).unwrap_or_else(Utc::now),
            review_count: r.review_count as u32,
        }))
    }

    async fn save_memory_state(&self, state: &MemoryState) -> anyhow::Result<()> {
        query(
            "INSERT INTO user_memory_states
             (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(user_id, content_key) DO UPDATE SET
                stability = excluded.stability,
                difficulty = excluded.difficulty,
                energy = excluded.energy,
                last_reviewed = excluded.last_reviewed,
                due_at = excluded.due_at,
                review_count = excluded.review_count"
        )
        .bind(&state.user_id)
        .bind(state.node_id)
        .bind(state.stability)
        .bind(state.difficulty)
        .bind(state.energy)
        .bind(state.last_reviewed.timestamp_millis())
        .bind(state.due_at.timestamp_millis())
        .bind(state.review_count as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_due_states(
        &self,
        user_id: &str,
        due_before: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<MemoryState>> {
        let rows = query_as::<_, MemoryStateRow>(
            "SELECT user_id, content_key, stability, difficulty, energy,
                    last_reviewed, due_at, review_count
             FROM user_memory_states
             WHERE user_id = ? AND due_at <= ?
             ORDER BY due_at ASC
             LIMIT ?",
        )
        .bind(user_id)
        .bind(due_before.timestamp_millis())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| MemoryState {
                user_id: r.user_id,
                node_id: r.content_key,
                stability: r.stability,
                difficulty: r.difficulty,
                energy: r.energy,
                last_reviewed: DateTime::from_timestamp_millis(r.last_reviewed)
                    .unwrap_or_else(Utc::now),
                due_at: DateTime::from_timestamp_millis(r.due_at).unwrap_or_else(Utc::now),
                review_count: r.review_count as u32,
            })
            .collect())
    }

    async fn update_energy(
        &self,
        user_id: &str,
        node_id: i64,
        new_energy: f64,
    ) -> anyhow::Result<()> {
        query("UPDATE user_memory_states SET energy = ? WHERE user_id = ? AND content_key = ?")
            .bind(new_energy)
            .bind(user_id)
            .bind(node_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn log_propagation(&self, event: &PropagationEvent) -> anyhow::Result<()> {
        // Insert event
        let result = query(
            "INSERT INTO propagation_events (source_content_key, event_timestamp)
             VALUES (?, ?)",
        )
        .bind(event.source_node_id)
        .bind(event.event_timestamp.timestamp_millis())
        .execute(&self.pool)
        .await?;

        let event_id = result.last_insert_rowid();

        // Insert details
        for detail in &event.details {
            query(
                "INSERT INTO propagation_details (event_id, target_content_key, energy_change, reason)
                 VALUES (?, ?, ?, ?)"
            )
            .bind(event_id)
            .bind(detail.target_node_id)
            .bind(detail.energy_change)
            .bind(&detail.reason)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn get_session_state(&self) -> anyhow::Result<Vec<i64>> {
        let rows = query_as::<_, SessionStateRow>(
            "SELECT content_key, session_order FROM session_state ORDER BY session_order ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.content_key).collect())
    }

    async fn save_session_state(&self, node_ids: &[i64]) -> anyhow::Result<()> {
        // Clear existing
        self.clear_session_state().await?;

        // Insert new
        for (idx, &node_id) in node_ids.iter().enumerate() {
            query("INSERT INTO session_state (content_key, session_order) VALUES (?, ?)")
                .bind(node_id)
                .bind(idx as i64)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    async fn clear_session_state(&self) -> anyhow::Result<()> {
        query("DELETE FROM session_state")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn create_session(&self, session: &Session) -> anyhow::Result<()> {
        query(
            "INSERT INTO sessions (id, user_id, goal_id, started_at, completed_at, items_count, items_completed)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&session.id)
        .bind(&session.user_id)
        .bind(&session.goal_id)
        .bind(session.started_at.timestamp_millis())
        .bind(session.completed_at.map(|t| t.timestamp_millis()))
        .bind(session.items_count)
        .bind(session.items_completed)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_session(&self, session_id: &str) -> anyhow::Result<Option<Session>> {
        let row = query_as::<_, SessionRow>(
            "SELECT id, user_id, goal_id, started_at, completed_at, items_count, items_completed
             FROM sessions
             WHERE id = ?",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Session {
            id: r.id,
            user_id: r.user_id,
            goal_id: r.goal_id,
            started_at: DateTime::from_timestamp_millis(r.started_at).unwrap_or_else(Utc::now),
            completed_at: r.completed_at.and_then(DateTime::from_timestamp_millis),
            items_count: r.items_count as i32,
            items_completed: r.items_completed as i32,
        }))
    }

    async fn get_active_session(&self, user_id: &str) -> anyhow::Result<Option<Session>> {
        let row = query_as::<_, SessionRow>(
            "SELECT id, user_id, goal_id, started_at, completed_at, items_count, items_completed
             FROM sessions
             WHERE user_id = ? AND completed_at IS NULL
             ORDER BY started_at DESC
             LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Session {
            id: r.id,
            user_id: r.user_id,
            goal_id: r.goal_id,
            started_at: DateTime::from_timestamp_millis(r.started_at).unwrap_or_else(Utc::now),
            completed_at: r.completed_at.and_then(DateTime::from_timestamp_millis),
            items_count: r.items_count as i32,
            items_completed: r.items_completed as i32,
        }))
    }

    async fn update_session_progress(
        &self,
        session_id: &str,
        items_completed: i32,
    ) -> anyhow::Result<()> {
        query("UPDATE sessions SET items_completed = ? WHERE id = ?")
            .bind(items_completed)
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn complete_session(&self, session_id: &str) -> anyhow::Result<()> {
        let now = Utc::now().timestamp_millis();
        query("UPDATE sessions SET completed_at = ? WHERE id = ?")
            .bind(now)
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn insert_session_item(&self, item: &SessionItem) -> anyhow::Result<()> {
        query(
            "INSERT INTO session_items (session_id, node_id, exercise_type, grade, duration_ms, completed_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&item.session_id)
        .bind(item.node_id)
        .bind(&item.exercise_type)
        .bind(item.grade)
        .bind(item.duration_ms)
        .bind(item.completed_at.map(|t| t.timestamp_millis()))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_session_summary(&self, session_id: &str) -> anyhow::Result<SessionSummary> {
        let row = query_as::<_, SessionRow>(
            "SELECT id, user_id, goal_id, started_at, completed_at, items_count, items_completed
             FROM sessions
             WHERE id = ?",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(anyhow::anyhow!("Session not found"));
        };

        let started_at = DateTime::from_timestamp_millis(row.started_at).unwrap_or_else(Utc::now);
        let completed_at = row
            .completed_at
            .and_then(DateTime::from_timestamp_millis)
            .unwrap_or_else(Utc::now);

        let duration_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds()
            .max(0);

        let grade_rows = query_as::<_, (i64, i64)>(
            "SELECT grade, COUNT(*) FROM session_items WHERE session_id = ? GROUP BY grade",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        let mut again_count = 0;
        let mut hard_count = 0;
        let mut good_count = 0;
        let mut easy_count = 0;

        for (grade, count) in grade_rows {
            match grade {
                1 => again_count = count as i32,
                2 => hard_count = count as i32,
                3 => good_count = count as i32,
                4 => easy_count = count as i32,
                _ => {}
            }
        }

        Ok(SessionSummary {
            session_id: row.id,
            items_count: row.items_count as i32,
            items_completed: row.items_completed as i32,
            duration_ms,
            again_count,
            hard_count,
            good_count,
            easy_count,
        })
    }

    async fn get_stat(&self, key: &str) -> anyhow::Result<Option<String>> {
        let row = query_as::<_, UserStatRow>("SELECT key, value FROM user_stats WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.value))
    }

    async fn set_stat(&self, key: &str, value: &str) -> anyhow::Result<()> {
        query(
            "INSERT INTO user_stats (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_setting(&self, key: &str) -> anyhow::Result<Option<String>> {
        let row = query_as::<_, UserStatRow>("SELECT key, value FROM app_settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.value))
    }

    async fn set_setting(&self, key: &str, value: &str) -> anyhow::Result<()> {
        query(
            "INSERT INTO app_settings (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // Task 3.1: Atomic Review Saving (Transaction Wrapping)
    // ========================================================================

    async fn save_review_atomic(
        &self,
        user_id: &str,
        state: &MemoryState,
        energy_updates: Vec<(i64, f64)>,
        propagation_event: Option<PropagationEvent>,
    ) -> anyhow::Result<()> {
        // Begin transaction
        let mut tx = self.pool.begin().await?;

        // 1. Save the updated memory state
        Self::save_memory_state_in_tx(&mut tx, state).await?;

        // 2. Apply energy updates to target nodes
        for (node_id, new_energy) in energy_updates {
            Self::update_energy_in_tx(&mut tx, user_id, node_id, new_energy).await?;
        }

        // 3. Log propagation event if provided
        if let Some(ref event) = propagation_event {
            Self::log_propagation_in_tx(&mut tx, event).await?;
        }

        // Commit transaction - if any step failed, we would have returned early
        // and the transaction would auto-rollback on drop
        tx.commit().await?;

        Ok(())
    }

    async fn save_memory_states_batch(&self, states: &[MemoryState]) -> anyhow::Result<()> {
        if states.is_empty() {
            return Ok(());
        }

        // Use a transaction for atomicity and better performance
        let mut tx = self.pool.begin().await?;

        for state in states {
            Self::save_memory_state_in_tx(&mut tx, state).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // ========================================================================
    // Scheduler v2.0 Methods
    // ========================================================================

    async fn get_parent_energies(
        &self,
        user_id: &str,
        parent_ids: &[i64],
    ) -> anyhow::Result<HashMap<i64, f32>> {
        if parent_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut result: HashMap<i64, f32> = HashMap::new();

        // SQLite parameter limit is ~999, so chunk into batches of 500
        const CHUNK_SIZE: usize = 500;

        for chunk in parent_ids.chunks(CHUNK_SIZE) {
            // Build parameterized query
            let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            let sql = format!(
                "SELECT content_key AS node_id, CAST(energy AS REAL) as energy
                 FROM user_memory_states
                 WHERE user_id = ? AND content_key IN ({})",
                placeholders
            );

            let mut query = query_as::<_, ParentEnergyRow>(&sql);
            query = query.bind(user_id);
            for node_id in chunk {
                query = query.bind(node_id);
            }

            let rows = query.fetch_all(&self.pool).await?;

            // Add to result map
            for row in rows {
                result.insert(row.node_id, row.energy);
            }
        }

        Ok(result)
    }

    async fn get_memory_basics(
        &self,
        user_id: &str,
        node_ids: &[i64],
    ) -> anyhow::Result<HashMap<i64, MemoryBasics>> {
        if node_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut result: HashMap<i64, MemoryBasics> = HashMap::new();

        // SQLite parameter limit is ~999, so chunk into batches of 500
        const CHUNK_SIZE: usize = 500;

        for chunk in node_ids.chunks(CHUNK_SIZE) {
            // Build parameterized query
            let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            let sql = format!(
                "SELECT content_key AS node_id,
                        CAST(energy AS REAL) as energy,
                        due_at as next_due_ts
                 FROM user_memory_states
                 WHERE user_id = ? AND content_key IN ({})",
                placeholders
            );

            let mut query = query_as::<_, MemoryBasicsRow>(&sql);
            query = query.bind(user_id);
            for node_id in chunk {
                query = query.bind(node_id);
            }

            let rows = query.fetch_all(&self.pool).await?;

            // Add to result map
            for row in rows {
                result.insert(
                    row.node_id,
                    MemoryBasics {
                        energy: row.energy,
                        next_due_ts: row.next_due_ts,
                    },
                );
            }
        }

        Ok(result)
    }

    // ========================================================================
    // Scheduler v2.1 Bandit Methods
    // ========================================================================

    async fn get_bandit_arms(
        &self,
        user_id: &str,
        goal_group: &str,
    ) -> anyhow::Result<Vec<BanditArmState>> {
        let rows = query_as::<_, BanditArmRow>(
            "SELECT profile_name, successes, failures
             FROM user_bandit_state
             WHERE user_id = ? AND goal_group = ?",
        )
        .bind(user_id)
        .bind(goal_group)
        .fetch_all(&self.pool)
        .await?;

        // Convert rows to BanditArmState
        // Note: We need to parse profile_name string to ProfileName enum
        use iqrah_core::scheduler_v2::ProfileName;

        let mut arms = Vec::new();
        for row in rows {
            if let Some(profile_name) = ProfileName::parse_str(&row.profile_name) {
                arms.push(BanditArmState {
                    profile_name,
                    successes: row.successes,
                    failures: row.failures,
                });
            }
        }

        Ok(arms)
    }

    async fn update_bandit_arm(
        &self,
        user_id: &str,
        goal_group: &str,
        profile_name: &str,
        successes: f32,
        failures: f32,
    ) -> anyhow::Result<()> {
        // Get current timestamp in milliseconds
        let now_ms = Utc::now().timestamp_millis();

        query(
            "INSERT INTO user_bandit_state (user_id, goal_group, profile_name, successes, failures, last_updated)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT (user_id, goal_group, profile_name)
             DO UPDATE SET
                successes = excluded.successes,
                failures = excluded.failures,
                last_updated = excluded.last_updated",
        )
        .bind(user_id)
        .bind(goal_group)
        .bind(profile_name)
        .bind(successes)
        .bind(failures)
        .bind(now_ms)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
