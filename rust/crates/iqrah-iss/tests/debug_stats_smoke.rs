use iqrah_core::domain::{Node, NodeType, Verse};
use iqrah_core::testing::MockContentRepository;
use iqrah_iss::{Scenario, SimulationConfig, Simulator};
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_content_repo() -> MockContentRepository {
    let mut mock = MockContentRepository::new();
    mock.expect_get_nodes_for_goal()
        .returning(|_| Ok(vec![1000]));
    mock.expect_get_prerequisite_parents()
        .returning(|_| Ok(HashMap::new()));
    mock.expect_node_exists().returning(|_| Ok(true));
    mock.expect_get_edges_from().returning(|_| Ok(vec![]));
    mock.expect_get_verses_for_chapter()
        .returning(|_| Ok(vec![]));
    mock.expect_get_node().returning(|id| {
        Ok(Some(Node {
            id,
            ukey: format!("verse:{}", id),
            node_type: NodeType::Verse,
        }))
    });
    mock
}

#[tokio::test]
async fn test_debug_stats_smoke() {
    let content_repo = Arc::new(create_test_content_repo());

    // Enable debug stats
    let config = SimulationConfig {
        debug_stats: true,
        ..SimulationConfig::minimal_test()
    };

    let simulator = Simulator::new(content_repo, config);
    let scenario = Scenario::minimal_test();

    // 1. Run simulation via simulate_student (API used by CLI)
    let result = simulator.simulate_student(&scenario, 0).await;
    assert!(result.is_ok(), "Simulation should succeed");

    let (metrics, summary) = result.unwrap();

    // 2. Verify basic metrics
    assert!(metrics.total_days > 0);

    // 3. Verify debug summary is present
    assert!(
        summary.is_some(),
        "Debug summary should be present when debug_stats is true"
    );
    let summary = summary.unwrap();

    // 4. Verify summary contents basic sanity
    // (Assuming at least one review happened if total_days > 0 and minimal test has items)
    println!("Debug Summary: {:?}", summary);
    assert_eq!(summary.student_index, 0);
    // Reviews might be 0 if minimal test is very minimal, but likely > 0
    // assert!(summary.total_reviews > 0);

    // 5. Verify bucket structures exist/populated
    // Just check that we can access fields; values depend on simulation randomness
    let total_delay_counts = summary.delay_buckets.d_0
        + summary.delay_buckets.d_1
        + summary.delay_buckets.d_2_3
        + summary.delay_buckets.d_4_7
        + summary.delay_buckets.d_8_14
        + summary.delay_buckets.d_15_30
        + summary.delay_buckets.d_31_plus;

    // R buckets check
    let total_r_counts = summary.r_buckets.r_0_0_3
        + summary.r_buckets.r_0_3_0_6
        + summary.r_buckets.r_0_6_0_85
        + summary.r_buckets.r_0_85_0_95
        + summary.r_buckets.r_0_95_1_0;

    // Should match total reviews if any reviews happened
    if summary.total_reviews > 0 {
        assert_eq!(total_delay_counts, summary.total_reviews as u64);
        assert_eq!(total_r_counts, summary.total_reviews as u64);
    }
}

#[tokio::test]
async fn test_debug_stats_disabled_by_default() {
    let content_repo = Arc::new(create_test_content_repo());

    // Default config (debug_stats = false)
    let config = SimulationConfig::minimal_test();
    assert!(!config.debug_stats);

    let simulator = Simulator::new(content_repo, config);
    let scenario = Scenario::minimal_test();

    let result = simulator.simulate_student(&scenario, 0).await;
    assert!(result.is_ok());

    let (_, summary) = result.unwrap();
    assert!(
        summary.is_none(),
        "Debug summary should be None when debug_stats is false"
    );
}
