use std::sync::Arc;
use chrono::Utc;
use crate::{
    MemoryState, ReviewGrade, PropagationEvent, PropagationDetail,
    ContentRepository, UserRepository,
};
use anyhow::Result;

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

    /// Process a single review and update memory state
    pub async fn process_review(
        &self,
        user_id: &str,
        node_id: &str,
        grade: ReviewGrade,
    ) -> Result<MemoryState> {
        // 1. Get or create current memory state
        let current_state = self.get_or_create_state(user_id, node_id).await?;

        // 2. Calculate FSRS update
        let new_state = self.update_fsrs_state(current_state.clone(), grade)?;

        // 3. Calculate energy delta
        let energy_delta = calculate_energy_delta(grade, current_state.energy);
        let new_energy = (current_state.energy + energy_delta).clamp(0.0, 1.0);

        // 4. Create final state with updated energy
        let final_state = MemoryState {
            energy: new_energy,
            ..new_state
        };

        // 5. Save the updated state
        self.user_repo.save_memory_state(&final_state).await?;

        // 6. Propagate energy if significant change
        if energy_delta.abs() > 0.0001 {
            self.propagate_energy(user_id, node_id, energy_delta).await?;
        }

        Ok(final_state)
    }

    /// Get memory state or create a new one
    async fn get_or_create_state(&self, user_id: &str, node_id: &str) -> Result<MemoryState> {
        match self.user_repo.get_memory_state(user_id, node_id).await? {
            Some(state) => Ok(state),
            None => {
                // Lazy creation - only create state on first review
                let state = MemoryState::new_for_node(user_id.to_string(), node_id.to_string());
                self.user_repo.save_memory_state(&state).await?;
                Ok(state)
            }
        }
    }

    /// Update FSRS scheduling parameters
    fn update_fsrs_state(&self, current: MemoryState, grade: ReviewGrade) -> Result<MemoryState> {
        use fsrs::{FSRS, MemoryState as FSRSMemory};

        let fsrs = FSRS::new(Some(&[]))?;
        let now = Utc::now();
        let optimal_retention = 0.8f32;

        // Calculate elapsed days since last review
        let elapsed_days = ((now.timestamp_millis() - current.last_reviewed.timestamp_millis()) as f64
            / (24.0 * 60.0 * 60.0 * 1000.0))  as u32;

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
        let due_at = now + chrono::Duration::try_days(selected_state.interval as i64)
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

    /// Propagate energy changes through the knowledge graph
    async fn propagate_energy(
        &self,
        user_id: &str,
        source_node_id: &str,
        delta: f64,
    ) -> Result<()> {
        // Get edges from this node
        let edges = self.content_repo.get_edges_from(source_node_id).await?;

        if edges.is_empty() {
            return Ok(());
        }

        let mut details = Vec::new();

        for edge in edges {
            // Calculate propagated delta based on edge distribution
            let propagated_delta = self.calculate_propagated_delta(&edge, delta);

            if propagated_delta.abs() < 0.001 {
                continue; // Skip negligible changes
            }

            // Get current state of target node
            if let Some(target_state) = self.user_repo
                .get_memory_state(user_id, &edge.target_id)
                .await?
            {
                // Update target energy
                let new_energy = (target_state.energy + propagated_delta).clamp(0.0, 1.0);
                self.user_repo
                    .update_energy(user_id, &edge.target_id, new_energy)
                    .await?;

                details.push(PropagationDetail {
                    target_node_id: edge.target_id.clone(),
                    energy_change: propagated_delta,
                    reason: format!("Propagated from {}", source_node_id),
                });
            }
        }

        // Log propagation event
        if !details.is_empty() {
            let event = PropagationEvent {
                source_node_id: source_node_id.to_string(),
                event_timestamp: Utc::now(),
                details,
            };

            self.user_repo.log_propagation(&event).await?;
        }

        Ok(())
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
