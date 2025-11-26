/// Scheduler Service - Orchestrates data fetching for scheduler v2.0
///
/// This service is the ONLY component that depends on both ContentRepository and UserRepository.
/// It orchestrates calls between them to get the necessary data for the scheduler.
///
/// # Architecture Principle
///
/// - ContentRepository provides content-specific data (nodes, metadata, goals, prerequisites)
/// - UserRepository provides user-specific data (memory states, energies, due dates)
/// - SchedulerService bridges the two domains by:
///   1. Fetching candidate nodes from ContentRepository
///   2. Enriching them with user memory data from UserRepository
///   3. Filtering based on due status
///
use crate::scheduler_v2::CandidateNode;
use crate::{ContentRepository, UserRepository};
use std::sync::Arc;

pub struct SchedulerService {
    content_repo: Arc<dyn ContentRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl SchedulerService {
    /// Create a new SchedulerService
    pub fn new(
        content_repo: Arc<dyn ContentRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            content_repo,
            user_repo,
        }
    }

    /// Get candidate nodes for scheduling with user memory data
    ///
    /// This method orchestrates data fetching from both repositories:
    /// 1. Fetches all nodes for the goal from ContentRepository (with metadata)
    /// 2. Gets memory basics (energy + next_due_ts) from UserRepository
    /// 3. Populates the energy and next_due_ts fields in candidates
    /// 4. Filters to only include nodes that are:
    ///    - New (no memory state), OR
    ///    - Due (next_due_ts <= now_ts)
    ///
    /// # Arguments
    /// * `goal_id` - The goal to fetch candidates for
    /// * `user_id` - The user ID
    /// * `now_ts` - Current timestamp in milliseconds
    ///
    /// # Returns
    /// Vector of CandidateNode with all fields populated and filtered for due/new nodes
    pub async fn get_scheduler_candidates(
        &self,
        goal_id: &str,
        user_id: &str,
        now_ts: i64,
    ) -> anyhow::Result<Vec<CandidateNode>> {
        // Step 1: Get all nodes for the goal from ContentRepository
        // This returns candidates with metadata but energy=0.0 and next_due_ts=0
        let candidates = self
            .content_repo
            .get_scheduler_candidates(goal_id, user_id, now_ts)
            .await?;

        if candidates.is_empty() {
            return Ok(vec![]);
        }

        // Step 2: Get memory basics for all candidates from UserRepository
        let node_ids: Vec<String> = candidates.iter().map(|c| c.id.clone()).collect();
        let memory_basics = self.user_repo.get_memory_basics(user_id, &node_ids).await?;

        // Step 3: Enrich candidates with user memory data and filter for due/new nodes
        let enriched_candidates: Vec<CandidateNode> = candidates
            .into_iter()
            .filter_map(|mut candidate| {
                if let Some(basics) = memory_basics.get(&candidate.id) {
                    // Node has memory state - populate fields
                    candidate.energy = basics.energy;
                    candidate.next_due_ts = basics.next_due_ts;

                    // Include if due
                    if basics.next_due_ts <= now_ts {
                        Some(candidate)
                    } else {
                        None
                    }
                } else {
                    // Node has no memory state - it's new, include it
                    // energy and next_due_ts remain at default values (0.0 and 0)
                    Some(candidate)
                }
            })
            .collect();

        Ok(enriched_candidates)
    }
}
