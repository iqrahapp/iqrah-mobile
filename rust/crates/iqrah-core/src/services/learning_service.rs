use crate::{
    ContentRepository, MemoryState, PropagationDetail, PropagationEvent, ReviewGrade,
    UserRepository,
};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

/// Learning service handles review processing and FSRS scheduling
pub struct LearningService {
    content_repo: Arc<dyn ContentRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl LearningService {
    pub fn new(
        content_repo: Arc<dyn ContentRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            content_repo,
            user_repo,
        }
    }

    /// Process a single review and update memory state atomically
    /// All database changes are wrapped in a transaction - either all succeed or all rollback
    pub async fn process_review(
        &self,
        user_id: &str,
        node_id: i64,
        grade: ReviewGrade,
    ) -> Result<MemoryState> {
        // Task 3.2: Validate node exists in content.db before processing
        if !self.content_repo.node_exists(node_id).await? {
            return Err(anyhow::anyhow!(
                "Invalid node reference: node {} does not exist in content database",
                node_id
            ));
        }

        // 1. Get current memory state (read-only, outside transaction)
        let current_state = self.get_or_create_initial_state(user_id, node_id).await?;

        // 2. Calculate FSRS update (pure computation)
        let new_state = self.update_fsrs_state(current_state.clone(), grade)?;

        // 3. Calculate energy delta (pure computation)
        let energy_delta = calculate_energy_delta(grade, current_state.energy);
        let new_energy = (current_state.energy + energy_delta).clamp(0.0, 1.0);

        // 4. Create final state with updated energy
        let final_state = MemoryState {
            energy: new_energy,
            ..new_state
        };

        // 5. Prepare propagation data (read from content.db, outside transaction)
        let (energy_updates, propagation_event) = if energy_delta.abs() > 0.0001 {
            self.prepare_propagation(user_id, node_id, energy_delta)
                .await?
        } else {
            (vec![], None)
        };

        // ====================================================================
        // Task 3.1: ATOMIC TRANSACTION - All writes via save_review_atomic
        // ====================================================================
        self.user_repo
            .save_review_atomic(
                user_id,
                &final_state,
                energy_updates,
                propagation_event.as_ref(),
            )
            .await?;

        Ok(final_state)
    }

    /// Get memory state or prepare initial state for a new node
    async fn get_or_create_initial_state(
        &self,
        user_id: &str,
        node_id: i64,
    ) -> Result<MemoryState> {
        match self.user_repo.get_memory_state(user_id, node_id).await? {
            Some(state) => Ok(state),
            None => {
                // Return a new state - it will be saved in the transaction
                Ok(MemoryState::new_for_node(user_id.to_string(), node_id))
            }
        }
    }

    /// Update FSRS scheduling parameters
    fn update_fsrs_state(&self, current: MemoryState, grade: ReviewGrade) -> Result<MemoryState> {
        use fsrs::{MemoryState as FSRSMemory, FSRS};

        let fsrs = FSRS::new(Some(&[]))?;
        let now = Utc::now();
        let optimal_retention = 0.8f32;

        // Calculate elapsed days since last review
        let elapsed_days = ((now.timestamp_millis() - current.last_reviewed.timestamp_millis())
            as f64
            / (24.0 * 60.0 * 60.0 * 1000.0)) as u32;

        // Create FSRS memory state (cast to f32)
        let memory_state = FSRSMemory {
            stability: current.stability as f32,
            difficulty: current.difficulty as f32,
        };

        // Get next states (wrap in Some for new items)
        let next_states = fsrs.next_states(Some(memory_state), optimal_retention, elapsed_days)?;

        // Select the appropriate state based on grade
        let selected_state = match grade {
            ReviewGrade::Again => next_states.again,
            ReviewGrade::Hard => next_states.hard,
            ReviewGrade::Good => next_states.good,
            ReviewGrade::Easy => next_states.easy,
        };

        // Calculate due date from interval
        let due_at = now
            + chrono::Duration::try_days(selected_state.interval as i64)
                .unwrap_or(chrono::Duration::days(1));

        // Convert back to our MemoryState (cast f32 to f64)
        Ok(MemoryState {
            user_id: current.user_id,
            node_id: current.node_id,
            stability: selected_state.memory.stability as f64,
            difficulty: selected_state.memory.difficulty as f64,
            energy: current.energy, // Will be updated separately
            last_reviewed: now,
            due_at,
            review_count: current.review_count + 1,
        })
    }

    /// Prepare propagation data (reads only, to be applied in transaction)
    /// Returns: Vec of (target_node_id, new_energy) updates and optional propagation event
    async fn prepare_propagation(
        &self,
        user_id: &str,
        source_node_id: i64,
        delta: f64,
    ) -> Result<(Vec<(i64, f64)>, Option<PropagationEvent>)> {
        // Get edges from this node
        let edges = self.content_repo.get_edges_from(source_node_id).await?;

        let mut updates = Vec::new();
        let mut details = Vec::new();

        for edge in edges {
            // Calculate propagated delta based on edge distribution
            let propagated_delta = self.calculate_propagated_delta(&edge, delta);

            if propagated_delta.abs() < 0.001 {
                continue; // Skip negligible changes
            }

            // Get current state of target node
            if let Some(target_state) = self
                .user_repo
                .get_memory_state(user_id, edge.target_id)
                .await?
            {
                // Calculate new energy
                let new_energy = (target_state.energy + propagated_delta).clamp(0.0, 1.0);
                updates.push((edge.target_id, new_energy));

                details.push(PropagationDetail {
                    target_node_id: edge.target_id,
                    energy_change: propagated_delta,
                    reason: format!("Propagated from {}", source_node_id),
                });
            }
        }

        if details.is_empty() {
            return Ok((updates, None));
        }

        let event = PropagationEvent {
            source_node_id,
            event_timestamp: Utc::now(),
            details,
        };

        Ok((updates, Some(event)))
    }

    /// Calculate how much energy propagates through an edge
    fn calculate_propagated_delta(&self, edge: &crate::Edge, source_delta: f64) -> f64 {
        use crate::DistributionType;

        // Simple propagation: reduce by 50% for const, more complex for others
        match edge.distribution_type {
            DistributionType::Const => source_delta * 0.5,
            DistributionType::Normal => {
                // Use param1 as attenuation factor (0.0-1.0)
                source_delta * edge.param1.clamp(0.0, 1.0)
            }
            DistributionType::Beta => {
                // More complex: use beta distribution parameters
                // For now, simple attenuation
                source_delta * 0.3
            }
        }
    }
}

/// Calculate energy delta based on review grade
fn calculate_energy_delta(grade: ReviewGrade, current_energy: f64) -> f64 {
    let base_delta = match grade {
        ReviewGrade::Again => -0.1,
        ReviewGrade::Hard => 0.02,
        ReviewGrade::Good => 0.05,
        ReviewGrade::Easy => 0.08,
    };

    // Diminishing returns as energy approaches 1.0
    base_delta * (1.0 - current_energy)
}
