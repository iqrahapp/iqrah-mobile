pub mod api;
mod app;
mod database;
mod frb_generated;
mod repository;
mod sqlite_repo;

// Re-export scheduler for internal use
pub use iqrah_scheduler as scheduler;

#[cfg(test)]
mod tests {
    use crate::repository::{
        KnowledgeGraphRepository, LearningService, MemoryState, NodeData, ReviewGrade,
    };
    use crate::sqlite_repo::SqliteRepository;
    use std::sync::Arc;

    // Unit test with mock repo
    struct MockRepository;

    impl KnowledgeGraphRepository for MockRepository {
        fn get_due_items(&self, _user_id: &str, limit: u32) -> anyhow::Result<Vec<NodeData>> {
            Ok((0..limit)
                .map(|i| NodeData {
                    id: format!("test_{}", i),
                    arabic: "تست".to_string(),
                    translation: "test".to_string(),
                })
                .collect())
        }

        fn get_node_data(&self, node_id: &str) -> anyhow::Result<NodeData> {
            Ok(NodeData {
                id: node_id.to_string(),
                arabic: "الْحَمْدُ".to_string(),
                translation: "All praise".to_string(),
            })
        }

        fn process_review(
            &self,
            _user_id: &str,
            _node_id: &str,
            _grade: ReviewGrade,
        ) -> anyhow::Result<MemoryState> {
            Ok(MemoryState {
                stability: 2.0,
                difficulty: 4.0,
                due_at: chrono::Utc::now().timestamp_millis() + 86400000, // +1 day
                review_count: 1,
                last_reviewed: chrono::Utc::now().timestamp_millis(),
            })
        }
    }

    #[test]
    fn test_service_with_mock_repo() {
        let mock_repo = Arc::new(MockRepository);
        let service = LearningService::new(mock_repo);

        // Test getting due items
        let items = service.get_due_items("test_user", 5).unwrap();
        assert_eq!(items.len(), 5);
        assert_eq!(items[0].id, "test_0");

        // Test processing review
        let result = service
            .process_review("test_user", "test_0", ReviewGrade::Good)
            .unwrap();
        assert_eq!(result.stability, 2.0);
        assert_eq!(result.review_count, 1);
    }

    #[test]
    fn test_integration_full_learning_cycle() {
        let repo = Arc::new(SqliteRepository::new(None).unwrap());
        repo.seed().unwrap();

        let service = LearningService::new(repo);
        let user_id = "default_user";

        // Step 1: Get initial due count (whatever it is)
        let initial_due_items = service.get_due_items(user_id, 100).unwrap();
        let initial_count = initial_due_items.len();
        assert!(
            initial_count > 0,
            "Should have some due items after seeding"
        );

        // Step 2: Pick first item to review
        let first_item = &initial_due_items[0];
        assert!(
            !first_item.arabic.is_empty(),
            "Items should have Arabic text"
        );
        assert!(
            !first_item.translation.is_empty(),
            "Items should have translations"
        );

        // Step 3: Process review - the core behavior we're testing
        let memory_state = service
            .process_review(user_id, &first_item.id, ReviewGrade::Good)
            .unwrap();
        assert!(
            memory_state.stability > 0.0,
            "Review should update stability"
        );
        assert!(
            memory_state.review_count > 0,
            "Review count should increment"
        );
        assert!(
            memory_state.due_at > chrono::Utc::now().timestamp_millis(),
            "Should be due in future"
        );

        // Step 4: Verify the behavior - due count decreased by exactly 1
        let after_review_items = service.get_due_items(user_id, 100).unwrap();
        assert_eq!(
            after_review_items.len(),
            initial_count - 1,
            "Due count should decrease by 1 after review"
        );

        // Step 5: Verify the reviewed item is excluded
        let remaining_items = service.get_due_items(user_id, 100).unwrap();
        assert!(
            !remaining_items.iter().any(|item| item.id == first_item.id),
            "Reviewed item should not appear in due items"
        );

        // Step 6: Test that different grades produce different outcomes
        if let Some(second_item) = remaining_items.first() {
            let again_result = service
                .process_review(user_id, &second_item.id, ReviewGrade::Again)
                .unwrap();
            let good_result = memory_state; // From earlier

            // Verify Again gives worse scheduling than Good (behavior, not specific values)
            assert!(
                again_result.stability < good_result.stability,
                "Again should give lower stability than Good"
            );
            assert!(
                again_result.due_at < good_result.due_at,
                "Again should be due sooner than Good"
            );
        }

        println!("✅ Learning cycle works: items can be reviewed and scheduling updates correctly");
    }

    #[test]
    fn test_node_metadata_retrieval() {
        let repo = Arc::new(SqliteRepository::new(None).unwrap());
        repo.seed().unwrap();

        let service = LearningService::new(repo);

        // Test getting specific node data
        let node = service.get_node_data("fatiha_01").unwrap();
        assert_eq!(node.id, "fatiha_01");
        assert_eq!(node.arabic, "بِسْمِ");
        assert_eq!(node.translation, "In (the) name");
    }
}
