use iqrah_core::scheduler_v2::session_generator::{generate_session, SessionMode};
use iqrah_core::scheduler_v2::types::UserProfile;
use iqrah_core::scheduler_v2::{CandidateNode, SessionMixConfig};
use iqrah_iss::config::compute_min_new_for_plan;
use std::collections::HashMap;

#[test]
fn test_min_new_computation() {
    // Juz Amma params: 564 items, 180 days, session 15
    let items = 564;
    let days = 180;
    let session = 15;

    let min_new = compute_min_new_for_plan(items, days, session);
    println!("Min new: {}", min_new);
    // 180*0.8 = 144. 564/144 = 3.91 -> 4.
    // Cap: 15*0.3 = 4.5 -> 4.
    assert_eq!(min_new, 4);
}

#[test]
fn test_generate_session_includes_min_new() {
    // Setup candidates: 12 old (high score), 40 new (low score).
    // Total 52. Top-K (K=45) should include 12 old + 33 new.

    let mut candidates = Vec::new();

    // 12 Old items (Due)
    for i in 0..12 {
        candidates.push(CandidateNode {
            id: i as i64,
            energy: 0.5,
            next_due_ts: 100, // Due
            difficulty_score: 0.5,
            foundational_score: 0.5,
            influence_score: 0.5,
            quran_order: i * 10,
        });
    }

    // 40 New items
    for i in 12..52 {
        candidates.push(CandidateNode {
            id: i as i64,
            energy: 0.0,
            next_due_ts: 0, // New
            difficulty_score: 0.5,
            foundational_score: 0.5,
            influence_score: 0.5,
            quran_order: i * 10,
        });
    }

    // 10 Failed items (Energy 0, Due > 0) - Should be ReallyStruggling, NOT New
    for i in 52..62 {
        candidates.push(CandidateNode {
            id: i as i64,
            energy: 0.0,
            next_due_ts: 100, // Due (Failed)
            difficulty_score: 0.5,
            foundational_score: 0.5,
            influence_score: 0.5,
            quran_order: i * 10,
        });
    }

    let profile = UserProfile::balanced();
    let session_size = 15;
    let now_ts = 200; // > 100, so old items are overdue

    let mut mix_config = SessionMixConfig::default();
    mix_config.min_new_per_session = 4;
    mix_config.pct_new = 0.0; // Force reliance on min_new

    let session = generate_session(
        candidates,
        HashMap::new(), // parent_map
        HashMap::new(), // parent_energies
        &profile,
        session_size,
        now_ts,
        SessionMode::MixedLearning,
        Some(&mix_config),
    );

    println!("Session: {:?}", session);

    // Check count of FRESH new items (id 12..52)
    let fresh_count = session.iter().filter(|&&id| id >= 12 && id < 52).count();
    // Check count of FAILED items (id 52..62)
    let failed_count = session.iter().filter(|&&id| id >= 52).count();

    println!(
        "Fresh count: {}, Failed count: {}",
        fresh_count, failed_count
    );

    // With the fix:
    // target_new = 4. (Fresh items).
    // target_really_struggling = 15 * 0.1 = 1.5 -> 2. (Failed items).
    // So we expect 4 Fresh items.
    // If bug exists, Failed items (which sort higher) would consume "New" slots, leaving 0 Fresh items.
    assert_eq!(
        fresh_count, 4,
        "Should include exactly min_new=4 FRESH items"
    );
    // We also expect some failed items to be picked as ReallyStruggling
    assert!(
        failed_count >= 1,
        "Should include some failed items as ReallyStruggling"
    );
}
