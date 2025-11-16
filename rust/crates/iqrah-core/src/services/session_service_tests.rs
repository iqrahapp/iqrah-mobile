#[cfg(test)]
mod tests {
    use super::super::SessionService;
    use crate::{
        ContentRepository, UserRepository,
        Node, NodeType, MemoryState,
    };
    use std::sync::Arc;
    use std::collections::HashMap;
    use async_trait::async_trait;
    use chrono::{Utc, Duration};

    // Mock ContentRepository
    struct MockContentRepo {
        nodes: HashMap<String, Node>,
    }

    impl MockContentRepo {
        fn new() -> Self {
            let mut nodes = HashMap::new();

            // Create test nodes of different types
            nodes.insert(
                "word_1".to_string(),
                Node {
                    id: "word_1".to_string(),
                    node_type: NodeType::WordInstance,
                },
            );

            nodes.insert(
                "word_2".to_string(),
                Node {
                    id: "word_2".to_string(),
                    node_type: NodeType::WordInstance,
                },
            );

            nodes.insert(
                "verse_1".to_string(),
                Node {
                    id: "verse_1".to_string(),
                    node_type: NodeType::Verse,
                },
            );

            nodes.insert(
                "surah_1".to_string(),
                Node {
                    id: "surah_1".to_string(),
                    node_type: NodeType::Chapter,
                },
            );

            Self { nodes }
        }
    }

    #[async_trait]
    impl ContentRepository for MockContentRepo {
        async fn get_node(&self, node_id: &str) -> anyhow::Result<Option<Node>> {
            Ok(self.nodes.get(node_id).cloned())
        }

        async fn get_edges_from(&self, _source_id: &str) -> anyhow::Result<Vec<crate::Edge>> {
            Ok(vec![])
        }

        async fn get_quran_text(&self, _node_id: &str) -> anyhow::Result<Option<String>> {
            Ok(Some("Test Arabic".to_string()))
        }

        async fn get_translation(&self, _node_id: &str, _lang: &str) -> anyhow::Result<Option<String>> {
            Ok(Some("Test Translation".to_string()))
        }

        async fn get_metadata(&self, _node_id: &str, _key: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }

        async fn get_all_metadata(&self, _node_id: &str) -> anyhow::Result<HashMap<String, String>> {
            Ok(HashMap::new())
        }

        async fn node_exists(&self, node_id: &str) -> anyhow::Result<bool> {
            Ok(self.nodes.contains_key(node_id))
        }

        async fn get_all_nodes(&self) -> anyhow::Result<Vec<Node>> {
            Ok(self.nodes.values().cloned().collect())
        }

        async fn get_nodes_by_type(&self, _node_type: NodeType) -> anyhow::Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn insert_nodes_batch(&self, _nodes: &[crate::ImportedNode]) -> anyhow::Result<()> {
            Ok(())
        }

        async fn insert_edges_batch(&self, _edges: &[crate::ImportedEdge]) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_words_in_ayahs(&self, _ayah_node_ids: &[String]) -> anyhow::Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn get_adjacent_words(&self, _word_node_id: &str) -> anyhow::Result<(Option<Node>, Option<Node>)> {
            Ok((None, None))
        }
    }

    // Mock UserRepository
    struct MockUserRepo {
        due_states: std::sync::Mutex<Vec<MemoryState>>,
        session_state: std::sync::Mutex<Vec<String>>,
        stats: std::sync::Mutex<HashMap<String, String>>,
    }

    impl MockUserRepo {
        fn new() -> Self {
            Self {
                due_states: std::sync::Mutex::new(Vec::new()),
                session_state: std::sync::Mutex::new(Vec::new()),
                stats: std::sync::Mutex::new(HashMap::new()),
            }
        }

        fn set_due_states(&self, states: Vec<MemoryState>) {
            *self.due_states.lock().unwrap() = states;
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn get_memory_state(&self, _user_id: &str, _node_id: &str) -> anyhow::Result<Option<MemoryState>> {
            Ok(None)
        }

        async fn save_memory_state(&self, _state: &MemoryState) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_due_states(
            &self,
            _user_id: &str,
            _due_before: chrono::DateTime<Utc>,
            _limit: u32,
        ) -> anyhow::Result<Vec<MemoryState>> {
            Ok(self.due_states.lock().unwrap().clone())
        }

        async fn update_energy(&self, _user_id: &str, _node_id: &str, _new_energy: f64) -> anyhow::Result<()> {
            Ok(())
        }

        async fn log_propagation(&self, _event: &crate::PropagationEvent) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_session_state(&self) -> anyhow::Result<Vec<String>> {
            Ok(self.session_state.lock().unwrap().clone())
        }

        async fn save_session_state(&self, node_ids: &[String]) -> anyhow::Result<()> {
            *self.session_state.lock().unwrap() = node_ids.to_vec();
            Ok(())
        }

        async fn clear_session_state(&self) -> anyhow::Result<()> {
            self.session_state.lock().unwrap().clear();
            Ok(())
        }

        async fn get_stat(&self, key: &str) -> anyhow::Result<Option<String>> {
            Ok(self.stats.lock().unwrap().get(key).cloned())
        }

        async fn set_stat(&self, key: &str, value: &str) -> anyhow::Result<()> {
            self.stats.lock().unwrap().insert(key.to_string(), value.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_get_due_items_returns_scored_items() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = SessionService::new(content_repo, user_repo.clone());

        let now = Utc::now();
        let overdue_time = now - Duration::try_days(5).unwrap();

        user_repo.set_due_states(vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "word_1".to_string(),
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: overdue_time,
                due_at: overdue_time,
                review_count: 3,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "word_2".to_string(),
                stability: 5.0,
                difficulty: 6.0,
                energy: 0.6,
                last_reviewed: now,
                due_at: now,
                review_count: 1,
            },
        ]);

        // Act
        let result = service.get_due_items("user1", 10, false).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2);
        assert!(items[0].priority_score > 0.0);
    }

