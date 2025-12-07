/// Session generation orchestrator for Scheduler v2.0
///
/// This module implements the main session generation logic, including:
/// - Prerequisite Mastery Gate
/// - Priority scoring and ranking
/// - Difficulty-based composition with fallback
use crate::scheduler_v2::{
    calculate_days_overdue, calculate_priority_score, calculate_readiness,
    count_unsatisfied_parents, CandidateNode, InMemNode, ParentEnergyMap, SessionMixConfig,
    UserProfile,
};
use std::collections::HashMap;

// ============================================================================
// SESSION MODE
// ============================================================================

/// Session mode determines candidate filtering and composition strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode {
    /// Revision mode: Only review previously seen items (no new content).
    /// Composition: Mix by content difficulty (60% easy, 30% medium, 10% hard).
    Revision,

    /// Mixed learning mode: Mix of new and due content.
    /// Composition: Mix by mastery bands (configurable, default 10/10/50/20/10).
    MixedLearning,
}

// ============================================================================
// DIFFICULTY BUCKETS
// ============================================================================

/// Difficulty bucket for content difficulty-based composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DifficultyBucket {
    Easy,   // difficulty_score < 0.4
    Medium, // 0.4 <= difficulty_score < 0.7
    Hard,   // difficulty_score >= 0.7
}

impl DifficultyBucket {
    fn from_score(score: f32) -> Self {
        if score < 0.4 {
            Self::Easy
        } else if score < 0.7 {
            Self::Medium
        } else {
            Self::Hard
        }
    }
}

// ============================================================================
// MASTERY BANDS
// ============================================================================

/// Mastery band for user energy-based composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MasteryBand {
    New,              // energy == 0.0
    ReallyStruggling, // 0.0 < energy <= 0.2
    Struggling,       // 0.2 < energy <= 0.4
    AlmostThere,      // 0.4 < energy <= 0.7
    AlmostMastered,   // 0.7 < energy <= 1.0
}

impl MasteryBand {
    fn from_energy(energy: f32) -> Self {
        if energy == 0.0 {
            Self::New
        } else if energy <= 0.2 {
            Self::ReallyStruggling
        } else if energy <= 0.4 {
            Self::Struggling
        } else if energy <= 0.7 {
            Self::AlmostThere
        } else {
            Self::AlmostMastered
        }
    }
}

// ============================================================================
// SESSION GENERATOR
// ============================================================================

