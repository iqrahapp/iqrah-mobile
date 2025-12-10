use iqrah_core::testing::MockContentRepository;
use iqrah_iss::{SanitySummary, Scenario, SimulationConfig, Simulator, StudentSanityData};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

// --- Mock Setup from integration_tests.rs (condensed) ---

fn create_test_content_repo_for_repro() -> MockContentRepository {
    let mut mock = MockContentRepository::new();

    // Mock get_nodes_for_goal - 10 items
    mock.expect_get_nodes_for_goal()
        .returning(|_| Ok((0..10).collect()));

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrency_determinism() {
    // Explicitly cast to trait object to satisfy Simulator::new
    let content_repo: Arc<dyn iqrah_core::ports::ContentRepository> =
        Arc::new(create_test_content_repo_for_repro());
    // Config: 3 students
    // Remove target_days as it is not in SimulationConfig
    let config = SimulationConfig {
        base_seed: 12345,
        ..SimulationConfig::minimal_test()
    };

    // Scenario controls duration
    let mut scenario = Scenario::minimal_test();
    scenario.target_days = 10;

    let num_students = 3;

    // --- Sequential Run ---
    let mut seq_results = Vec::new();
    for i in 0..num_students {
        let sim = Simulator::new(Arc::clone(&content_repo), config.clone());
        let res = sim
            .simulate_student(&scenario, i)
            .await
            .expect("Seq simulation failed");
        // tuple: (metrics, debug_sum, sanity)
        let sanity = res.2.expect("Sanity data missing");
        seq_results.push(sanity);
    }

    // --- Parallel Run (Rayon) ---
    // Replicating comparison.rs logic
    let scenario_clone = scenario.clone();
    let content_repo_clone = Arc::clone(&content_repo);
    let config_clone = config.clone();

    let par_results: Vec<StudentSanityData> = tokio::task::block_in_place(|| {
        (0..num_students)
            .into_par_iter()
            .map(|i| {
                let rt = Runtime::new().unwrap();
                let sim = Simulator::new(Arc::clone(&content_repo_clone), config_clone.clone());
                let res = rt
                    .block_on(sim.simulate_student(&scenario_clone, i))
                    .expect("Par simulation failed");
                res.2.expect("Sanity data missing")
            })
            .collect()
    });

    // --- Assertion ---
    assert_eq!(seq_results.len(), par_results.len());

    for i in 0..num_students {
        let s_seq = &seq_results[i];
        let s_par = &par_results[i];

        println!(
            "Student {}: Seq Seen={}, Par Seen={}",
            i, s_seq.unique_items_seen, s_par.unique_items_seen
        );
        assert_eq!(
            s_seq.unique_items_seen, s_par.unique_items_seen,
            "Student {} unique_items_seen mismatch",
            i
        );

        // Check energy histogram buckets
        assert_eq!(
            s_seq.energy_histogram.total(),
            s_par.energy_histogram.total(),
            "Energy histogram total mismatch"
        );
        assert_eq!(
            s_seq.energy_histogram.bucket_0_0_2,
            s_par.energy_histogram.bucket_0_0_2
        );

        // Check retrievability histogram
        assert_eq!(
            s_seq.retrievability_histogram.bucket_0_9_1_0,
            s_par.retrievability_histogram.bucket_0_9_1_0
        );

        // Check daily reviews
        assert_eq!(
            s_seq.daily_review_counts, s_par.daily_review_counts,
            "Daily review counts mismatch"
        );
    }

    // Aggregation check
    let seq_summary = SanitySummary::from_students("test", &seq_results);
    let par_summary = SanitySummary::from_students("test", &par_results);

    assert!((seq_summary.unique_items_seen_mean - par_summary.unique_items_seen_mean).abs() < 1e-9);
    assert!(
        (seq_summary.avg_reviews_per_seen_item_mean - par_summary.avg_reviews_per_seen_item_mean)
            .abs()
            < 1e-9
    );

    println!("Concurrency Test PASSED: Results are bit-identical.");
}
