use crate::scheduler_v2::events::{
    BucketAllocation, NullEventSink, SchedulerEvent, SchedulerEventSink, ScoreBreakdown,
    SessionModeEvent,
};
/// Session generation orchestrator for Scheduler v2.0
///
/// This module implements the main session generation logic, including:
/// - Prerequisite Mastery Gate
/// - Priority scoring and ranking
/// - Difficulty-based composition with fallback
use crate::scheduler_v2::{
    calculate_days_overdue, calculate_readiness, CandidateNode, InMemNode, ParentEnergyMap,
    SessionMixConfig, UserProfile,
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
/// * `event_sink` - Optional event sink for observability (spec §9)
///
/// # Returns
/// * Vec of node_ids to include in the session
///
/// # Pipeline Architecture
///
/// Single unified pipeline where FSRS influences **urgency scoring**, not session composition:
/// 1. **Prerequisite Gate**: Filters nodes with unsatisfied prerequisites (energy < 0.3).
/// 2. **Priority Scoring**: Combines graph (foundational, influence), readiness,
///    and urgency (days_overdue from FSRS). Due items get higher scores naturally.
/// 3. **Sorting**: All candidates ranked by score DESC, quran_order ASC.
/// 4. **Band Composition**: Top K candidates composed by SessionMode (mastery bands or difficulty).
///
/// There is NO separate "due vs non-due" overlay. FSRS due status affects urgency in scoring.
pub fn generate_session(
    candidates: Vec<CandidateNode>,
    parent_map: HashMap<i64, Vec<i64>>,
    parent_energies: ParentEnergyMap,
    profile: &UserProfile,
    session_size: usize,
    now_ts: i64,
    mode: SessionMode,
    mix_config: Option<&SessionMixConfig>,
    event_sink: Option<&dyn SchedulerEventSink>,
) -> Vec<i64> {
    // Use NullEventSink as default when none provided (zero overhead)
    let null_sink = NullEventSink;
    let sink: &dyn SchedulerEventSink = event_sink.unwrap_or(&null_sink);

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

    tracing::info!("Before prereq gate: {} nodes", nodes.len());

    // Step 2: Apply Prerequisite Mastery Gate with event emission
    let nodes_before_gate = nodes.len();
    nodes.retain(|node| {
        let unsatisfied_parents: Vec<i64> =
            get_unsatisfied_parent_ids(&node.parent_ids, &parent_energies);
        let passes_gate = unsatisfied_parents.is_empty();
        if !passes_gate {
            sink.emit(SchedulerEvent::PrerequisiteGateFailed {
                node_id: node.data.id,
                unsatisfied_parents,
            });
        }
        passes_gate
    });
    let gate_filtered = nodes_before_gate - nodes.len();

    tracing::info!(
        "After prereq gate: {} nodes ({} filtered)",
        nodes.len(),
        gate_filtered
    );

    if nodes.is_empty() {
        return Vec::new();
    }

    // Step 3: Calculate readiness, days_overdue, and priority score for each node
    // FSRS influences urgency here via days_overdue in the scoring function
    let mut scored_nodes: Vec<(InMemNode, f32, f32, f64)> = nodes
        .into_iter()
        .map(|node| {
            let readiness = calculate_readiness(&node.parent_ids, &parent_energies);
            let days_overdue = calculate_days_overdue(node.data.next_due_ts, now_ts);
            let (score, _tie_breaker, breakdown) =
                calculate_priority_score_with_breakdown(&node, profile, readiness, days_overdue);

            // Emit PriorityComputed event
            sink.emit(SchedulerEvent::PriorityComputed {
                node_id: node.data.id,
                components: breakdown,
            });

            (node, readiness, days_overdue, score)
        })
        .collect();

    // Step 4: Sort ALL scored nodes by priority (score DESC, quran_order ASC)
    // No due/non-due split - single unified ranking
    scored_nodes.sort_by(|a, b| {
        let score_cmp = b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal);
        if score_cmp == std::cmp::Ordering::Equal {
            a.0.data.quran_order.cmp(&b.0.data.quran_order)
        } else {
            score_cmp
        }
    });

    // Step 5: Take top K candidates for composition
    // K must be large enough to include min_new fresh items, since composition
    // needs those to honor min_new_per_session. For small pools use 3x, for large use 10x.
    let base_k = if scored_nodes.len() > session_size * 5 {
        session_size * 10 // Large pools: wider net to capture new items
    } else {
        session_size * 3 // Small pools: tighter selection
    };
    let k = base_k.min(scored_nodes.len());

    tracing::info!(
        "Top-K selection: k={}, total_scored={}",
        k,
        scored_nodes.len()
    );

    let top_nodes: Vec<InMemNode> = scored_nodes
        .into_iter()
        .take(k)
        .map(|(n, _, _, _)| n)
        .collect();

    // Step 6: Apply mode-specific composition with event emission
    match mode {
        SessionMode::Revision => {
            let (session, buckets) = compose_revision_session_with_buckets(top_nodes, session_size);
            sink.emit(SchedulerEvent::SessionComposed {
                mode: SessionModeEvent::Revision,
                buckets,
            });
            session
        }
        SessionMode::MixedLearning => {
            let default_config = SessionMixConfig::default();
            let config = mix_config.unwrap_or(&default_config);
            let (session, buckets) =
                compose_mixed_learning_session_with_buckets(top_nodes, session_size, config);
            sink.emit(SchedulerEvent::SessionComposed {
                mode: SessionModeEvent::MixedLearning,
                buckets,
            });
            session
        }
    }
}

