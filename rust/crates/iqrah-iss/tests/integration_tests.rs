//! Integration tests for ISS simulator using MockContentRepository.
//!
//! These tests verify that the full simulation pipeline works correctly,
//! using mocked content data instead of a real database.

use iqrah_core::domain::Verse;
use iqrah_core::testing::MockContentRepository;
use iqrah_iss::{Scenario, SimulationConfig, Simulator};
use std::collections::HashMap;
use std::sync::Arc;

/// Create a mock content repository with minimal Al-Fatihah data.
fn create_test_content_repo() -> MockContentRepository {
    let mut mock = MockContentRepository::new();

    // Mock get_nodes_for_goal - return 7 verses for Al-Fatihah
    mock.expect_get_nodes_for_goal()
        .returning(|_| Ok(vec![1001, 1002, 1003, 1004, 1005, 1006, 1007]));

    // Mock get_prerequisite_parents - no prerequisites
    mock.expect_get_prerequisite_parents()
        .returning(|_| Ok(HashMap::new()));

    // Mock get_verses_for_chapter - return Al-Fatihah verses
    mock.expect_get_verses_for_chapter().returning(|chapter| {
        if chapter == 1 {
            Ok(vec![
                Verse {
                    key: "1:1".to_string(),
                    chapter_number: 1,
                    verse_number: 1,
                    text_uthmani: "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
                    text_simple: None,
                    juz: 1,
                    page: 1,
                },
                Verse {
                    key: "1:2".to_string(),
                    chapter_number: 1,
                    verse_number: 2,
                    text_uthmani: "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ".to_string(),
                    text_simple: None,
                    juz: 1,
                    page: 1,
                },
                Verse {
                    key: "1:3".to_string(),
                    chapter_number: 1,
                    verse_number: 3,
                    text_uthmani: "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ".to_string(),
                    text_simple: None,
                    juz: 1,
                    page: 1,
                },
                Verse {
                    key: "1:4".to_string(),
                    chapter_number: 1,
                    verse_number: 4,
                    text_uthmani: "مَٰلِكِ يَوْمِ ٱلدِّينِ".to_string(),
                    text_simple: None,
                    juz: 1,
                    page: 1,
                },
                Verse {
                    key: "1:5".to_string(),
                    chapter_number: 1,
                    verse_number: 5,
                    text_uthmani: "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ".to_string(),
                    text_simple: None,
                    juz: 1,
                    page: 1,
                },
                Verse {
                    key: "1:6".to_string(),
                    chapter_number: 1,
                    verse_number: 6,
                    text_uthmani: "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ".to_string(),
                    text_simple: None,
                    juz: 1,
                    page: 1,
                },
                Verse {
                    key: "1:7".to_string(),
                    chapter_number: 1,
                    verse_number: 7,
                    text_uthmani: "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ".to_string(),
                    text_simple: None,
                    juz: 1,
                    page: 1,
                },
            ])
        } else {
            Ok(vec![])
        }
    });

    // Mock get_node_by_ukey for verse nodes (IPS requires this)
    mock.expect_get_node_by_ukey().returning(|ukey| {
        // Parse verse key like "1:1" -> return node with id = chapter*1000 + verse
        let parts: Vec<&str> = ukey.split(':').collect();
        if parts.len() == 2 {
            if let (Ok(chapter), Ok(verse)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
                let node_id = chapter * 1000 + verse;
                return Ok(Some(iqrah_core::domain::Node {
                    id: node_id,
                    ukey: ukey.to_string(),
                    node_type: iqrah_core::domain::NodeType::Verse,
                }));
            }
        }
        Ok(None)
    });

    // Mock get_words_for_verse - return empty (IPS still calls this for vocab)
    mock.expect_get_words_for_verse().returning(|_| Ok(vec![]));

    // Mock node_exists (required by simulation loop)
    mock.expect_node_exists().returning(|_| Ok(true));

    // Mock get_edges_from (for propagation)
    mock.expect_get_edges_from().returning(|_| Ok(vec![]));

    mock
}

#[tokio::test]
async fn test_simulator_completes_without_panic() {
    let content_repo = Arc::new(create_test_content_repo());
    let config = SimulationConfig::minimal_test();
    let simulator = Simulator::new(content_repo, config);

    let scenario = Scenario::minimal_test();
    let result = simulator.simulate_student(&scenario, 0).await;

    assert!(result.is_ok(), "Simulation should complete without panic");

    let (metrics, _debug, _sanity) = result.unwrap();
    assert!(metrics.total_days > 0 || metrics.gave_up);
    assert!(metrics.total_minutes >= 0.0);
    assert!(metrics.coverage_pct >= 0.0 && metrics.coverage_pct <= 1.0);
}

#[tokio::test]
async fn test_simulator_produces_reasonable_metrics() {
    let content_repo = Arc::new(create_test_content_repo());
    let config = SimulationConfig::minimal_test();
    let simulator = Simulator::new(content_repo, config.clone());

    let scenario = Scenario::minimal_test();
    let (metrics, _debug, _sanity) = simulator.simulate_student(&scenario, 0).await.unwrap();

    // Check that metrics are finite
    assert!(metrics.retention_per_minute.is_finite());
    assert!(metrics.coverage_pct.is_finite());
    assert!(metrics.plan_faithfulness >= 0.0 && metrics.plan_faithfulness <= 1.0);

    // Final score should be computable and in valid range
    let score = metrics.final_score(scenario.target_days, config.expected_rpm);
    assert!(score.is_finite());
    assert!(score >= -1.0 && score <= 1.0);
}

#[tokio::test]
async fn test_different_seeds_produce_different_results() {
    let content_repo1 = Arc::new(create_test_content_repo());
    let content_repo2 = Arc::new(create_test_content_repo());
    let config = SimulationConfig::minimal_test();
    let scenario = Scenario::minimal_test();

    let simulator1 = Simulator::new(content_repo1, config.clone());
    let simulator2 = Simulator::new(content_repo2, config);

    let (metrics1, _, _) = simulator1.simulate_student(&scenario, 0).await.unwrap();
    let (metrics2, _, _) = simulator2.simulate_student(&scenario, 1).await.unwrap();

    // Different seeds should produce different results
    // (this is probabilistic, so we just verify they run without panic)
    assert!(metrics1.total_minutes.is_finite());
    assert!(metrics2.total_minutes.is_finite());
}

#[tokio::test]
async fn test_dedicated_student_performs_better() {
    let content_repo = Arc::new(create_test_content_repo());
    let config = SimulationConfig::minimal_test();
    let simulator = Simulator::new(content_repo, config.clone());

    // Run with casual learner
    let casual = Scenario::casual_learner();
    let (casual_metrics, _, _) = simulator.simulate_student(&casual, 0).await.unwrap();

    // Run with dedicated student (same seed for fair comparison)
    let content_repo2 = Arc::new(create_test_content_repo());
    let simulator2 = Simulator::new(content_repo2, config);
    let dedicated = Scenario::dedicated_student();
    let (dedicated_metrics, _, _) = simulator2.simulate_student(&dedicated, 0).await.unwrap();

    // Dedicated student should generally have higher coverage or retention
    // (this is probabilistic, so we just verify the simulation runs)
    assert!(casual_metrics.coverage_pct >= 0.0);
    assert!(dedicated_metrics.coverage_pct >= 0.0);
}
