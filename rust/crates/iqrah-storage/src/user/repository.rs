use super::models::{MemoryStateRow, SessionStateRow, UserStatRow};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use iqrah_core::{MemoryState, PropagationEvent, UserRepository};
use sqlx::{query, query_as, SqlitePool};

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn get_memory_state(
        &self,
        user_id: &str,
        node_id: &str,
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
            node_id: r.content_key, // Map content_key to node_id for domain model
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
        .bind(&state.node_id)
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
                node_id: r.content_key, // Map content_key to node_id
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
        node_id: &str,
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
        .bind(&event.source_node_id)
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
            .bind(&detail.target_node_id)
            .bind(detail.energy_change)
            .bind(&detail.reason)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn get_session_state(&self) -> anyhow::Result<Vec<String>> {
        let rows = query_as::<_, SessionStateRow>(
            "SELECT content_key, session_order FROM session_state ORDER BY session_order ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.content_key).collect())
    }

    async fn save_session_state(&self, node_ids: &[String]) -> anyhow::Result<()> {
        // Clear existing
        self.clear_session_state().await?;

        // Insert new
        for (idx, node_id) in node_ids.iter().enumerate() {
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
}
