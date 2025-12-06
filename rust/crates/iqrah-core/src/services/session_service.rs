use crate::domain::{KnowledgeAxis, KnowledgeNode, MemoryState, NodeType};
use crate::{ContentRepository, Node, UserRepository};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, instrument};

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
    #[instrument(skip(self), fields(user_id, limit, is_high_yield_mode))]
    pub async fn get_due_items(
        &self,
        user_id: &str,
        limit: u32,
        is_high_yield_mode: bool,
        axis_filter: Option<KnowledgeAxis>,
    ) -> Result<Vec<ScoredItem>> {
        debug!("Fetching due items");
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
            let node = match self.content_repo.get_node(state.node_id).await? {
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
            let knowledge_axis = KnowledgeNode::parse(&node.ukey).map(|kn| kn.axis);

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
    pub async fn get_session_state(&self) -> Result<Vec<i64>> {
        self.user_repo.get_session_state().await
    }

    /// Save session state (for persistence)
    pub async fn save_session_state(&self, node_ids: &[i64]) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{MockContentRepository, MockUserRepository};
    use crate::{MemoryState, Node, NodeType};
    use chrono::{Duration, Utc};
    use std::sync::Arc;

    /// Helper to create a mock ContentRepository with basic node setup
    fn create_content_mock() -> MockContentRepository {
        let mut mock = MockContentRepository::new();

        // Setup get_node for nodes 1-4 of different types
        mock.expect_get_node().returning(|node_id| {
            let node_type = match node_id {
                1 => NodeType::WordInstance,
                2 => NodeType::WordInstance,
                3 => NodeType::Verse,
                4 => NodeType::Chapter,
                _ => NodeType::WordInstance,
            };
            Ok(Some(Node {
                id: node_id,
                ukey: format!("node_{}", node_id),
                node_type,
            }))
        });

        mock
    }

    /// Helper to create a mock UserRepository with configurable due states
    fn create_user_mock_with_due_states(states: Vec<MemoryState>) -> MockUserRepository {
        let mut mock = MockUserRepository::new();

        // Clone states for get_due_states
        let states_clone = states.clone();
        mock.expect_get_due_states()
            .returning(move |_, _, _| Ok(states_clone.clone()));

        // Session state management
        let session_state = std::sync::Arc::new(std::sync::Mutex::new(Vec::<i64>::new()));
        let session_state_save = session_state.clone();
        let session_state_get = session_state.clone();
        let session_state_clear = session_state.clone();

        mock.expect_save_session_state().returning(move |node_ids| {
            let mut state = session_state_save.lock().unwrap();
            *state = node_ids.to_vec();
            Ok(())
        });

        mock.expect_get_session_state().returning(move || {
            let state = session_state_get.lock().unwrap();
            Ok(state.clone())
        });

        mock.expect_clear_session_state().returning(move || {
            let mut state = session_state_clear.lock().unwrap();
            state.clear();
            Ok(())
        });

        // Stat management
        let stats = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::<
            String,
            String,
        >::new()));
        let stats_set = stats.clone();
        let stats_get = stats.clone();

        mock.expect_set_stat().returning(move |key, value| {
            let mut s = stats_set.lock().unwrap();
            s.insert(key.to_string(), value.to_string());
            Ok(())
        });

        mock.expect_get_stat().returning(move |key| {
            let s = stats_get.lock().unwrap();
            Ok(s.get(key).cloned())
        });

        mock
    }

    #[tokio::test]
    async fn test_get_due_items_returns_items() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();

        let states = vec![MemoryState {
            user_id: "user1".to_string(),
            node_id: 1,
            stability: 10.0,
            difficulty: 5.0,
            energy: 0.3,
            last_reviewed: now,
            due_at: now,
            review_count: 3,
        }];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].node.id, 1);
    }

    #[tokio::test]
    async fn test_filters_by_node_type() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();

        // Create states with different node types (1=WordInstance, 3=Verse, 4=Chapter)
        let states = vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 1, // WordInstance
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 3,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 3, // Verse
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 2,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 4, // Chapter (should be filtered out)
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 1,
            },
        ];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2, "Should filter out Chapter type");

        for item in &items {
            assert!(
                matches!(
                    item.node.node_type,
                    NodeType::WordInstance | NodeType::Verse
                ),
                "Only WordInstance and Verse should be included"
            );
        }
    }

    #[tokio::test]
    async fn test_sorts_by_priority_descending() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();
        let very_overdue = now - Duration::try_days(10).unwrap();
        let slightly_overdue = now - Duration::try_days(1).unwrap();

        // Node 2 should have higher priority (low energy + very overdue)
        let states = vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 1,
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.9, // High energy, low need
                last_reviewed: slightly_overdue,
                due_at: slightly_overdue,
                review_count: 3,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 2,
                stability: 5.0,
                difficulty: 6.0,
                energy: 0.1, // Low energy, high need
                last_reviewed: very_overdue,
                due_at: very_overdue,
                review_count: 1,
            },
        ];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].node.id, 2,
            "Node 2 should be first (higher priority)"
        );
        assert!(items[0].priority_score > items[1].priority_score);
    }

    #[tokio::test]
    async fn test_respects_limit() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();

        let states = vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 1,
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 3,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: 2,
                stability: 5.0,
                difficulty: 6.0,
                energy: 0.6,
                last_reviewed: now,
                due_at: now,
                review_count: 1,
            },
        ];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", 1, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1, "Should respect limit parameter");
    }

    #[tokio::test]
    async fn test_session_state_management() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let user_repo = Arc::new(create_user_mock_with_due_states(vec![]));
        let service = SessionService::new(content_repo, user_repo);

        let node_ids = vec![1, 2];

        // Act - Save session state
        let save_result = service.save_session_state(&node_ids).await;
        assert!(save_result.is_ok());

        // Act - Get session state
        let get_result = service.get_session_state().await;
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), node_ids);

        // Act - Clear session state
        let clear_result = service.clear_session_state().await;
        assert!(clear_result.is_ok());

        let empty_result = service.get_session_state().await;
        assert!(empty_result.is_ok());
        assert_eq!(empty_result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_stat_management() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let user_repo = Arc::new(create_user_mock_with_due_states(vec![]));
        let service = SessionService::new(content_repo, user_repo);

        // Act - Set stat
        let set_result = service.set_stat("reviews_today", "5").await;
        assert!(set_result.is_ok());

        // Act - Get stat
        let get_result = service.get_stat("reviews_today").await;
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), Some("5".to_string()));

        // Act - Get non-existent stat
        let none_result = service.get_stat("nonexistent").await;
        assert!(none_result.is_ok());
        assert_eq!(none_result.unwrap(), None);
    }

    #[tokio::test]
    async fn test_increment_stat() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let user_repo = Arc::new(create_user_mock_with_due_states(vec![]));
        let service = SessionService::new(content_repo, user_repo);

        // Act - Increment from 0
        let first_result = service.increment_stat("reviews_today").await;
        assert!(first_result.is_ok());
        assert_eq!(first_result.unwrap(), 1);

        // Act - Increment again
        let second_result = service.increment_stat("reviews_today").await;
        assert!(second_result.is_ok());
        assert_eq!(second_result.unwrap(), 2);

        // Verify final value
        let get_result = service.get_stat("reviews_today").await;
        assert_eq!(get_result.unwrap(), Some("2".to_string()));
    }

    #[tokio::test]
    async fn test_calculates_mastery_gap_correctly() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();

        let states = vec![MemoryState {
            user_id: "user1".to_string(),
            node_id: 1,
            stability: 10.0,
            difficulty: 5.0,
            energy: 0.2, // mastery_gap should be 0.8
            last_reviewed: now,
            due_at: now,
            review_count: 3,
        }];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);

        let mastery_gap = items[0].mastery_gap;
        assert!(
            (mastery_gap - 0.8).abs() < 0.001,
            "Mastery gap should be 1.0 - energy"
        );
    }

    #[tokio::test]
    async fn test_calculates_days_overdue_correctly() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();
        let three_days_ago = now - Duration::try_days(3).unwrap();

        let states = vec![MemoryState {
            user_id: "user1".to_string(),
            node_id: 1,
            stability: 10.0,
            difficulty: 5.0,
            energy: 0.5,
            last_reviewed: three_days_ago,
            due_at: three_days_ago,
            review_count: 3,
        }];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);

        let days_overdue = items[0].days_overdue;
        assert!(
            (2.9..=3.1).contains(&days_overdue),
            "Days overdue should be approximately 3, got {}",
            days_overdue
        );
    }
}