/// Helper to get unsatisfied parent IDs for event emission
fn get_unsatisfied_parent_ids(parent_ids: &[i64], parent_energies: &ParentEnergyMap) -> Vec<i64> {
    use crate::scheduler_v2::types::MASTERY_THRESHOLD;
    parent_ids
        .iter()
        .filter(|id| {
            let energy = parent_energies.get(id).copied().unwrap_or(0.0);
            energy < MASTERY_THRESHOLD
        })
        .copied()
        .collect()
}

/// Calculate priority score with full breakdown for event emission
fn calculate_priority_score_with_breakdown(
    node: &InMemNode,
    profile: &UserProfile,
    readiness: f32,
    days_overdue: f32,
) -> (f64, i64, ScoreBreakdown) {
    // Replicate scoring logic to capture all components
    const TARGET_REVIEWS: u32 = 7;
    const TARGET_RECALL: f32 = 0.7;
    const C_MAX: f32 = 9.0;

    let urgency_factor = 1.0 + (profile.w_urgency * (1.0 + days_overdue.max(0.0)).ln());

    let review_deficit = (TARGET_REVIEWS as i32 - node.data.review_count as i32).max(0) as f32;
    let recall_deficit = (TARGET_RECALL - node.data.predicted_recall).max(0.0);
    let fairness_additive = profile.w_fairness * (review_deficit + recall_deficit);

    let coverage_factor = if node.data.review_count < TARGET_REVIEWS {
        1.0 + (C_MAX * (1.0 - node.data.review_count as f32 / TARGET_REVIEWS as f32))
    } else {
        1.0
    };

    let learning_potential = profile.w_readiness * readiness
        + profile.w_foundation * node.data.foundational_score
        + profile.w_influence * node.data.influence_score
        + fairness_additive;

    let final_score = urgency_factor * coverage_factor * learning_potential;

    let breakdown = ScoreBreakdown::new(
        urgency_factor,
        coverage_factor,
        readiness,
        node.data.foundational_score,
        node.data.influence_score,
        fairness_additive,
        final_score as f64,
    );

    (final_score as f64, -node.data.quran_order, breakdown)
}

// ============================================================================
// REVISION MODE COMPOSITION
// ============================================================================

/// Composes a revision session using content difficulty buckets (60/30/10).
#[allow(dead_code)] // Kept for backward compatibility, main code uses _with_buckets version
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

/// Composes a revision session and returns bucket allocation for event emission.
fn compose_revision_session_with_buckets(
    nodes: Vec<InMemNode>,
    session_size: usize,
) -> (Vec<i64>, BucketAllocation) {
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

    // Track actual counts for bucket allocation
    let actual_easy = easy.len().min(target_easy);
    let actual_medium = medium.len().min(target_medium);
    let actual_hard = hard.len().min(target_hard);

    // Take from each bucket up to target
    session.extend(easy.iter().take(target_easy).map(|n| n.data.id));
    session.extend(medium.iter().take(target_medium).map(|n| n.data.id));
    session.extend(hard.iter().take(target_hard).map(|n| n.data.id));

    // Fallback: if we didn't reach session_size, fill from remaining nodes
    if session.len() < session_size {
        let remaining_needed = session_size - session.len();

        let added_ids: std::collections::HashSet<_> = session.iter().cloned().collect();
        let remaining: Vec<_> = easy
            .iter()
            .chain(medium.iter())
            .chain(hard.iter())
            .filter(|n| !added_ids.contains(&n.data.id))
            .collect();

        session.extend(remaining.iter().take(remaining_needed).map(|n| n.data.id));
    }

    session.truncate(session_size);

    // For revision mode, map difficulty buckets to allocation struct
    // (easy -> almost_mastered, medium -> almost_there, hard -> struggling)
    let buckets = BucketAllocation::revision(actual_easy, actual_medium, actual_hard);

    (session, buckets)
}

