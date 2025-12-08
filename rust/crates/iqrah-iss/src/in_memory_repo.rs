//! In-memory implementation of UserRepository for fast simulation.
//!
//! Uses HashMap-backed storage behind RwLock for thread safety.
//! Implements the same trait that `iqrah-storage` uses for SQLite.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use iqrah_core::domain::{MemoryState, PropagationEvent};
use iqrah_core::ports::UserRepository;
use iqrah_core::scheduler_v2::bandit::BanditArmState;
use iqrah_core::scheduler_v2::profiles::ProfileName;
use iqrah_core::scheduler_v2::MemoryBasics;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory user repository for simulation.
///
/// All data is stored in RAM using HashMaps protected by RwLock.
/// Supports multiple concurrent users.
pub struct InMemoryUserRepository {
    /// Memory states indexed by (user_id, node_id)
    memory_states: RwLock<HashMap<(String, i64), MemoryState>>,

    /// Bandit arm states indexed by (user_id, goal_group, profile_name)
    bandit_arms: RwLock<HashMap<(String, String, String), (f32, f32)>>,

    /// Session state (current session node IDs)
    session_state: RwLock<Vec<i64>>,

    /// User stats indexed by key
    stats: RwLock<HashMap<String, String>>,

    /// App settings indexed by key
    settings: RwLock<HashMap<String, String>>,

    /// Propagation events log (optional, for debugging)
    propagation_log: RwLock<Vec<PropagationEvent>>,
}

impl InMemoryUserRepository {
    /// Create a new empty in-memory repository.
    pub fn new() -> Self {
        Self {
            memory_states: RwLock::new(HashMap::new()),
            bandit_arms: RwLock::new(HashMap::new()),
            session_state: RwLock::new(Vec::new()),
            stats: RwLock::new(HashMap::new()),
            settings: RwLock::new(HashMap::new()),
            propagation_log: RwLock::new(Vec::new()),
        }
    }

    /// Get all memory states for a user (useful for metrics computation).
    pub fn get_all_states_for_user(&self, user_id: &str) -> HashMap<i64, MemoryState> {
        let states = self.memory_states.read().unwrap();
        states
            .iter()
            .filter(|((uid, _), _)| uid == user_id)
            .map(|((_, node_id), state)| (*node_id, state.clone()))
            .collect()
    }

    /// Get stability values for all nodes for a user.
    pub fn get_stabilities_for_user(&self, user_id: &str) -> HashMap<i64, f64> {
        let states = self.memory_states.read().unwrap();
        states
            .iter()
            .filter(|((uid, _), _)| uid == user_id)
            .map(|((_, node_id), state)| (*node_id, state.stability))
            .collect()
    }

    /// Clear all data for a user (useful for resetting between simulations).
    pub fn clear_user(&self, user_id: &str) {
        {
            let mut states = self.memory_states.write().unwrap();
            states.retain(|(uid, _), _| uid != user_id);
        }
        {
            let mut arms = self.bandit_arms.write().unwrap();
            arms.retain(|(uid, _, _), _| uid != user_id);
        }
    }

    /// Pre-initialize a memory state (for prior knowledge).
    pub fn initialize_state(&self, state: MemoryState) {
        let mut states = self.memory_states.write().unwrap();
        states.insert((state.user_id.clone(), state.node_id), state);
    }

    // =========================================================================
    // Synchronous batch methods for ISS performance optimization
    // These bypass async overhead for in-memory operations
    // =========================================================================

    /// Get all memory states for specified nodes (synchronous batch).
    /// This is O(n) with a single lock acquisition instead of O(n) async calls.
    pub fn get_memory_states_batch_sync(
        &self,
        user_id: &str,
        node_ids: &[i64],
    ) -> HashMap<i64, MemoryState> {
        let states = self.memory_states.read().unwrap();
        let mut result = HashMap::with_capacity(node_ids.len());
        for &node_id in node_ids {
            if let Some(state) = states.get(&(user_id.to_string(), node_id)) {
                result.insert(node_id, state.clone());
            }
        }
        result
    }

    /// Get memory basics for all nodes in one batch (synchronous).
    pub fn get_memory_basics_sync(
        &self,
        user_id: &str,
        node_ids: &[i64],
    ) -> HashMap<i64, MemoryBasics> {
        let states = self.memory_states.read().unwrap();
        let mut result = HashMap::with_capacity(node_ids.len());
        for &node_id in node_ids {
            if let Some(state) = states.get(&(user_id.to_string(), node_id)) {
                result.insert(
                    node_id,
                    MemoryBasics {
                        energy: state.energy as f32,
                        next_due_ts: state.due_at.timestamp_millis(),
                    },
                );
            }
        }
        result
    }