    #[tokio::test]
    async fn test_high_yield_mode_emphasizes_importance() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = SessionService::new(content_repo, user_repo.clone());

        let now = Utc::now();

        user_repo.set_due_states(vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "word_1".to_string(),
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 3,
            },
        ]);

        // Act
        let normal_result = service.get_due_items("user1", 10, false).await.unwrap();
        let high_yield_result = service.get_due_items("user1", 10, true).await.unwrap();

        // Assert - High yield mode should produce different scores
        assert_eq!(normal_result.len(), 1);
        assert_eq!(high_yield_result.len(), 1);

        // High yield mode has w_yield = 10.0 vs 1.5 in normal mode
        // So high yield score should be higher for same item
        assert!(high_yield_result[0].priority_score > normal_result[0].priority_score);
    }

    #[tokio::test]
    async fn test_filters_by_node_type() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = SessionService::new(content_repo, user_repo.clone());

        let now = Utc::now();

        user_repo.set_due_states(vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "word_1".to_string(),
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 3,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "verse_1".to_string(),
                stability: 5.0,
                difficulty: 6.0,
                energy: 0.6,
                last_reviewed: now,
                due_at: now,
                review_count: 1,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "surah_1".to_string(),  // Should be filtered out
                stability: 3.0,
                difficulty: 4.0,
                energy: 0.5,
                last_reviewed: now,
                due_at: now,
                review_count: 0,
            },
        ]);

        // Act
        let result = service.get_due_items("user1", 10, false).await;

        // Assert - Should only include WordInstance and Verse types
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2, "Should filter out Surah type");

        for item in &items {
            assert!(
                matches!(item.node.node_type, NodeType::WordInstance | NodeType::Verse),
                "Only WordInstance and Verse should be included"
            );
        }
    }

    #[tokio::test]
    async fn test_sorts_by_priority_descending() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = SessionService::new(content_repo, user_repo.clone());

        let now = Utc::now();
        let very_overdue = now - Duration::try_days(10).unwrap();
        let slightly_overdue = now - Duration::try_days(1).unwrap();

        user_repo.set_due_states(vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "word_1".to_string(),
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.9,  // High energy, low need
                last_reviewed: slightly_overdue,
                due_at: slightly_overdue,
                review_count: 3,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "word_2".to_string(),
                stability: 5.0,
                difficulty: 6.0,
                energy: 0.1,  // Low energy, high need
                last_reviewed: very_overdue,
                due_at: very_overdue,
                review_count: 1,
            },
        ]);

        // Act
        let result = service.get_due_items("user1", 10, false).await;

        // Assert - word_2 should be higher priority (more overdue + higher need)
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2);

        // First item should be word_2 (more overdue, higher need)
        assert_eq!(items[0].node.id, "word_2");
        assert!(items[0].priority_score > items[1].priority_score);
    }

    #[tokio::test]
    async fn test_respects_limit() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = SessionService::new(content_repo, user_repo.clone());

        let now = Utc::now();

        user_repo.set_due_states(vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "word_1".to_string(),
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.3,
                last_reviewed: now,
                due_at: now,
                review_count: 3,
            },
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "word_2".to_string(),
                stability: 5.0,
                difficulty: 6.0,
                energy: 0.6,
                last_reviewed: now,
                due_at: now,
                review_count: 1,
            },
        ]);

        // Act
        let result = service.get_due_items("user1", 1, false).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1, "Should respect limit parameter");
    }

    #[tokio::test]
    async fn test_session_state_management() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = SessionService::new(content_repo, user_repo.clone());

        let node_ids = vec!["word_1".to_string(), "word_2".to_string()];

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
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = SessionService::new(content_repo, user_repo.clone());

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
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = SessionService::new(content_repo, user_repo.clone());

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
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = SessionService::new(content_repo, user_repo.clone());

        let now = Utc::now();

        user_repo.set_due_states(vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "word_1".to_string(),
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.2,  // mastery_gap should be 0.8
                last_reviewed: now,
                due_at: now,
                review_count: 3,
            },
        ]);

        // Act
        let result = service.get_due_items("user1", 10, false).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);

        let mastery_gap = items[0].mastery_gap;
        assert!((mastery_gap - 0.8).abs() < 0.001, "Mastery gap should be 1.0 - energy");
    }

    #[tokio::test]
    async fn test_calculates_days_overdue_correctly() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = SessionService::new(content_repo, user_repo.clone());

        let now = Utc::now();
        let three_days_ago = now - Duration::try_days(3).unwrap();

        user_repo.set_due_states(vec![
            MemoryState {
                user_id: "user1".to_string(),
                node_id: "word_1".to_string(),
                stability: 10.0,
                difficulty: 5.0,
                energy: 0.5,
                last_reviewed: three_days_ago,
                due_at: three_days_ago,
                review_count: 3,
            },
        ]);

        // Act
        let result = service.get_due_items("user1", 10, false).await;

        // Assert
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);

        let days_overdue = items[0].days_overdue;
        assert!((2.9..=3.1).contains(&days_overdue),
            "Days overdue should be approximately 3, got {}", days_overdue);
    }
}
