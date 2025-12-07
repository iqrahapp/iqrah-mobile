use iqrah_core::domain::{Node, NodeType, Verse};
use iqrah_core::ports::UserRepository;
use iqrah_core::testing::MockContentRepository;
use iqrah_iss::brain::{StudentParams, StudentParamsSelector};
use iqrah_iss::{Scenario, SimulationConfig, Simulator};
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_content_repo() -> MockContentRepository {
    let mut mock = MockContentRepository::new();

    // Mock get_nodes_for_goal - return 7 verses for Al-Fatihah
    mock.expect_get_nodes_for_goal()
        .returning(|_| Ok(vec![1001, 1002, 1003, 1004, 1005, 1006, 1007]));

    // Mock get_prerequisite_parents - no prerequisites
    mock.expect_get_prerequisite_parents()
        .returning(|_| Ok(HashMap::new()));

    // Mock node_exists - always true for our test items
    mock.expect_node_exists().returning(|_| Ok(true));

    // Mock get_edges_from - empty
    mock.expect_get_edges_from().returning(|_| Ok(vec![]));

    // Mock get_node
    mock.expect_get_node().returning(|id| {
        Ok(Some(Node {
            id,
            ukey: format!("verse:{}", id),
            node_type: NodeType::Verse,
        }))
    });

    // Mock get_verses_for_chapter - return Al-Fatihah verses for init
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

    mock
}

#[tokio::test]
async fn test_debug_fatiha_learning() {
    let content_repo = Arc::new(create_test_content_repo());
    let config = SimulationConfig::default();
    let simulator = Simulator::new(content_repo, config);

    let scenario = Scenario {
        name: "debug_fatiha".to_string(),
        scheduler: iqrah_iss::SchedulerVariant::IqrahDefault,
        goal_id: "surah:1".to_string(),
        target_days: 60,
        daily_minutes: 15.0,
        student_params: StudentParams::default(), // Uses default params, might be casual
        student_params_selector: None,
        session_size: 20,
        enable_bandit: false,
        student_count: 1,
    };

    println!("\nRunning Fatiha simulation for 60 days...");
    let (metrics, user_repo, _debug) = simulator
        .simulate_student_debug(&scenario, 0)
        .await
        .expect("Simulation failed");

    println!("\n=== Fatiha Debug Stats ===");
    println!("Total Days: {}", metrics.total_days);
    println!(
        "Items Mastered: {}/{}",
        metrics.items_mastered, metrics.goal_item_count
    );
    println!("Coverage(T): {:.1}%", metrics.coverage_t * 100.0);
    println!("Mean R(T): {:.2}", metrics.mean_r_t);
    println!("Cov(Acq): {:.1}%", metrics.coverage_acq * 100.0);
    println!("Mean R(Acq): {:.2}", metrics.mean_r_acq);

    let goal_items = vec![1001, 1002, 1003, 1004, 1005, 1006, 1007];
    let mut total_reviews = 0;
    let mut total_stability = 0.0;

    println!(
        "\n{:<10} {:<10} {:<10} {:<30} {:<30}",
        "Node ID", "Reviews", "Stability", "Last Reviewed", "Next Due"
    );
    println!("{}", "-".repeat(100));

    for node_id in &goal_items {
        if let Ok(Some(state)) = user_repo
            .get_memory_state(&format!("sim_student_0"), *node_id)
            .await
        {
            println!(
                "{:<10} {:<10} {:<10.2} {:<30} {:<30}",
                node_id,
                state.review_count,
                state.stability,
                state.last_reviewed.to_string(),
                state.due_at.to_string()
            );
            total_reviews += state.review_count;
            total_stability += state.stability;
        } else {
            println!("{:<10} NOT FOUND", node_id);
        }
    }

    println!("\nTotal Reviews: {}", total_reviews);
    println!(
        "Avg Reviews: {:.1}",
        total_reviews as f64 / goal_items.len() as f64
    );
    println!(
        "Avg Stability: {:.2}",
        total_stability / goal_items.len() as f64
    );
}