    /// Save memory state (synchronous).
    pub fn save_memory_state_sync(&self, state: &MemoryState) {
        let mut states = self.memory_states.write().unwrap();
        states.insert((state.user_id.clone(), state.node_id), state.clone());
    }

    /// Get single memory state (synchronous).
    pub fn get_memory_state_sync(&self, user_id: &str, node_id: i64) -> Option<MemoryState> {
        let states = self.memory_states.read().unwrap();
        states.get(&(user_id.to_string(), node_id)).cloned()
    }
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn get_memory_state(&self, user_id: &str, node_id: i64) -> Result<Option<MemoryState>> {
        let states = self.memory_states.read().unwrap();
        Ok(states.get(&(user_id.to_string(), node_id)).cloned())
    }

    async fn save_memory_state(&self, state: &MemoryState) -> Result<()> {
        let mut states = self.memory_states.write().unwrap();
        states.insert((state.user_id.clone(), state.node_id), state.clone());
        Ok(())
    }

    async fn get_due_states(
        &self,
        user_id: &str,
        due_before: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<MemoryState>> {
        let states = self.memory_states.read().unwrap();
        let mut due: Vec<_> = states
            .iter()
            .filter(|((uid, _), state)| uid == user_id && state.due_at <= due_before)
            .map(|(_, state)| state.clone())
            .collect();

        // Sort by due_at ascending
        due.sort_by_key(|s| s.due_at);
        due.truncate(limit as usize);
        Ok(due)
    }

    async fn update_energy(&self, user_id: &str, node_id: i64, new_energy: f64) -> Result<()> {
        let mut states = self.memory_states.write().unwrap();
        if let Some(state) = states.get_mut(&(user_id.to_string(), node_id)) {
            state.energy = new_energy.clamp(0.0, 1.0);
        }
        Ok(())
    }

    async fn log_propagation(&self, event: &PropagationEvent) -> Result<()> {
        let mut log = self.propagation_log.write().unwrap();
        log.push(event.clone());
        Ok(())
    }

    async fn get_session_state(&self) -> Result<Vec<i64>> {
        let session = self.session_state.read().unwrap();
        Ok(session.clone())
    }

    async fn save_session_state(&self, node_ids: &[i64]) -> Result<()> {
        let mut session = self.session_state.write().unwrap();
        *session = node_ids.to_vec();
        Ok(())
    }

    async fn clear_session_state(&self) -> Result<()> {
        let mut session = self.session_state.write().unwrap();
        session.clear();
        Ok(())
    }

    async fn get_stat(&self, key: &str) -> Result<Option<String>> {
        let stats = self.stats.read().unwrap();
        Ok(stats.get(key).cloned())
    }

    async fn set_stat(&self, key: &str, value: &str) -> Result<()> {
        let mut stats = self.stats.write().unwrap();
        stats.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let settings = self.settings.read().unwrap();
        Ok(settings.get(key).cloned())
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let mut settings = self.settings.write().unwrap();
        settings.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn save_review_atomic(
        &self,
        user_id: &str,
        state: &MemoryState,
        energy_updates: Vec<(i64, f64)>,
        propagation_event: Option<PropagationEvent>,
    ) -> Result<()> {
        // In-memory is naturally atomic within the locks
        {
            let mut states = self.memory_states.write().unwrap();
            states.insert((user_id.to_string(), state.node_id), state.clone());

            for (node_id, new_energy) in energy_updates {
                if let Some(s) = states.get_mut(&(user_id.to_string(), node_id)) {
                    s.energy = new_energy.clamp(0.0, 1.0);
                }
            }
        }

        if let Some(event) = propagation_event {
            let mut log = self.propagation_log.write().unwrap();
            log.push(event);
        }

        Ok(())
    }

    async fn save_memory_states_batch(&self, states: &[MemoryState]) -> Result<()> {
        let mut mem_states = self.memory_states.write().unwrap();
        for state in states {
            mem_states.insert((state.user_id.clone(), state.node_id), state.clone());
        }
        Ok(())
    }

    async fn get_parent_energies(
        &self,
        user_id: &str,
        parent_ids: &[i64],
    ) -> Result<HashMap<i64, f32>> {
        let states = self.memory_states.read().unwrap();
        let mut result = HashMap::new();
        for &node_id in parent_ids {
            if let Some(state) = states.get(&(user_id.to_string(), node_id)) {
                result.insert(node_id, state.energy as f32);
            }
        }
        Ok(result)
    }

    async fn get_memory_basics(
        &self,
        user_id: &str,
        node_ids: &[i64],
    ) -> Result<HashMap<i64, MemoryBasics>> {
        let states = self.memory_states.read().unwrap();
        let mut result = HashMap::new();
        for &node_id in node_ids {
            if let Some(state) = states.get(&(user_id.to_string(), node_id)) {
                result.insert(
                    node_id,
                    MemoryBasics {
                        energy: state.energy as f32,
                        next_due_ts: state.due_at.timestamp_millis(),
                    },
                );
            }
        }
        Ok(result)
    }

    async fn get_bandit_arms(
        &self,
        user_id: &str,
        goal_group: &str,
    ) -> Result<Vec<BanditArmState>> {
        let arms = self.bandit_arms.read().unwrap();
        let result: Vec<_> = arms
            .iter()
            .filter(|((uid, gg, _), _)| uid == user_id && gg == goal_group)
            .filter_map(|((_, _, profile_name), (successes, failures))| {
                // Use parse_str instead of FromStr
                ProfileName::parse_str(profile_name)
                    .map(|pn| BanditArmState::with_params(pn, *successes, *failures))
            })
            .collect();
        Ok(result)
    }

    async fn update_bandit_arm(
        &self,
        user_id: &str,
        goal_group: &str,
        profile_name: &str,
        successes: f32,
        failures: f32,
    ) -> Result<()> {
        let mut arms = self.bandit_arms.write().unwrap();
        arms.insert(
            (
                user_id.to_string(),
                goal_group.to_string(),
                profile_name.to_string(),
            ),
            (successes, failures),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_state_round_trip() {
        let repo = InMemoryUserRepository::new();

        let state = MemoryState::new_for_node("user1".to_string(), 123);
        repo.save_memory_state(&state).await.unwrap();

        let loaded = repo.get_memory_state("user1", 123).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().node_id, 123);
    }

    #[tokio::test]
    async fn test_get_due_states() {
        let repo = InMemoryUserRepository::new();

        // Create states with different due dates
        let now = Utc::now();
        let mut state1 = MemoryState::new_for_node("user1".to_string(), 1);
        state1.due_at = now - chrono::Duration::hours(1); // Overdue

        let mut state2 = MemoryState::new_for_node("user1".to_string(), 2);
        state2.due_at = now + chrono::Duration::hours(1); // Not due yet

        repo.save_memory_state(&state1).await.unwrap();
        repo.save_memory_state(&state2).await.unwrap();

        let due = repo.get_due_states("user1", now, 10).await.unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].node_id, 1);
    }

    #[tokio::test]
    async fn test_bandit_arms() {
        let repo = InMemoryUserRepository::new();

        repo.update_bandit_arm("user1", "hifdh", "Balanced", 5.0, 3.0)
            .await
            .unwrap();

        let arms = repo.get_bandit_arms("user1", "hifdh").await.unwrap();
        assert_eq!(arms.len(), 1);
        assert_eq!(arms[0].successes, 5.0);
        assert_eq!(arms[0].failures, 3.0);
    }

    #[tokio::test]
    async fn test_atomic_review_save() {
        let repo = InMemoryUserRepository::new();

        // Create initial state
        let state = MemoryState::new_for_node("user1".to_string(), 1);
        repo.save_memory_state(&state).await.unwrap();

        // Create another node to receive energy update
        let state2 = MemoryState::new_for_node("user1".to_string(), 2);
        repo.save_memory_state(&state2).await.unwrap();

        // Perform atomic save
        let mut updated_state = state.clone();
        updated_state.stability = 10.0;

        repo.save_review_atomic("user1", &updated_state, vec![(2, 0.8)], None)
            .await
            .unwrap();

        // Check both updates applied
        let s1 = repo.get_memory_state("user1", 1).await.unwrap().unwrap();
        assert_eq!(s1.stability, 10.0);

        let s2 = repo.get_memory_state("user1", 2).await.unwrap().unwrap();
        assert!((s2.energy - 0.8).abs() < 0.001);
    }
}
