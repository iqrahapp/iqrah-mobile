use crate::{ContentRepository, KnowledgeAxis, MemoryState, Node, NodeType, UserRepository};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

/// Scoring weights for session prioritization
#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub w_due: f64,   // Weight for days overdue
    pub w_need: f64,  // Weight for mastery gap (1.0 - energy)
    pub w_yield: f64, // Weight for importance/yield
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            w_due: 1.0,
            w_need: 2.0,
            w_yield: 1.5, // Default for foundational mode
        }
    }
}

/// Scored item for session generation
#[derive(Debug, Clone)]
pub struct ScoredItem {
    pub node: Node,
    pub memory_state: MemoryState,
    pub priority_score: f64,
    pub days_overdue: f64,
    pub mastery_gap: f64,
    /// Knowledge axis if this is a knowledge node (Phase 4)
    pub knowledge_axis: Option<KnowledgeAxis>,
}

/// Session service handles session generation and scoring
pub struct SessionService {
    content_repo: Arc<dyn ContentRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl SessionService {
    pub fn new(
        content_repo: Arc<dyn ContentRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            content_repo,
            user_repo,
        }
    }

    /// Get due items for a session with priority scoring
    ///
    /// # Arguments
    /// * `user_id` - The user ID
    /// * `limit` - Maximum number of items to return
    /// * `is_high_yield_mode` - Whether to emphasize high-influence nodes
    /// * `axis_filter` - Optional knowledge axis to filter by (Phase 4)
    pub async fn get_due_items(
        &self,
        user_id: &str,
        limit: u32,
        is_high_yield_mode: bool,
        axis_filter: Option<KnowledgeAxis>,
    ) -> Result<Vec<ScoredItem>> {
        let now = Utc::now();

        // Set weights based on mode
        let weights = if is_high_yield_mode {
            ScoreWeights {
                w_due: 1.0,
                w_need: 2.0,
                w_yield: 10.0, // High emphasis on influence
            }
        } else {
            ScoreWeights::default()
        };

        // Get all due memory states
        let due_states = self
            .user_repo
            .get_due_states(user_id, now, limit * 3) // Get extra for filtering
            .await?;

        // Score and sort
        let mut scored_items = Vec::new();

        for state in due_states {
            // Get node info
            let node = match self.content_repo.get_node(&state.node_id).await? {
                Some(n) => n,
                None => continue, // Skip if node doesn't exist
            };

            // Include word_instance, verse, and knowledge types
            let is_reviewable = matches!(
                node.node_type,
                NodeType::WordInstance | NodeType::Verse | NodeType::Knowledge
            );

            if !is_reviewable {
                continue;
            }

            // Phase 4: Filter by knowledge axis if specified
            let knowledge_axis = node.knowledge_node.as_ref().map(|kn| kn.axis);

            if let Some(filter_axis) = axis_filter {
                // If filtering by axis, only include matching knowledge nodes
                match knowledge_axis {
                    Some(axis) if axis == filter_axis => {
                        // Include this node
                    }
                    _ => {
                        // Skip nodes that don't match the axis filter
                        continue;
                    }
                }
            }

            // Calculate priority score
            let days_overdue = (now.timestamp_millis() - state.due_at.timestamp_millis()) as f64
                / (24.0 * 60.0 * 60.0 * 1000.0);
            let days_overdue = days_overdue.max(0.0);

            let mastery_gap = (1.0 - state.energy).max(0.0);

            // For now, use a simple importance based on node type
            // In future, could query from importance_scores table
            let importance = match node.node_type {
                NodeType::WordInstance => 0.5,
                NodeType::Verse => 0.3,
                NodeType::Knowledge => 0.6, // Knowledge nodes are important
                _ => 0.0,
            };

            let priority_score = weights.w_due * days_overdue
                + weights.w_need * mastery_gap
                + weights.w_yield * importance;

            scored_items.push(ScoredItem {
                node,
                memory_state: state,
                priority_score,
                days_overdue,
                mastery_gap,
                knowledge_axis,
            });
        }

        // Sort by priority (highest first)
        scored_items.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top N items
        scored_items.truncate(limit as usize);

        Ok(scored_items)
    }

    /// Get session state (for resuming)
    pub async fn get_session_state(&self) -> Result<Vec<String>> {
        self.user_repo.get_session_state().await
    }

    /// Save session state (for persistence)
    pub async fn save_session_state(&self, node_ids: &[String]) -> Result<()> {
        self.user_repo.save_session_state(node_ids).await
    }

    /// Clear session state
    pub async fn clear_session_state(&self) -> Result<()> {
        self.user_repo.clear_session_state().await
    }

    /// Get user statistics
    pub async fn get_stat(&self, key: &str) -> Result<Option<String>> {
        self.user_repo.get_stat(key).await
    }

    /// Set user statistics
    pub async fn set_stat(&self, key: &str, value: &str) -> Result<()> {
        self.user_repo.set_stat(key, value).await
    }

    /// Increment a stat (like reviews_today)
    pub async fn increment_stat(&self, key: &str) -> Result<u32> {
        let current = self
            .get_stat(key)
            .await?
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let new_value = current + 1;
        self.set_stat(key, &new_value.to_string()).await?;

        Ok(new_value)
    }
}