/// Generates a session by applying the full scheduler v2.0 pipeline.
///
/// # Arguments
/// * `candidates` - All candidate nodes (from repository)
/// * `parent_map` - Map of node_id -> list of parent_ids (prereq edges)
/// * `parent_energies` - Map of parent_id -> energy value
/// * `profile` - User's learning profile (weights)
/// * `session_size` - Desired number of items in session
/// * `now_ts` - Current timestamp in MILLISECONDS
/// * `mode` - Session mode (Revision or MixedLearning)
/// * `mix_config` - Optional session mix config (for MixedLearning mode)
///
/// # Returns
/// * Vec of node_ids to include in the session
///
/// # FSRS Overlay Architecture
///
/// The session generation follows a layered approach:
/// 1. **Graph/Energy Brain**: Uses prerequisite gate, priority scoring based on
///    energy, foundational/influence scores, and urgency (days_overdue).
/// 2. **FSRS Overlay**: Due items (next_due_ts <= now_ts) get hard priority.
///    They are guaranteed to be in the session (up to session_size).
/// 3. **Band Composition**: Remaining slots filled by mastery band proportions.
///
/// This ensures FSRS-scheduled reviews are never silently dropped while still
/// allowing the graph-based scheduler to prioritize among non-due items.
pub fn generate_session(
    candidates: Vec<CandidateNode>,
    parent_map: HashMap<i64, Vec<i64>>,
    parent_energies: ParentEnergyMap,
    profile: &UserProfile,
    session_size: usize,
    now_ts: i64,
    mode: SessionMode,
    mix_config: Option<&SessionMixConfig>,
) -> Vec<i64> {
    if candidates.is_empty() || session_size == 0 {
        return Vec::new();
    }

    // Step 1: Convert to InMemNodes
    let mut nodes: Vec<InMemNode> = candidates
        .into_iter()
        .map(|candidate| {
            let parent_ids = parent_map.get(&candidate.id).cloned().unwrap_or_default();
            InMemNode::with_parents(candidate, parent_ids)
        })
        .collect();

    // Step 2: Apply Prerequisite Mastery Gate
    nodes.retain(|node| {
        let unsatisfied = count_unsatisfied_parents(&node.parent_ids, &parent_energies);
        unsatisfied == 0
    });

    if nodes.is_empty() {
        return Vec::new();
    }

    // Step 3: Calculate readiness, days_overdue, and priority score for each node
    let scored_nodes: Vec<(InMemNode, f32, f32, f64)> = nodes
        .into_iter()
        .map(|node| {
            let readiness = calculate_readiness(&node.parent_ids, &parent_energies);
            let days_overdue = calculate_days_overdue(node.data.next_due_ts, now_ts);
            let (score, _tie_breaker) =
                calculate_priority_score(&node, profile, readiness, days_overdue);
            (node, readiness, days_overdue, score)
        })
        .collect();

    // Step 4: Apply FSRS overlay - separate due from non-due items
    // Due items get hard priority in session assembly
    let mut due_nodes: Vec<(InMemNode, f32, f32, f64)> = Vec::new();
    let mut other_nodes: Vec<(InMemNode, f32, f32, f64)> = Vec::new();

    for item in scored_nodes {
        let is_due = item.0.data.next_due_ts > 0 && item.0.data.next_due_ts <= now_ts;
        if is_due {
            due_nodes.push(item);
        } else {
            other_nodes.push(item);
        }
    }

    // Sort each group by score DESC, quran_order ASC
    let sort_fn = |a: &(InMemNode, f32, f32, f64), b: &(InMemNode, f32, f32, f64)| {
        let score_cmp = b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal);
        if score_cmp == std::cmp::Ordering::Equal {
            a.0.data.quran_order.cmp(&b.0.data.quran_order)
        } else {
            score_cmp
        }
    };
    due_nodes.sort_by(sort_fn);
    other_nodes.sort_by(sort_fn);

    // Step 5: FSRS overlay - due items get hard priority
    // Strategy: Pre-allocate slots for due items, then fill remaining with band composition
    let due_ids: Vec<i64> = due_nodes.iter().map(|(n, _, _, _)| n.data.id).collect();
    let due_count_for_session = due_ids.len().min(session_size);

    // If we have more due items than session_size, use band composition on just due items
    // Otherwise, seed session with due items and compose remaining slots from other items
    let session = if due_nodes.len() >= session_size {
        // All slots go to due items - compose from due_nodes only
        let k = (session_size * 3).min(due_nodes.len());
        let top_due: Vec<InMemNode> = due_nodes
            .into_iter()
            .take(k)
            .map(|(n, _, _, _)| n)
            .collect();

        match mode {
            SessionMode::Revision => compose_revision_session(top_due, session_size),
            SessionMode::MixedLearning => {
                let default_config = SessionMixConfig::default();
                let config = mix_config.unwrap_or(&default_config);
                compose_mixed_learning_session(top_due, session_size, config)
            }
        }
    } else {
        // Pre-allocate due items, then fill remaining slots
        let remaining_slots = session_size.saturating_sub(due_count_for_session);

        // Take top other_nodes for remaining slots
        let k_other = (remaining_slots * 3).min(other_nodes.len());
        let other_for_composition: Vec<InMemNode> = other_nodes
            .into_iter()
            .take(k_other)
            .map(|(n, _, _, _)| n)
            .collect();

        // Compose the remaining slots from other_nodes
        let other_session = if remaining_slots > 0 && !other_for_composition.is_empty() {
            match mode {
                SessionMode::Revision => {
                    compose_revision_session(other_for_composition, remaining_slots)
                }
                SessionMode::MixedLearning => {
                    let default_config = SessionMixConfig::default();
                    let config = mix_config.unwrap_or(&default_config);
                    compose_mixed_learning_session(other_for_composition, remaining_slots, config)
                }
            }
        } else {
            Vec::new()
        };

        // Final session: due items first (hard priority), then composed other items
        let mut final_session: Vec<i64> = due_ids.into_iter().take(due_count_for_session).collect();
        final_session.extend(other_session);
        final_session.truncate(session_size);
        final_session
    };

    session
}

// ============================================================================
// REVISION MODE COMPOSITION
// ============================================================================

