use crate::domain::{MemoryState, PropagationEvent};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Get memory state for a node
    async fn get_memory_state(
        &self,
        user_id: &str,
        node_id: &str,
    ) -> anyhow::Result<Option<MemoryState>>;

    /// Save or update memory state
    async fn save_memory_state(&self, state: &MemoryState) -> anyhow::Result<()>;

    /// Get all due memory states
    async fn get_due_states(
        &self,
        user_id: &str,
        due_before: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<MemoryState>>;

    /// Update energy for a node
    async fn update_energy(
        &self,
        user_id: &str,
        node_id: &str,
        new_energy: f64,
    ) -> anyhow::Result<()>;

    /// Log a propagation event
    async fn log_propagation(&self, event: &PropagationEvent) -> anyhow::Result<()>;

    /// Get session state
    async fn get_session_state(&self) -> anyhow::Result<Vec<String>>;

    /// Save session state
    async fn save_session_state(&self, node_ids: &[String]) -> anyhow::Result<()>;

    /// Clear session state
    async fn clear_session_state(&self) -> anyhow::Result<()>;

    /// Get user stat
    async fn get_stat(&self, key: &str) -> anyhow::Result<Option<String>>;

    /// Set user stat
    async fn set_stat(&self, key: &str, value: &str) -> anyhow::Result<()>;

    /// Get app setting
    async fn get_setting(&self, key: &str) -> anyhow::Result<Option<String>>;

    /// Set app setting
    async fn set_setting(&self, key: &str, value: &str) -> anyhow::Result<()>;
}
