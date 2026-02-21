use crate::domain::{KnowledgeAxis, KnowledgeNode, MemoryState, NodeType};
use crate::{ContentRepository, Node, UserRepository};
use anyhow::Result;
use chrono::{DateTime, Utc};
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
    /// * `now` - Reference instant for computing days-overdue (injectable for deterministic tests)
    /// * `limit` - Maximum number of items to return
    /// * `is_high_yield_mode` - Whether to emphasize high-influence nodes
    /// * `axis_filter` - Optional knowledge axis to filter by (Phase 4)
    #[instrument(skip(self), fields(user_id, limit, is_high_yield_mode))]
    pub async fn get_due_items(
        &self,
        user_id: &str,
        now: DateTime<Utc>,
        limit: u32,
        is_high_yield_mode: bool,
        axis_filter: Option<KnowledgeAxis>,
    ) -> Result<Vec<ScoredItem>> {
        debug!("Fetching due items");

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

        // Deduplicate: keep the most-overdue state per node (earliest due_at wins).
        // This is a defensive guard — the repository should not return duplicates,
        // but if it does, we select the state that would score highest rather than
        // accepting whichever one arrives first.
        let mut best_by_node: std::collections::HashMap<i64, MemoryState> =
            std::collections::HashMap::new();
        for state in due_states {
            best_by_node
                .entry(state.node_id)
                .and_modify(|existing| {
                    // Prefer the state with the earlier due_at (longer overdue)
                    if state.due_at < existing.due_at {
                        *existing = state.clone();
                    }
                })
                .or_insert(state);
        }

        // Score and sort
        let mut scored_items = Vec::new();

        for state in best_by_node.into_values() {
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
            let knowledge_axis = KnowledgeNode::parse(&node.ukey).map(|kn| kn.axis).or(
                if matches!(node.node_type, NodeType::Verse) {
                    Some(KnowledgeAxis::Memorization)
                } else {
                    None
                },
            );

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
    use chrono::{DateTime, Duration, Utc};
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
        let result = service.get_due_items("user1", now, 10, false, None).await;

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
        let result = service.get_due_items("user1", now, 10, false, None).await;

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
    async fn test_defaults_axis_for_verse_nodes() {
        // Arrange
        let content_repo = Arc::new(create_content_mock());
        let now = Utc::now();

        let states = vec![MemoryState {
            user_id: "user1".to_string(),
            node_id: 3, // Verse
            stability: 10.0,
            difficulty: 5.0,
            energy: 0.3,
            last_reviewed: now,
            due_at: now,
            review_count: 2,
        }];

        let user_repo = Arc::new(create_user_mock_with_due_states(states));
        let service = SessionService::new(content_repo, user_repo);

        // Act
        let result = service.get_due_items("user1", now, 10, false, None).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].knowledge_axis, Some(KnowledgeAxis::Memorization));
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
        let result = service.get_due_items("user1", now, 10, false, None).await;

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
        let result = service.get_due_items("user1", now, 1, false, None).await;

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
        let result = service.get_due_items("user1", now, 10, false, None).await;

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
        let result = service.get_due_items("user1", now, 10, false, None).await;

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

    // ========================================================================
    // C-001: Deterministic Golden Session Scenarios
    //
    // These tests use fixed input data and assert exact outputs.
    // They serve as regression anchors: any behavioral change will break them.
    // Scenarios: cold-start, due-review, chunk/axis-filtered mode.
    // ========================================================================
    mod golden_scenarios {
        use super::*;

        /// Golden scenario 1: Cold-start — brand-new user with zero memory states.
        ///
        /// Documents the known cold-start gap: the current runtime scheduler
        /// is due-queue-only, so a user with no states gets an empty session.
        /// This test locks in that behavior as the baseline for C-003 to fix.
        #[tokio::test]
        async fn test_golden_cold_start_empty_session() {
            let content_repo = Arc::new(create_content_mock());
            let user_repo = Arc::new(create_user_mock_with_due_states(vec![]));
            let service = SessionService::new(content_repo, user_repo);
            let now = Utc::now();

            let result = service
                .get_due_items("new_user", now, 20, false, None)
                .await;

            assert!(result.is_ok());
            let items = result.unwrap();
            // GOLDEN: new user with no memory states → empty session.
            // This gap is tracked as C-003 (cold-start empty-session fix).
            assert_eq!(
                items.len(),
                0,
                "Cold-start: new user with zero memory states must get an empty session (C-003 gap)"
            );
        }

        /// Golden scenario 2: Due-review — deterministic priority ordering with fixed data.
        ///
        /// Three WordInstance nodes with known energies and overdue durations are
        /// scored with default weights (w_due=1.0, w_need=2.0, w_yield=1.5).
        /// Expected priority scores (importance=0.5 for WordInstance):
        ///   Node 10: 1.0×7 + 2.0×0.9 + 1.5×0.5 = 9.55  (highest)
        ///   Node 11: 1.0×3 + 2.0×0.5 + 1.5×0.5 = 4.75  (medium)
        ///   Node 12: 1.0×1 + 2.0×0.2 + 1.5×0.5 = 2.15  (lowest)
        #[tokio::test]
        async fn test_golden_due_review_deterministic_priority() {
            // Fixed reference point — makes priority scores fully deterministic across
            // any CI environment. Using a pinned Unix timestamp avoids wall-clock drift.
            let fixed_now = DateTime::from_timestamp(1_740_000_000, 0).unwrap();
            let seven_days_ago = fixed_now - Duration::try_days(7).unwrap();
            let three_days_ago = fixed_now - Duration::try_days(3).unwrap();
            let one_day_ago = fixed_now - Duration::try_days(1).unwrap();

            let states = vec![
                MemoryState {
                    user_id: "user_golden".to_string(),
                    node_id: 10,
                    stability: 5.0,
                    difficulty: 6.0,
                    energy: 0.1, // mastery_gap = 0.9
                    last_reviewed: seven_days_ago,
                    due_at: seven_days_ago, // overdue by exactly 7 days at fixed_now
                    review_count: 2,
                },
                MemoryState {
                    user_id: "user_golden".to_string(),
                    node_id: 11,
                    stability: 8.0,
                    difficulty: 5.0,
                    energy: 0.5, // mastery_gap = 0.5
                    last_reviewed: three_days_ago,
                    due_at: three_days_ago, // overdue by exactly 3 days at fixed_now
                    review_count: 5,
                },
                MemoryState {
                    user_id: "user_golden".to_string(),
                    node_id: 12,
                    stability: 12.0,
                    difficulty: 4.0,
                    energy: 0.8, // mastery_gap = 0.2
                    last_reviewed: one_day_ago,
                    due_at: one_day_ago, // overdue by exactly 1 day at fixed_now
                    review_count: 10,
                },
            ];

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|node_id| {
                if matches!(node_id, 10..=12) {
                    Ok(Some(Node {
                        id: node_id,
                        ukey: format!("WORD_INSTANCE:1:{}:1", node_id),
                        node_type: NodeType::WordInstance,
                    }))
                } else {
                    Ok(None)
                }
            });

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let result = service
                .get_due_items("user_golden", fixed_now, 10, false, None)
                .await;
            assert!(result.is_ok());
            let items = result.unwrap();

            // GOLDEN: all three items returned
            assert_eq!(items.len(), 3, "All three due items must be returned");

            // GOLDEN: exact ordering by priority score
            assert_eq!(
                items[0].node.id, 10,
                "Node 10 must rank first (7 days overdue, energy=0.1)"
            );
            assert_eq!(
                items[1].node.id, 11,
                "Node 11 must rank second (3 days overdue, energy=0.5)"
            );
            assert_eq!(
                items[2].node.id, 12,
                "Node 12 must rank last (1 day overdue, energy=0.8)"
            );

            // GOLDEN: priority scores are strictly descending
            assert!(
                items[0].priority_score > items[1].priority_score,
                "Score[0]={:.3} must exceed score[1]={:.3}",
                items[0].priority_score,
                items[1].priority_score
            );
            assert!(
                items[1].priority_score > items[2].priority_score,
                "Score[1]={:.3} must exceed score[2]={:.3}",
                items[1].priority_score,
                items[2].priority_score
            );

            // GOLDEN: approximate score values (regression guard)
            assert!(
                (items[0].priority_score - 9.55).abs() < 0.2,
                "Node 10 priority ≈ 9.55, got {:.3}",
                items[0].priority_score
            );
            assert!(
                (items[1].priority_score - 4.75).abs() < 0.2,
                "Node 11 priority ≈ 4.75, got {:.3}",
                items[1].priority_score
            );
            assert!(
                (items[2].priority_score - 2.15).abs() < 0.2,
                "Node 12 priority ≈ 2.15, got {:.3}",
                items[2].priority_score
            );
        }

        /// Golden scenario 3: Chunk/axis-filtered mode — Memorization axis only.
        ///
        /// Simulates a user selecting a contiguous Quran chunk for memorization practice.
        /// Only nodes whose axis resolves to Memorization are returned.
        ///
        /// Input nodes:
        ///   20: Verse         → auto-assigned Memorization axis  → INCLUDED
        ///   21: Knowledge     → ukey ends in ":memorization"     → INCLUDED
        ///   22: Knowledge     → ukey ends in ":translation"      → EXCLUDED
        ///   23: WordInstance  → no axis suffix                   → EXCLUDED
        #[tokio::test]
        async fn test_golden_chunk_mode_memorization_axis_filter() {
            let now = Utc::now();

            let states: Vec<MemoryState> = vec![20_i64, 21, 22, 23]
                .into_iter()
                .map(|id| MemoryState {
                    user_id: "chunk_user".to_string(),
                    node_id: id,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                })
                .collect();

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|id| {
                let (ukey, node_type) = match id {
                    20 => ("VERSE:1:1".to_string(), NodeType::Verse),
                    21 => (
                        "WORD_INSTANCE:1:1:1:memorization".to_string(),
                        NodeType::Knowledge,
                    ),
                    22 => ("VERSE:1:2:translation".to_string(), NodeType::Knowledge),
                    23 => ("WORD_INSTANCE:1:1:1".to_string(), NodeType::WordInstance),
                    _ => return Ok(None),
                };
                Ok(Some(Node {
                    id,
                    ukey,
                    node_type,
                }))
            });

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let result = service
                .get_due_items(
                    "chunk_user",
                    now,
                    10,
                    false,
                    Some(KnowledgeAxis::Memorization),
                )
                .await;
            assert!(result.is_ok());
            let items = result.unwrap();

            let returned_ids: Vec<i64> = items.iter().map(|i| i.node.id).collect();

            // GOLDEN: exact inclusion/exclusion
            assert!(
                returned_ids.contains(&20),
                "Node 20 (Verse → auto-Memorization) must be included"
            );
            assert!(
                returned_ids.contains(&21),
                "Node 21 (Knowledge:memorization) must be included"
            );
            assert!(
                !returned_ids.contains(&22),
                "Node 22 (Knowledge:translation) must be excluded in Memorization chunk mode"
            );
            assert!(
                !returned_ids.contains(&23),
                "Node 23 (WordInstance, no axis) must be excluded in Memorization chunk mode"
            );
            assert_eq!(
                items.len(),
                2,
                "Exactly 2 nodes match the Memorization axis filter"
            );

            // GOLDEN: every returned item carries the Memorization axis
            for item in &items {
                assert_eq!(
                    item.knowledge_axis,
                    Some(KnowledgeAxis::Memorization),
                    "Every item in chunk mode must carry the requested axis"
                );
            }
        }
    }

    // ========================================================================
    // C-002: Scheduler Invariants Test Suite
    //
    // These tests verify properties that must hold for ALL inputs:
    //   1. No duplicate node IDs in session output
    //   2. Session size never exceeds the requested limit
    //   3. Verse nodes always receive the Memorization axis
    //   4. Chapter nodes are never included in session output
    //   5. Axis filter excludes all non-matching nodes without exception
    // ========================================================================
    mod scheduler_invariants {
        use super::*;

        /// Invariant 1: No duplicate node IDs in session output.
        ///
        /// Even if the repository defensively returns duplicate states for the
        /// same node, the service must deduplicate before returning.
        #[tokio::test]
        async fn test_invariant_no_duplicate_node_ids() {
            let content_repo = Arc::new(create_content_mock());
            let now = Utc::now();

            // Simulate a repository returning the same node_id twice
            let states = vec![
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 1,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 1, // same node — duplicate
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 2,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
            ];

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(content_repo, user_repo);

            let result = service.get_due_items("user1", now, 10, false, None).await;
            assert!(result.is_ok());
            let items = result.unwrap();

            let ids: Vec<i64> = items.iter().map(|i| i.node.id).collect();
            let unique_count = ids
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<i64>>()
                .len();

            // INVARIANT: no duplicate node IDs
            assert_eq!(
                ids.len(),
                unique_count,
                "Session must not contain duplicate node IDs; got {:?}",
                ids
            );
            // Expect exactly 2 unique nodes (1 and 2), not 3
            assert_eq!(
                items.len(),
                2,
                "Duplicate state for node 1 must be collapsed to one item"
            );
        }

        /// Invariant 2: Session size never exceeds the requested limit.
        ///
        /// Tests multiple limit values against a pool of 20 candidate items.
        #[tokio::test]
        async fn test_invariant_limit_never_exceeded() {
            let content_repo = Arc::new(create_content_mock());
            let now = Utc::now();

            // 20 states with node IDs 1-20; node 4 is Chapter (filtered out → 19 valid)
            let states: Vec<MemoryState> = (1_i64..=20)
                .map(|i| MemoryState {
                    user_id: "user1".to_string(),
                    node_id: i,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                })
                .collect();

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(content_repo, user_repo);

            for limit in [1u32, 3, 5, 10, 15, 20] {
                let result = service
                    .get_due_items("user1", now, limit, false, None)
                    .await;
                assert!(result.is_ok(), "get_due_items failed for limit={}", limit);
                let items = result.unwrap();
                assert!(
                    items.len() <= limit as usize,
                    "INVARIANT VIOLATED: limit={} but got {} items",
                    limit,
                    items.len()
                );
            }
        }

        /// Invariant 3: Every Verse node always receives the Memorization axis.
        ///
        /// Verse nodes do not carry an explicit axis suffix in their ukey, so
        /// the service must assign Memorization as the default axis.
        #[tokio::test]
        async fn test_invariant_verse_always_memorization_axis() {
            let now = Utc::now();

            let states: Vec<MemoryState> = (30_i64..=34)
                .map(|i| MemoryState {
                    user_id: "user1".to_string(),
                    node_id: i,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                })
                .collect();

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|id| {
                Ok(Some(Node {
                    id,
                    ukey: format!("VERSE:1:{}", id),
                    node_type: NodeType::Verse,
                }))
            });

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let result = service.get_due_items("user1", now, 10, false, None).await;
            assert!(result.is_ok());
            let items = result.unwrap();

            assert!(!items.is_empty(), "Verse items should be returned");
            assert_eq!(items.len(), 5, "All 5 Verse nodes should be included");

            // INVARIANT: every Verse carries the Memorization axis
            for item in &items {
                assert_eq!(
                    item.knowledge_axis,
                    Some(KnowledgeAxis::Memorization),
                    "Verse node {} must always have Memorization axis, got {:?}",
                    item.node.id,
                    item.knowledge_axis
                );
            }
        }

        /// Invariant 4: Chapter nodes are never included in session output.
        ///
        /// Chapter is not a reviewable node type and must be filtered out
        /// regardless of its position in the due-states list.
        #[tokio::test]
        async fn test_invariant_chapter_nodes_never_in_session() {
            // create_content_mock: node 4 → Chapter, nodes 1 and 3 → reviewable
            let content_repo = Arc::new(create_content_mock());
            let now = Utc::now();

            let states = vec![
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 1,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 4, // Chapter — must be excluded
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
                MemoryState {
                    user_id: "user1".to_string(),
                    node_id: 3, // Verse — must be included
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                },
            ];

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(content_repo, user_repo);

            let result = service.get_due_items("user1", now, 10, false, None).await;
            assert!(result.is_ok());
            let items = result.unwrap();

            // INVARIANT: no Chapter node in session output
            for item in &items {
                assert_ne!(
                    item.node.node_type,
                    NodeType::Chapter,
                    "Chapter node {} must never appear in session output",
                    item.node.id
                );
            }
            // Only nodes 1 and 3 should pass through
            assert_eq!(
                items.len(),
                2,
                "Exactly 2 reviewable nodes (excluding Chapter) expected"
            );
        }

        /// Invariant 5: Axis filter excludes ALL non-matching nodes without exception.
        ///
        /// When an axis filter is active, only nodes whose resolved axis matches
        /// the filter are included. No node slips through on any other basis.
        #[tokio::test]
        async fn test_invariant_axis_filter_excludes_all_non_matching() {
            let now = Utc::now();

            // Four nodes covering all axis-resolution code paths:
            //   40: Verse           → Memorization (auto)   — matches filter
            //   41: Knowledge       → Translation (parsed)  — excluded
            //   42: WordInstance    → no axis               — excluded
            //   43: Knowledge       → Tafsir (parsed)       — excluded
            let states: Vec<MemoryState> = vec![40_i64, 41, 42, 43]
                .into_iter()
                .map(|id| MemoryState {
                    user_id: "user1".to_string(),
                    node_id: id,
                    stability: 5.0,
                    difficulty: 5.0,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                })
                .collect();

            let mut content_mock = MockContentRepository::new();
            content_mock.expect_get_node().returning(|id| {
                let (ukey, node_type) = match id {
                    40 => ("VERSE:1:1".to_string(), NodeType::Verse),
                    41 => ("VERSE:1:2:translation".to_string(), NodeType::Knowledge),
                    42 => ("WORD_INSTANCE:1:1:1".to_string(), NodeType::WordInstance),
                    43 => ("VERSE:2:1:tafsir".to_string(), NodeType::Knowledge),
                    _ => return Ok(None),
                };
                Ok(Some(Node {
                    id,
                    ukey,
                    node_type,
                }))
            });

            let user_repo = Arc::new(create_user_mock_with_due_states(states));
            let service = SessionService::new(Arc::new(content_mock), user_repo);

            let result = service
                .get_due_items("user1", now, 10, false, Some(KnowledgeAxis::Memorization))
                .await;
            assert!(result.is_ok());
            let items = result.unwrap();

            // INVARIANT: every returned item matches the axis filter exactly
            for item in &items {
                assert_eq!(
                    item.knowledge_axis,
                    Some(KnowledgeAxis::Memorization),
                    "Axis filter must exclude all non-matching items; node {} has axis {:?}",
                    item.node.id,
                    item.knowledge_axis
                );
            }

            // GOLDEN sub-check: only node 40 (Verse → Memorization) should pass
            assert_eq!(
                items.len(),
                1,
                "Only 1 node matches the Memorization axis filter"
            );
            assert_eq!(
                items[0].node.id, 40,
                "Only the Verse node (id=40) should pass the filter"
            );
        }
    }
}