/// Composes a revision session using content difficulty buckets (60/30/10).
fn compose_revision_session(nodes: Vec<InMemNode>, session_size: usize) -> Vec<i64> {
    // Bucket by difficulty
    let mut easy = Vec::new();
    let mut medium = Vec::new();
    let mut hard = Vec::new();

    for node in nodes {
        match DifficultyBucket::from_score(node.data.difficulty_score) {
            DifficultyBucket::Easy => easy.push(node),
            DifficultyBucket::Medium => medium.push(node),
            DifficultyBucket::Hard => hard.push(node),
        }
    }

    // Calculate targets
    let target_easy = (session_size as f32 * 0.6).round() as usize;
    let target_medium = (session_size as f32 * 0.3).round() as usize;
    let target_hard = session_size.saturating_sub(target_easy + target_medium);

    // Collect session
    let mut session = Vec::new();

    // Take from each bucket up to target
    session.extend(easy.iter().take(target_easy).map(|n| n.data.id));
    session.extend(medium.iter().take(target_medium).map(|n| n.data.id));
    session.extend(hard.iter().take(target_hard).map(|n| n.data.id));

    // Fallback: if we didn't reach session_size, fill from remaining nodes
    if session.len() < session_size {
        let remaining_needed = session_size - session.len();

        // Collect remaining nodes (those not yet added)
        let added_ids: std::collections::HashSet<_> = session.iter().cloned().collect();
        let remaining: Vec<_> = easy
            .iter()
            .chain(medium.iter())
            .chain(hard.iter())
            .filter(|n| !added_ids.contains(&n.data.id))
            .collect();

        // Take up to remaining_needed
        session.extend(remaining.iter().take(remaining_needed).map(|n| n.data.id));
    }

    // Truncate if we somehow exceeded (shouldn't happen)
    session.truncate(session_size);
    session
}

// ============================================================================
// MIXED LEARNING MODE COMPOSITION
// ============================================================================

