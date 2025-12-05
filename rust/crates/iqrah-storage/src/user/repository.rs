use super::models::{
    BanditArmRow, MemoryBasicsRow, MemoryStateRow, ParentEnergyRow, SessionStateRow, UserStatRow,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use iqrah_core::{
    scheduler_v2::{BanditArmState, MemoryBasics},
    MemoryState, PropagationEvent, UserRepository,
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
        propagation_event: Option<&PropagationEvent>,
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
        if let Some(event) = propagation_event {
            Self::log_propagation_in_tx(&mut tx, event).await?;
        }

        // Commit transaction - if any step failed, we would have returned early
        // and the transaction would auto-rollback on drop
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