// ============================================================================
// MIXED LEARNING MODE COMPOSITION
// ============================================================================

/// Composes a mixed learning session using configurable mastery bands.
#[allow(dead_code)] // Kept for backward compatibility, main code uses _with_buckets version
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
            MasteryBand::AlmostMastered => almost_mastered.push(node),
            MasteryBand::Struggling => struggling.push(node),
            MasteryBand::ReallyStruggling => really_struggling.push(node),
            MasteryBand::AlmostThere => almost_there.push(node),
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

    tracing::info!(
        "Composition: new={}/{}, am={}, at={}, str={}, rs={} (size={}, min_new={})",
        target_new,
        available_new,
        target_almost_mastered,
        target_almost_there,
        target_struggling,
        target_really_struggling,
        session_size,
        config.min_new_per_session
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

    // Debug: Count actual new items in returned session
    let actual_new_count = session
        .iter()
        .filter(|&id| new.iter().any(|n| n.data.id == *id))
        .count();
    tracing::info!(
        "Session returned: {} items ({} actually new from new bucket)",
        session.len(),
        actual_new_count
    );

    session
}

/// Composes a mixed learning session and returns bucket allocation for event emission.
fn compose_mixed_learning_session_with_buckets(
    nodes: Vec<InMemNode>,
    session_size: usize,
    config: &SessionMixConfig,
) -> (Vec<i64>, BucketAllocation) {
    // Bucket by mastery band
    let mut new = Vec::new();
    let mut really_struggling = Vec::new();
    let mut struggling = Vec::new();
    let mut almost_there = Vec::new();
    let mut almost_mastered = Vec::new();

    for node in nodes {
        match MasteryBand::from_energy(node.data.energy) {
            MasteryBand::New => new.push(node),
            MasteryBand::AlmostMastered => almost_mastered.push(node),
            MasteryBand::Struggling => struggling.push(node),
            MasteryBand::ReallyStruggling => really_struggling.push(node),
            MasteryBand::AlmostThere => almost_there.push(node),
        }
    }

    // Calculate targets using configurable percentages
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

    // Track actual counts
    let actual_new = new.len().min(target_new);
    let actual_almost_mastered = almost_mastered.len().min(target_almost_mastered);
    let actual_almost_there = almost_there.len().min(target_almost_there);
    let actual_struggling = struggling.len().min(target_struggling);
    let actual_really_struggling = really_struggling.len().min(target_really_struggling);

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

    let buckets = BucketAllocation::mixed_learning(
        actual_new,
        actual_almost_mastered,
        actual_almost_there,
        actual_struggling,
        actual_really_struggling,
    );

    (session, buckets)
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
            review_count: if energy > 0.0 { 1 } else { 0 }, // Simple heuristic
            predicted_recall: energy,                       // Simple approximation
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
            None, // event_sink
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
            None, // event_sink
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
            None, // event_sink
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
            None, // event_sink
        );

        assert_eq!(session.len(), 5);
        assert!(session.contains(&1));
        assert!(session.contains(&2));
        assert!(session.contains(&3));
        assert!(session.contains(&4));
        assert!(session.contains(&5));
    }

    // =========================================================================
    // URGENCY SCORING TESTS (unified pipeline - no overlay)
    // =========================================================================

    #[test]
    fn test_urgency_increases_score_for_overdue_items() {
        // Two identical items except one is overdue - overdue should get higher score
        // and thus appear first when sorted by score
        use crate::scheduler_v2::calculate_priority_score;

        let now_ts = 1_000_000_000i64; // 1 billion ms

        // Item A: 5 days overdue
        let candidate_a = CandidateNode {
            id: 1,
            foundational_score: 0.5,
            influence_score: 0.5,
            difficulty_score: 0.3,
            energy: 0.3,
            next_due_ts: now_ts - (5 * 86400 * 1000), // 5 days ago
            quran_order: 1000,
            review_count: 2,
            predicted_recall: 0.6,
        };
        let node_a = InMemNode::new(candidate_a);

        // Item B: not due (due tomorrow)
        let candidate_b = CandidateNode {
            id: 2,
            foundational_score: 0.5,
            influence_score: 0.5,
            difficulty_score: 0.3,
            energy: 0.3,
            next_due_ts: now_ts + (86400 * 1000), // tomorrow
            quran_order: 2000,
            review_count: 2,
            predicted_recall: 0.7,
        };
        let node_b = InMemNode::new(candidate_b);

        let profile = UserProfile::balanced();
        let readiness = 1.0;

        // Calculate days_overdue for each
        let days_overdue_a =
            crate::scheduler_v2::calculate_days_overdue(node_a.data.next_due_ts, now_ts);
        let days_overdue_b =
            crate::scheduler_v2::calculate_days_overdue(node_b.data.next_due_ts, now_ts);

        // Overdue item should have positive days_overdue
        assert!(
            days_overdue_a > 0.0,
            "Overdue item should have days_overdue > 0"
        );
        assert_eq!(
            days_overdue_b, 0.0,
            "Not-due item should have days_overdue = 0"
        );

        // Calculate scores
        let (score_a, _) = calculate_priority_score(&node_a, &profile, readiness, days_overdue_a);
        let (score_b, _) = calculate_priority_score(&node_b, &profile, readiness, days_overdue_b);

        // Overdue item should have higher score due to urgency
        assert!(
            score_a > score_b,
            "Overdue item should have higher priority score: {} vs {}",
            score_a,
            score_b
        );
    }

    #[test]
    fn test_unified_pipeline_ranks_by_combined_score() {
        // Test that the unified pipeline ranks items by combined score
        // (not by due/non-due status separately)
        let now_ts = 1_000_000_000i64;

        let candidates = vec![
            // Item 1: slightly overdue, low graph scores
            make_candidate(1, 0.2, 0.1, 0.3, 0.3, now_ts - (1 * 86400 * 1000), 1000),
            // Item 2: not due, high graph scores
            make_candidate(2, 0.9, 0.9, 0.3, 0.5, now_ts + 100_000, 2000),
            // Item 3: very overdue (10 days), medium graph scores
            make_candidate(3, 0.5, 0.5, 0.3, 0.3, now_ts - (10 * 86400 * 1000), 3000),
        ];

        let session = generate_session(
            candidates,
            HashMap::new(),
            HashMap::new(),
            &UserProfile::balanced(),
            3,
            now_ts,
            SessionMode::MixedLearning,
            None,
            None, // event_sink
        );

        // All items should be in session (only 3 candidates, 3 slots)
        assert_eq!(session.len(), 3);
        assert!(session.contains(&1));
        assert!(session.contains(&2));
        assert!(session.contains(&3));

        // Note: We're NOT guaranteeing due items always come first.
        // The ranking is by combined score (graph + urgency + readiness).
    }

    #[test]
    fn test_high_foundational_score_boosts_priority() {
        // Two items with same graph structure and urgency, different foundational scores
        use crate::scheduler_v2::calculate_priority_score;

        let candidate_high = CandidateNode {
            id: 1,
            foundational_score: 0.9,
            influence_score: 0.5,
            difficulty_score: 0.3,
            energy: 0.3,
            next_due_ts: 0, // new
            quran_order: 1000,
            review_count: 1,
            predicted_recall: 0.5,
        };

        let candidate_low = CandidateNode {
            id: 2,
            foundational_score: 0.2,
            influence_score: 0.5,
            difficulty_score: 0.3,
            energy: 0.3,
            next_due_ts: 0, // new
            quran_order: 2000,
            review_count: 1,
            predicted_recall: 0.5,
        };

        let profile = UserProfile::balanced();
        let readiness = 1.0;
        let days_overdue = 0.0;

        let (score_high, _) = calculate_priority_score(
            &InMemNode::new(candidate_high),
            &profile,
            readiness,
            days_overdue,
        );
        let (score_low, _) = calculate_priority_score(
            &InMemNode::new(candidate_low),
            &profile,
            readiness,
            days_overdue,
        );

        assert!(
            score_high > score_low,
            "Higher foundational score should yield higher priority: {} vs {}",
            score_high,
            score_low
        );
    }
}
