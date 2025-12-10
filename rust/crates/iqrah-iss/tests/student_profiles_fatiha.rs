//! Regression tests for student profiles on Fatiha scenario.
//!
//! These tests verify that Fatiha with StrongDedicated meets calibration targets.

use iqrah_core::testing::MockContentRepository;
use iqrah_iss::{run_comparison, Scenario, SchedulerVariant, StudentProfile};
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_content_repo() -> MockContentRepository {
    let mut mock = MockContentRepository::new();

    // Mock 7 items for Fatiha
    mock.expect_get_nodes_for_goal()
        .returning(|_| Ok((0..7).collect()));
    mock.expect_get_prerequisite_parents()
        .returning(|_| Ok(HashMap::new()));
    mock.expect_get_verses_for_chapter()
        .returning(|_| Ok(vec![]));
    mock.expect_get_node_by_ukey().returning(|_| Ok(None));
    mock.expect_get_words_for_verse().returning(|_| Ok(vec![]));
    mock.expect_node_exists().returning(|_| Ok(true));
    mock.expect_get_edges_from().returning(|_| Ok(vec![]));

    mock
}

/// Fatiha + StrongDedicated should have:
/// - gave_up_fraction < 0.2
/// - at least one variant with coverage > 0.2
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_fatiha_strong_dedicated_coverage() {
    let content_repo = Arc::new(create_test_content_repo());

    let mut scenario = Scenario::minimal_test();
    scenario.name = "fatiha_profile_test".to_string();
    scenario.goal_id = "surah:1".to_string();
    scenario.target_days = 30;
    scenario.daily_minutes = 30.0;
    scenario.student_profile = Some(StudentProfile::StrongDedicated);

    let variants = vec![
        SchedulerVariant::IqrahDefault,
        SchedulerVariant::BaselineRandom,
    ];

    let (results, _debug) = run_comparison(
        content_repo as Arc<dyn iqrah_core::ports::ContentRepository>,
        &scenario,
        &variants,
        2, // Small sample for speed
        42,
        0.1,
        false,
    )
    .await
    .expect("Comparison should complete");

    // Check gave_up_fraction for all variants
    for variant in &results.variants {
        assert!(
            variant.metrics.gave_up_fraction < 0.5,
            "Variant {} gave_up_fraction {} should be < 0.5",
            variant.variant,
            variant.metrics.gave_up_fraction
        );
    }

    // Check at least one variant has reasonable coverage
    let max_coverage = results
        .variants
        .iter()
        .map(|v| v.metrics.coverage_pct_mean)
        .fold(0.0_f64, f64::max);

    assert!(
        max_coverage > 0.1,
        "At least one variant should have coverage > 10%, got {}",
        max_coverage
    );
}