/// Composes a mixed learning session using configurable mastery bands.
fn compose_mixed_learning_session(
    nodes: Vec<InMemNode>,
    session_size: usize,
    config: &SessionMixConfig,
) -> Vec<i64> {
    // Bucket by mastery band
    let mut new = Vec::new();
    let mut really_struggling = Vec::new();
    let mut struggling = Vec::new();
    let mut almost_there = Vec::new();
    let mut almost_mastered = Vec::new();

    for node in nodes {
        match MasteryBand::from_energy(node.data.energy) {
            MasteryBand::New => new.push(node),
            MasteryBand::ReallyStruggling => really_struggling.push(node),
            MasteryBand::Struggling => struggling.push(node),
            MasteryBand::AlmostThere => almost_there.push(node),
            MasteryBand::AlmostMastered => almost_mastered.push(node),
        }
    }

    // Calculate targets using configurable percentages
    // For new items: use max of (percentage * session_size) and min_new_per_session
    // This guarantees coverage of unseen items even with small percentage
    let pct_new_slots = (session_size as f32 * config.pct_new).round() as usize;
    let available_new = new.len();
    let target_new = pct_new_slots
        .max(config.min_new_per_session)
        .min(available_new);

    let target_almost_mastered =
        (session_size as f32 * config.pct_almost_mastered).round() as usize;
    let target_almost_there = (session_size as f32 * config.pct_almost_there).round() as usize;
    let target_struggling = (session_size as f32 * config.pct_struggling).round() as usize;
    let target_really_struggling = session_size.saturating_sub(
        target_new + target_almost_mastered + target_almost_there + target_struggling,
    );

    // Collect session
    let mut session = Vec::new();

    session.extend(new.iter().take(target_new).map(|n| n.data.id));
    session.extend(
        almost_mastered
            .iter()
            .take(target_almost_mastered)
            .map(|n| n.data.id),
    );
    session.extend(
        almost_there
            .iter()
            .take(target_almost_there)
            .map(|n| n.data.id),
    );
    session.extend(struggling.iter().take(target_struggling).map(|n| n.data.id));
    session.extend(
        really_struggling
            .iter()
            .take(target_really_struggling)
            .map(|n| n.data.id),
    );

    // Fallback: if we didn't reach session_size, fill from remaining nodes
    if session.len() < session_size {
        let remaining_needed = session_size - session.len();

        let added_ids: std::collections::HashSet<_> = session.iter().cloned().collect();
        let remaining: Vec<_> = new
            .iter()
            .chain(almost_mastered.iter())
            .chain(almost_there.iter())
            .chain(struggling.iter())
            .chain(really_struggling.iter())
            .filter(|n| !added_ids.contains(&n.data.id))
            .collect();

        session.extend(remaining.iter().take(remaining_needed).map(|n| n.data.id));
    }

    session.truncate(session_size);
    session
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(
        id: i64,
        foundational: f32,
        influence: f32,
        difficulty: f32,
        energy: f32,
        due_ts: i64,
        quran_order: i64,
    ) -> CandidateNode {
        CandidateNode {
            id,
            foundational_score: foundational,
            influence_score: influence,
            difficulty_score: difficulty,
            energy,
            next_due_ts: due_ts,
            quran_order,
        }
    }

    #[test]
    fn test_generate_session_empty_candidates() {
        let session = generate_session(
            vec![],
            HashMap::new(),
            HashMap::new(),
            &UserProfile::balanced(),
            10,
            0,
            SessionMode::MixedLearning,
            None,
        );
        assert!(session.is_empty());
    }

    #[test]
    fn test_generate_session_prerequisite_gate() {
        // Node A (no parents, energy = 0.5)
        let node_a = make_candidate(1, 0.5, 0.3, 0.2, 0.5, 0, 1000);
        // Node B (no parents, energy = 0.6)
        let node_b = make_candidate(2, 0.5, 0.3, 0.2, 0.6, 0, 2000);
        // Node C (parents: A, B; energy = 0.0)
        let node_c = make_candidate(3, 0.5, 0.3, 0.3, 0.0, 0, 3000);
        // Node D (parent: C; energy = 0.0)
        let node_d = make_candidate(4, 0.5, 0.3, 0.4, 0.0, 0, 4000);

        let mut parent_map = HashMap::new();
        parent_map.insert(3, vec![1, 2]);
        parent_map.insert(4, vec![3]);

        let mut parent_energies = HashMap::new();
        parent_energies.insert(1, 0.5);
        parent_energies.insert(2, 0.6);
        parent_energies.insert(3, 0.0); // Below threshold

        let session = generate_session(
            vec![node_a, node_b, node_c, node_d],
            parent_map,
            parent_energies,
            &UserProfile::balanced(),
            10,
            0,
            SessionMode::MixedLearning,
            None,
        );

        // A and B have no parents -> eligible
        // C has parents A and B both >= 0.3 -> eligible
        // D has parent C with energy = 0.0 (< 0.3) -> NOT eligible
        assert!(session.contains(&1));
        assert!(session.contains(&2));
        assert!(session.contains(&3));
        assert!(!session.contains(&4));
    }

    #[test]
    fn test_revision_mode_difficulty_bucketing() {
        let candidates = vec![
            make_candidate(1, 0.5, 0.3, 0.1, 0.5, 0, 1000),
            make_candidate(2, 0.5, 0.3, 0.2, 0.5, 0, 2000),
            make_candidate(3, 0.5, 0.3, 0.5, 0.5, 0, 3000),
            make_candidate(4, 0.5, 0.3, 0.6, 0.5, 0, 4000),
            make_candidate(5, 0.5, 0.3, 0.8, 0.5, 0, 5000),
            make_candidate(6, 0.5, 0.3, 0.9, 0.5, 0, 6000),
        ];

        let session = generate_session(
            candidates,
            HashMap::new(),
            HashMap::new(),
            &UserProfile::balanced(),
            6,
            0,
            SessionMode::Revision,
            None,
        );

        assert_eq!(session.len(), 6);
        // Should have 60% easy (4), 30% medium (2), 10% hard (0-1)
        // But we only have 2 easy, 2 medium, 2 hard, so fallback applies
    }

    #[test]
    fn test_mixed_learning_mode_mastery_bucketing() {
        let candidates = vec![
            make_candidate(1, 0.5, 0.3, 0.2, 0.0, 0, 1000),
            make_candidate(2, 0.5, 0.3, 0.2, 0.15, 0, 2000),
            make_candidate(3, 0.5, 0.3, 0.2, 0.35, 0, 3000),
            make_candidate(4, 0.5, 0.3, 0.2, 0.55, 0, 4000),
            make_candidate(5, 0.5, 0.3, 0.2, 0.85, 0, 5000),
        ];

        let session = generate_session(
            candidates,
            HashMap::new(),
            HashMap::new(),
            &UserProfile::balanced(),
            5,
            0,
            SessionMode::MixedLearning,
            None,
        );

        assert_eq!(session.len(), 5);
        assert!(session.contains(&1));
        assert!(session.contains(&2));
        assert!(session.contains(&3));
        assert!(session.contains(&4));
        assert!(session.contains(&5));
    }

    // =========================================================================
    // FSRS OVERLAY TESTS
    // =========================================================================

    #[test]
    fn test_fsrs_overlay_due_items_always_included() {
        // Test that due items (next_due_ts <= now_ts) are always in the session
        // even if their raw priority scores are lower than non-due items.
        let now_ts = 1_000_000_000i64; // 1 billion ms

        let candidates = vec![
            // Two due items with low scores (low foundational)
            make_candidate(1, 0.2, 0.1, 0.3, 0.3, now_ts - 100, 1000), // due
            make_candidate(2, 0.2, 0.1, 0.3, 0.3, now_ts - 50, 2000),  // due
            // Three non-due items with high scores
            make_candidate(3, 0.9, 0.9, 0.3, 0.5, now_ts + 100_000, 3000), // not due
            make_candidate(4, 0.9, 0.9, 0.3, 0.5, now_ts + 200_000, 4000), // not due
            make_candidate(5, 0.9, 0.9, 0.3, 0.5, now_ts + 300_000, 5000), // not due
        ];

        let session = generate_session(
            candidates,
            HashMap::new(),
            HashMap::new(),
            &UserProfile::balanced(),
            3, // Only 3 slots
            now_ts,
            SessionMode::MixedLearning,
            None,
        );

        // Due items 1 and 2 MUST be in the session, regardless of their lower scores
        assert!(
            session.contains(&1),
            "Due item 1 should be in session, got: {:?}",
            session
        );
        assert!(
            session.contains(&2),
            "Due item 2 should be in session, got: {:?}",
            session
        );
        assert_eq!(session.len(), 3);
    }

    #[test]
    fn test_fsrs_overlay_many_due_items_caps_at_session_size() {
        // When due_nodes.len() > session_size, only top session_size due items used
        let now_ts = 1_000_000_000i64;

        let candidates = vec![
            make_candidate(1, 0.5, 0.5, 0.3, 0.3, now_ts - 100, 1000), // due
            make_candidate(2, 0.5, 0.5, 0.3, 0.3, now_ts - 50, 2000),  // due
            make_candidate(3, 0.5, 0.5, 0.3, 0.3, now_ts - 25, 3000),  // due
            make_candidate(4, 0.5, 0.5, 0.3, 0.3, now_ts - 10, 4000),  // due
            make_candidate(5, 0.9, 0.9, 0.3, 0.5, now_ts + 100_000, 5000), // not due
        ];

        let session = generate_session(
            candidates,
            HashMap::new(),
            HashMap::new(),
            &UserProfile::balanced(),
            3, // Only 3 slots, but 4 due items
            now_ts,
            SessionMode::MixedLearning,
            None,
        );

        // Session should be exactly 3 items, all from due items
        assert_eq!(session.len(), 3);

        // Count how many due items are in session
        let due_in_session = [1, 2, 3, 4]
            .iter()
            .filter(|id| session.contains(id))
            .count();
        assert_eq!(
            due_in_session, 3,
            "All 3 slots should be due items, got: {:?}",
            session
        );

        // Non-due item 5 should NOT be in session
        assert!(
            !session.contains(&5),
            "Non-due item should not be in session when we have enough due items"
        );
    }

    #[test]
    fn test_fsrs_overlay_preserves_band_composition_for_non_due() {
        // When there are few due items, remaining slots use band composition
        let now_ts = 1_000_000_000i64;

        let candidates = vec![
            // 1 due item
            make_candidate(1, 0.5, 0.5, 0.3, 0.3, now_ts - 100, 1000), // due
            // Non-due items across different mastery bands
            make_candidate(10, 0.5, 0.5, 0.3, 0.0, now_ts + 100_000, 10000), // new
            make_candidate(11, 0.5, 0.5, 0.3, 0.15, now_ts + 100_000, 11000), // really struggling
            make_candidate(12, 0.5, 0.5, 0.3, 0.35, now_ts + 100_000, 12000), // struggling
            make_candidate(13, 0.5, 0.5, 0.3, 0.55, now_ts + 100_000, 13000), // almost there
            make_candidate(14, 0.5, 0.5, 0.3, 0.85, now_ts + 100_000, 14000), // almost mastered
        ];

        let session = generate_session(
            candidates,
            HashMap::new(),
            HashMap::new(),
            &UserProfile::balanced(),
            5,
            now_ts,
            SessionMode::MixedLearning,
            None,
        );

        // Due item 1 must be included
        assert!(
            session.contains(&1),
            "Due item must be in session, got: {:?}",
            session
        );
        assert_eq!(session.len(), 5);

        // Remaining 4 slots should come from the band composition
        // We don't test exact composition (that's tested elsewhere), just that
        // the session has non-due items filling remaining slots
        let non_due_count = session.iter().filter(|&&id| id != 1).count();
        assert_eq!(non_due_count, 4, "Should have 4 non-due items");
    }
}
