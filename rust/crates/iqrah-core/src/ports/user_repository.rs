use crate::domain::{MemoryState, PropagationEvent, Session, SessionItem, SessionSummary};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[cfg_attr(any(test, feature = "testing"), mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Get memory state for a node
    async fn get_memory_state(
        &self,
        user_id: &str,
        node_id: i64,
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
        node_id: i64,
        new_energy: f64,
    ) -> anyhow::Result<()>;

    /// Log a propagation event
    async fn log_propagation(&self, event: &PropagationEvent) -> anyhow::Result<()>;

    /// Get session state
    async fn get_session_state(&self) -> anyhow::Result<Vec<i64>>;

    /// Save session state
    async fn save_session_state(&self, node_ids: &[i64]) -> anyhow::Result<()>;

    /// Clear session state
    async fn clear_session_state(&self) -> anyhow::Result<()>;

    /// Create a new session record
    async fn create_session(&self, session: &Session) -> anyhow::Result<()>;

    /// Get a session by ID
    async fn get_session(&self, session_id: &str) -> anyhow::Result<Option<Session>>;

    /// Get the active (incomplete) session for a user
    async fn get_active_session(&self, user_id: &str) -> anyhow::Result<Option<Session>>;

    /// Update session progress (items completed)
    async fn update_session_progress(
        &self,
        session_id: &str,
        items_completed: i32,
    ) -> anyhow::Result<()>;

    /// Mark session as completed
    async fn complete_session(&self, session_id: &str) -> anyhow::Result<()>;

    /// Insert a session item (result)
    async fn insert_session_item(&self, item: &SessionItem) -> anyhow::Result<()>;

    /// Get a summary for a session
    async fn get_session_summary(&self, session_id: &str) -> anyhow::Result<SessionSummary>;

    /// Get user stat
    async fn get_stat(&self, key: &str) -> anyhow::Result<Option<String>>;

    /// Set user stat
    async fn set_stat(&self, key: &str, value: &str) -> anyhow::Result<()>;

    /// Get app setting
    async fn get_setting(&self, key: &str) -> anyhow::Result<Option<String>>;

    /// Set app setting
    async fn set_setting(&self, key: &str, value: &str) -> anyhow::Result<()>;

    // ========================================================================
    // Atomic Review Saving (Task 3.1: Transaction Wrapping)
    // ========================================================================

    /// Atomically save a review with energy propagation
    ///
    /// This method wraps all database operations in a single transaction:
    /// 1. Save the updated memory state
    /// 2. Update energies for all propagation targets
    /// 3. Log the propagation event
    ///
    /// If any operation fails, all changes are rolled back.
    ///
    /// # Arguments
    /// * `state` - The memory state to save
    /// * `energy_updates` - Vec of (node_id, new_energy) pairs to update
    /// * `propagation_event` - Optional propagation event to log (owned for mockall)
    ///
    /// # Returns
    /// Ok(()) if all operations succeed, Err if any fail (with rollback)
    async fn save_review_atomic(
        &self,
        user_id: &str,
        state: &MemoryState,
        energy_updates: Vec<(i64, f64)>,
        propagation_event: Option<PropagationEvent>,
    ) -> anyhow::Result<()>;

    /// Batch save multiple memory states atomically.
    ///
    /// Used for initial placement and bulk imports where many states
    /// need to be written efficiently.
    ///
    /// # Arguments
    /// * `states` - Memory states to save (all for the same user)
    ///
    /// # Returns
    /// Ok(()) if all states saved successfully
    async fn save_memory_states_batch(&self, states: &[MemoryState]) -> anyhow::Result<()>;

    // ========================================================================
    // Scheduler v2.0 Methods
    // ========================================================================

    /// Get energies for a list of parent nodes
    ///
    /// Returns a map of node_id -> energy value for all specified nodes.
    /// Nodes without memory states are not included in the map (treated as 0.0 by caller).
    ///
    /// # Arguments
    /// * `user_id` - The user ID
    /// * `node_ids` - The parent nodes to get energies for
    ///
    /// # Returns
    /// HashMap mapping node_id to energy value (0.0-1.0)
    async fn get_parent_energies(
        &self,
        user_id: &str,
        parent_ids: &[i64],
    ) -> anyhow::Result<std::collections::HashMap<i64, f32>>;

    /// Get memory basics (energy + next_due_ts) for a list of nodes
    ///
    /// Returns a map of node_id -> MemoryBasics for all specified nodes.
    /// Nodes without memory states are not included in the map (treated as defaults by caller).
    ///
    /// # Arguments
    /// * `user_id` - The user ID
    /// * `node_ids` - The nodes to get memory basics for
    ///
    /// # Returns
    /// HashMap mapping node_id to MemoryBasics (energy + next_due_ts)
    async fn get_memory_basics(
        &self,
        user_id: &str,
        node_ids: &[i64],
    ) -> anyhow::Result<std::collections::HashMap<i64, crate::scheduler_v2::MemoryBasics>>;

    // ========================================================================
    // Scheduler v2.1 Bandit Methods
    // ========================================================================

    /// Get bandit arms for a user and goal group
    ///
    /// Returns all profile states for the given (user_id, goal_group).
    /// If no states exist, returns empty vec (caller should initialize).
    async fn get_bandit_arms(
        &self,
        user_id: &str,
        goal_group: &str,
    ) -> anyhow::Result<Vec<crate::scheduler_v2::BanditArmState>>;

    /// Update a bandit arm state
    ///
    /// Upserts the (successes, failures) for the given (user_id, goal_group, profile_name).
    async fn update_bandit_arm(
        &self,
        user_id: &str,
        goal_group: &str,
        profile_name: &str,
        successes: f32,
        failures: f32,
    ) -> anyhow::Result<()>;
}
