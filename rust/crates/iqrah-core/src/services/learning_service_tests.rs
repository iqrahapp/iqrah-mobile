#[cfg(test)]
mod tests {
    use super::super::LearningService;
    use crate::{
        ContentRepository, DistributionType, Edge, EdgeType, MemoryState, Node, NodeType,
        PropagationEvent, ReviewGrade, UserRepository,
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;

    // Mock ContentRepository
    struct MockContentRepo {
        nodes: HashMap<String, Node>,
        edges: HashMap<String, Vec<Edge>>,
    }

    impl MockContentRepo {
        fn new() -> Self {
            let mut nodes = HashMap::new();
            let mut edges = HashMap::new();

            // Create test nodes
            nodes.insert(
                "word_1".to_string(),
                Node {
                    id: "word_1".to_string(),
                    node_type: NodeType::WordInstance,
                    knowledge_node: None,
                },
            );

            nodes.insert(
                "word_2".to_string(),
                Node {
                    id: "word_2".to_string(),
                    node_type: NodeType::WordInstance,
                    knowledge_node: None,
                },
            );

            // Create test edge (word_1 -> word_2)
            edges.insert(
                "word_1".to_string(),
                vec![Edge {
                    source_id: "word_1".to_string(),
                    target_id: "word_2".to_string(),
                    edge_type: EdgeType::Knowledge,
                    distribution_type: DistributionType::Const,
                    param1: 0.5,
                    param2: 0.0,
                }],
            );

            Self { nodes, edges }
        }
    }

    #[async_trait]
    impl ContentRepository for MockContentRepo {
        async fn get_node(&self, node_id: &str) -> anyhow::Result<Option<Node>> {
            Ok(self.nodes.get(node_id).cloned())
        }

        async fn get_edges_from(&self, source_id: &str) -> anyhow::Result<Vec<Edge>> {
            Ok(self.edges.get(source_id).cloned().unwrap_or_default())
        }

        async fn get_quran_text(&self, _node_id: &str) -> anyhow::Result<Option<String>> {
            Ok(Some("Test Arabic".to_string()))
        }

        async fn get_translation(
            &self,
            _node_id: &str,
            _lang: &str,
        ) -> anyhow::Result<Option<String>> {
            Ok(Some("Test Translation".to_string()))
        }

        async fn get_metadata(&self, _node_id: &str, _key: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }

        async fn get_all_metadata(
            &self,
            _node_id: &str,
        ) -> anyhow::Result<HashMap<String, String>> {
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

        async fn get_adjacent_words(
            &self,
            _word_node_id: &str,
        ) -> anyhow::Result<(Option<Node>, Option<Node>)> {
            Ok((None, None))
        }

        // V2 Stub Methods
        async fn get_chapter(
            &self,
            _chapter_number: i32,
        ) -> anyhow::Result<Option<crate::Chapter>> {
            Ok(None)
        }

        async fn get_chapters(&self) -> anyhow::Result<Vec<crate::Chapter>> {
            Ok(vec![])
        }

        async fn get_verse(&self, _verse_key: &str) -> anyhow::Result<Option<crate::Verse>> {
            Ok(None)
        }

        async fn get_verses_for_chapter(
            &self,
            _chapter_number: i32,
        ) -> anyhow::Result<Vec<crate::Verse>> {
            Ok(vec![])
        }

        async fn get_words_for_verse(&self, _verse_key: &str) -> anyhow::Result<Vec<crate::Word>> {
            Ok(vec![])
        }

        async fn get_word(&self, _word_id: i32) -> anyhow::Result<Option<crate::Word>> {
            Ok(None)
        }

        async fn get_languages(&self) -> anyhow::Result<Vec<crate::Language>> {
            Ok(vec![])
        }

        async fn get_language(&self, _code: &str) -> anyhow::Result<Option<crate::Language>> {
            Ok(None)
        }

        async fn get_translators_for_language(
            &self,
            _language_code: &str,
        ) -> anyhow::Result<Vec<crate::Translator>> {
            Ok(vec![])
        }

        async fn get_translator(
            &self,
            _translator_id: i32,
        ) -> anyhow::Result<Option<crate::Translator>> {
            Ok(None)
        }

        async fn get_translator_by_slug(
            &self,
            _slug: &str,
        ) -> anyhow::Result<Option<crate::Translator>> {
            Ok(None)
        }

        async fn get_verse_translation(
            &self,
            _verse_key: &str,
            _translator_id: i32,
        ) -> anyhow::Result<Option<String>> {
            Ok(None)
        }

        async fn get_word_translation(
            &self,
            _word_id: i32,
            _translator_id: i32,
        ) -> anyhow::Result<Option<String>> {
            Ok(None)
        }

        async fn insert_translator(
            &self,
            _slug: &str,
            _full_name: &str,
            _language_code: &str,
            _description: Option<&str>,
            _copyright_holder: Option<&str>,
            _license: Option<&str>,
            _website: Option<&str>,
            _version: Option<&str>,
            _package_id: Option<&str>,
        ) -> anyhow::Result<i32> {
            Ok(1)
        }

        async fn insert_verse_translation(
            &self,
            _verse_key: &str,
            _translator_id: i32,
            _translation: &str,
            _footnotes: Option<&str>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_available_packages(
            &self,
            _package_type: Option<crate::PackageType>,
            _language_code: Option<&str>,
        ) -> anyhow::Result<Vec<crate::ContentPackage>> {
            Ok(vec![])
        }

        async fn get_package(
            &self,
            _package_id: &str,
        ) -> anyhow::Result<Option<crate::ContentPackage>> {
            Ok(None)
        }

        async fn upsert_package(&self, _package: &crate::ContentPackage) -> anyhow::Result<()> {
            Ok(())
        }

        async fn delete_package(&self, _package_id: &str) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_installed_packages(&self) -> anyhow::Result<Vec<crate::InstalledPackage>> {
            Ok(vec![])
        }

        async fn is_package_installed(&self, _package_id: &str) -> anyhow::Result<bool> {
            Ok(false)
        }

        async fn mark_package_installed(&self, _package_id: &str) -> anyhow::Result<()> {
            Ok(())
        }

        async fn mark_package_uninstalled(&self, _package_id: &str) -> anyhow::Result<()> {
            Ok(())
        }

        async fn enable_package(&self, _package_id: &str) -> anyhow::Result<()> {
            Ok(())
        }

        async fn disable_package(&self, _package_id: &str) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_enabled_packages(&self) -> anyhow::Result<Vec<crate::InstalledPackage>> {
            Ok(vec![])
        }

        async fn get_scheduler_candidates(
            &self,
            _goal_id: &str,
            _user_id: &str,
            _now_ts: i64,
        ) -> anyhow::Result<Vec<crate::scheduler_v2::CandidateNode>> {
            Ok(vec![])
        }

        async fn get_prerequisite_parents(
            &self,
            _node_ids: &[String],
        ) -> anyhow::Result<HashMap<String, Vec<String>>> {
            Ok(HashMap::new())
        }

        async fn get_goal(
            &self,
            _goal_id: &str,
        ) -> anyhow::Result<Option<crate::ports::content_repository::SchedulerGoal>> {
            Ok(None)
        }

        async fn get_nodes_for_goal(&self, _goal_id: &str) -> anyhow::Result<Vec<String>> {
            Ok(vec![])
        }
    }

    // Mock UserRepository
    struct MockUserRepo {
        states: std::sync::Mutex<HashMap<String, MemoryState>>,
        propagation_events: std::sync::Mutex<Vec<PropagationEvent>>,
    }

    impl MockUserRepo {
        fn new() -> Self {
            Self {
                states: std::sync::Mutex::new(HashMap::new()),
                propagation_events: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn get_memory_state(
            &self,
            _user_id: &str,
            node_id: &str,
        ) -> anyhow::Result<Option<MemoryState>> {
            let states = self.states.lock().unwrap();
            Ok(states.get(node_id).cloned())
        }

        async fn save_memory_state(&self, state: &MemoryState) -> anyhow::Result<()> {
            let mut states = self.states.lock().unwrap();
            states.insert(state.node_id.clone(), state.clone());
            Ok(())
        }

        async fn get_due_states(
            &self,
            _user_id: &str,
            _due_before: chrono::DateTime<Utc>,
            _limit: u32,
        ) -> anyhow::Result<Vec<MemoryState>> {
            Ok(vec![])
        }

        async fn update_energy(
            &self,
            _user_id: &str,
            node_id: &str,
            new_energy: f64,
        ) -> anyhow::Result<()> {
            let mut states = self.states.lock().unwrap();
            if let Some(state) = states.get_mut(node_id) {
                state.energy = new_energy;
            }
            Ok(())
        }

        async fn log_propagation(&self, event: &PropagationEvent) -> anyhow::Result<()> {
            let mut events = self.propagation_events.lock().unwrap();
            events.push(event.clone());
            Ok(())
        }

        async fn get_session_state(&self) -> anyhow::Result<Vec<String>> {
            Ok(vec![])
        }

        async fn save_session_state(&self, _node_ids: &[String]) -> anyhow::Result<()> {
            Ok(())
        }

        async fn clear_session_state(&self) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_stat(&self, _key: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }

        async fn set_stat(&self, _key: &str, _value: &str) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_setting(&self, _key: &str) -> anyhow::Result<Option<String>> {
            Ok(None)
        }

        async fn set_setting(&self, _key: &str, _value: &str) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_parent_energies(
            &self,
            _user_id: &str,
            _node_ids: &[String],
        ) -> anyhow::Result<HashMap<String, f32>> {
            Ok(HashMap::new())
        }

        async fn get_memory_basics(
            &self,
            _user_id: &str,
            _node_ids: &[String],
        ) -> anyhow::Result<HashMap<String, crate::scheduler_v2::MemoryBasics>> {
            Ok(HashMap::new())
        }

        async fn get_bandit_arms(
            &self,
            _user_id: &str,
            _goal_group: &str,
        ) -> anyhow::Result<Vec<crate::scheduler_v2::BanditArmState>> {
            Ok(vec![])
        }

        async fn update_bandit_arm(
            &self,
            _user_id: &str,
            _goal_group: &str,
            _profile_name: &str,
            _successes: f32,
            _failures: f32,
        ) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_process_review_creates_new_state() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = LearningService::new(content_repo, user_repo.clone());

        // Act
        let result = service
            .process_review("user1", "word_1", ReviewGrade::Good)
            .await;

        // Assert
        assert!(result.is_ok());
        let state = result.unwrap();
        assert_eq!(state.user_id, "user1");
        assert_eq!(state.node_id, "word_1");
        assert_eq!(state.review_count, 1);
        assert!(state.energy > 0.0); // Energy should have increased
    }

    #[tokio::test]
    async fn test_process_review_increases_energy_on_good_grade() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = LearningService::new(content_repo, user_repo.clone());

        // Create initial state with some energy
        let initial_state = MemoryState {
            user_id: "user1".to_string(),
            node_id: "word_1".to_string(),
            stability: 1.0,
            difficulty: 5.0,
            energy: 0.5,
            last_reviewed: Utc::now(),
            due_at: Utc::now(),
            review_count: 1,
        };
        user_repo.save_memory_state(&initial_state).await.unwrap();

        // Act
        let result = service
            .process_review("user1", "word_1", ReviewGrade::Good)
            .await;

        // Assert
        assert!(result.is_ok());
        let new_state = result.unwrap();
        assert!(
            new_state.energy > initial_state.energy,
            "Energy should increase: {} -> {}",
            initial_state.energy,
            new_state.energy
        );
        assert_eq!(new_state.review_count, 2);
    }

    #[tokio::test]
    async fn test_process_review_decreases_energy_on_again_grade() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = LearningService::new(content_repo, user_repo.clone());

        // Create initial state with high energy
        let initial_state = MemoryState {
            user_id: "user1".to_string(),
            node_id: "word_1".to_string(),
            stability: 10.0,
            difficulty: 5.0,
            energy: 0.8,
            last_reviewed: Utc::now(),
            due_at: Utc::now(),
            review_count: 5,
        };
        user_repo.save_memory_state(&initial_state).await.unwrap();

        // Act
        let result = service
            .process_review("user1", "word_1", ReviewGrade::Again)
            .await;

        // Assert
        assert!(result.is_ok());
        let new_state = result.unwrap();
        assert!(
            new_state.energy < initial_state.energy,
            "Energy should decrease: {} -> {}",
            initial_state.energy,
            new_state.energy
        );
        assert_eq!(new_state.review_count, 6);
    }

    #[tokio::test]
    async fn test_energy_propagation_occurs() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = LearningService::new(content_repo, user_repo.clone());

        // Create states for both connected nodes
        let state1 = MemoryState {
            user_id: "user1".to_string(),
            node_id: "word_1".to_string(),
            stability: 1.0,
            difficulty: 5.0,
            energy: 0.3,
            last_reviewed: Utc::now(),
            due_at: Utc::now(),
            review_count: 1,
        };

        let state2 = MemoryState {
            user_id: "user1".to_string(),
            node_id: "word_2".to_string(),
            stability: 1.0,
            difficulty: 5.0,
            energy: 0.2,
            last_reviewed: Utc::now(),
            due_at: Utc::now(),
            review_count: 0,
        };

        user_repo.save_memory_state(&state1).await.unwrap();
        user_repo.save_memory_state(&state2).await.unwrap();

        // Act - review word_1 with Good grade
        let _ = service
            .process_review("user1", "word_1", ReviewGrade::Good)
            .await
            .unwrap();

        // Assert - Check that propagation event was logged
        let events = user_repo.propagation_events.lock().unwrap();
        assert!(!events.is_empty(), "Propagation event should be logged");
        assert_eq!(events[0].source_node_id, "word_1");
        assert!(
            !events[0].details.is_empty(),
            "Should have propagation details"
        );
        assert_eq!(events[0].details[0].target_node_id, "word_2");
    }

    #[tokio::test]
    async fn test_energy_bounded_between_0_and_1() {
        // Arrange
        let content_repo = Arc::new(MockContentRepo::new());
        let user_repo = Arc::new(MockUserRepo::new());
        let service = LearningService::new(content_repo, user_repo.clone());

        // Create state with very high energy
        let high_energy_state = MemoryState {
            user_id: "user1".to_string(),
            node_id: "word_1".to_string(),
            stability: 50.0,
            difficulty: 2.0,
            energy: 0.99,
            last_reviewed: Utc::now(),
            due_at: Utc::now(),
            review_count: 20,
        };
        user_repo
            .save_memory_state(&high_energy_state)
            .await
            .unwrap();

        // Act - review with Easy grade (should try to increase energy)
        let result = service
            .process_review("user1", "word_1", ReviewGrade::Easy)
            .await
            .unwrap();

        // Assert - energy should be capped at 1.0
        assert!(result.energy <= 1.0, "Energy should not exceed 1.0");
        assert!(result.energy >= 0.0, "Energy should not be negative");
    }
}
