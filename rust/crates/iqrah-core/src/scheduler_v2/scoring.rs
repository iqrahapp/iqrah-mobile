/// Scoring functions for Scheduler v2.0
///
/// This module implements the priority scoring algorithm used to rank candidate nodes
/// for session generation, based on urgency, readiness, foundation, and influence.
use crate::scheduler_v2::types::{InMemNode, ParentEnergyMap, UserProfile, MASTERY_THRESHOLD};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Calculates days overdue for a node.
///
/// # Arguments
/// * `next_due_ts` - Next due timestamp in MILLISECONDS (epoch)
/// * `now_ts` - Current timestamp in MILLISECONDS (epoch)
///
/// # Returns
/// * Number of days overdue (>= 0.0). Returns 0.0 if not yet due.
pub fn calculate_days_overdue(next_due_ts: i64, now_ts: i64) -> f32 {
    if next_due_ts < now_ts {
        let overdue_ms = (now_ts - next_due_ts) as f64;
        let overdue_days = overdue_ms / (86400.0 * 1000.0); // Convert ms to days
        overdue_days.floor() as f32
    } else {
        0.0
    }
}

/// Calculates readiness for a node based on parent energies.
///
/// Readiness represents how well-prepared the user is to learn this node,
/// based on their mastery of prerequisite concepts.
///
/// # Arguments
/// * `parent_ids` - IDs of prerequisite parent nodes
/// * `parent_energies` - Map of node_id -> energy
///
/// # Returns
/// * Readiness score (0.0-1.0):
///   - 1.0 if no parents (foundational concept)
///   - Mean of parent energies otherwise
pub fn calculate_readiness(parent_ids: &[i64], parent_energies: &ParentEnergyMap) -> f32 {
    if parent_ids.is_empty() {
        return 1.0; // No prerequisites = fully ready
    }

    let sum: f32 = parent_ids
        .iter()
        .map(|id| parent_energies.get(id).copied().unwrap_or(0.0))
        .sum();

    let count = parent_ids.len() as f32;
    sum / count
}

/// Counts how many parents have energy below the mastery threshold.
///
/// This is used for the Prerequisite Mastery Gate to determine if a node
/// is eligible for scheduling.
///
/// # Arguments
/// * `parent_ids` - IDs of prerequisite parent nodes
/// * `parent_energies` - Map of node_id -> energy
///
/// # Returns
/// * Number of unsatisfied parents (energy < MASTERY_THRESHOLD)
pub fn count_unsatisfied_parents(parent_ids: &[i64], parent_energies: &ParentEnergyMap) -> usize {
    parent_ids
        .iter()
        .filter(|id| {
            let energy = parent_energies.get(id).copied().unwrap_or(0.0);
            energy < MASTERY_THRESHOLD
        })
        .count()
}

// ============================================================================
// ISS v2.5: SUCCESS PROBABILITY WEIGHTING
// ============================================================================

/// Estimate success probability for an item based on energy and maturity.
///
/// This is a heuristic approximation used for scheduling decisions to prevent
/// "triage failure" - where the scheduler concentrates capacity on failing items.
///
/// # Factors
/// - Energy (E): Higher energy → higher success probability
/// - Review count (maturity): More reviews → more stable success
///
/// # Returns
/// Probability in [0.05, 0.95]
pub fn estimate_success_probability(energy: f32, review_count: u32) -> f32 {
    // Base success probability from energy
    // Calibration: E=0.10 → 10%, E=0.50 → 60%, E=0.90 → 90%
    let energy_contrib = energy.clamp(0.0, 1.0);

    // Maturity factor: young items are less stable
    // review_count: 0-5 → maturity: 0.0-1.0
    let maturity = (review_count as f32 / 5.0).clamp(0.0, 1.0);

    // Young items: more pessimistic (multiply by 0.7)
    // Mature items: use energy directly (multiply by 1.0)
    let maturity_adjustment = 0.7 + 0.3 * maturity;

    // Final success probability
    let success_prob = energy_contrib * maturity_adjustment;

    // Clamp to reasonable range
    success_prob.clamp(0.05, 0.95)
}

/// Compute weighted urgency score to prevent triage failure.
///
/// Base urgency is dampened by success probability to balance between
/// "urgent but failing" and "less urgent but viable" items.
///
/// # Formula
/// ```text
/// weighted = base_urgency × (α + (1-α) × success_prob)
/// ```
///
/// where α = 0.3 (minimum weight for very low success items)
///
/// # Effect
/// - Item with 10% success: urgency × 0.37 (dampened)
/// - Item with 50% success: urgency × 0.65 (moderately dampened)
/// - Item with 90% success: urgency × 0.93 (mostly preserved)
pub fn compute_weighted_urgency(
    base_urgency: f32,
    energy: f32,
    review_count: u32,
    enable_success_weighting: bool,
) -> f32 {
    if !enable_success_weighting {
        return base_urgency;
    }

    // Estimate success probability
    let success_prob = estimate_success_probability(energy, review_count);

    // Weight urgency by success probability
    // α = minimum weight for very low success items
    // This ensures they're not completely ignored, but deprioritized
    // ISS v2.5 tuning: reduced from 0.3 to 0.1 for more aggressive dampening
    let alpha = 0.1;
    let weight = alpha + (1.0 - alpha) * success_prob;

    base_urgency * weight
}

// ============================================================================
// PRIORITY SCORING
// ============================================================================

/// Calculates the priority score for a node.
///
/// The score combines urgency (time-based) with learning potential
/// (readiness, foundation, influence).
///
/// # Formula
/// ```text
/// urgency_factor = 1.0 + w_urgency * ln(1.0 + days_overdue)
///
/// learning_potential = w_readiness * readiness
///                    + w_foundation * foundational_score
///                    + w_influence * influence_score
///
/// final_score = urgency_factor * learning_potential
/// ```
///
/// # Arguments
/// * `node` - The in-memory node to score
/// * `profile` - User's learning profile (weights)
/// * `readiness` - Readiness score (pre-calculated)
/// * `days_overdue` - Days overdue (pre-calculated)
///
/// # Returns
/// * Tuple of (priority_score, tie_breaker):
///   - `priority_score`: Higher is better (sort descending)
///   - `tie_breaker`: Negative quran_order (for ascending Qur'an order on ties)
pub fn calculate_priority_score(
    node: &InMemNode,
    profile: &UserProfile,
    readiness: f32,
    days_overdue: f32,
) -> (f64, i64) {
    // Urgency factor: logarithmic growth with days overdue
    let urgency_factor = 1.0 + (profile.w_urgency * (1.0 + days_overdue.max(0.0)).ln());

    // Fairness term (spec §6.4, equations 206-217)
    // Pressures scheduler to cover underserved items and maintain balanced exposure
    const TARGET_REVIEWS: u32 = 7; // Items need ~7 reviews for stable long-term memory
    const TARGET_RECALL: f32 = 0.7;

    let review_deficit = (TARGET_REVIEWS as i32 - node.data.review_count as i32).max(0) as f32;
    let recall_deficit = (TARGET_RECALL - node.data.predicted_recall).max(0.0);

    // Additive fairness component (for fine-grained ranking among similar items)
    let fairness_additive = profile.w_fairness * (review_deficit + recall_deficit);

    // Coverage factor (spec §6.6, invariant S3)
    // Provides multiplicative boost to under-reviewed items
    //
    // CONSTRAINTS (per spec):
    // - Bounded: 1.0 ≤ coverage_factor ≤ 1.0 + C_MAX
    // - Monotone decreasing: as review_count increases, coverage_factor decreases
    // - FSRS-independent: depends ONLY on review_count, NOT on due-ness/stability
    const C_MAX: f32 = 9.0; // Maximum boost (10x at zero reviews)

    let coverage_factor = if node.data.review_count < TARGET_REVIEWS {
        1.0 + (C_MAX * (1.0 - node.data.review_count as f32 / TARGET_REVIEWS as f32))
    } else {
        1.0
    };

    // Learning potential: weighted sum of readiness, foundation, influence, and fairness
    let learning_potential = profile.w_readiness * readiness
        + profile.w_foundation * node.data.foundational_score
        + profile.w_influence * node.data.influence_score
        + fairness_additive;

    // Base score (spec §6.7): applies urgency, coverage, and learning potential
    let base_score = urgency_factor * coverage_factor * learning_potential;

    // ISS v2.5: Apply success probability weighting to prevent triage failure
    // This dampens urgency for items with low success probability, balancing
    // between "urgent but failing" and "less urgent but viable" items.
    let final_score = compute_weighted_urgency(
        base_score,
        node.data.energy,
        node.data.review_count,
        true, // Enable success weighting
    );

    // Return (score, negative_quran_order) for sorting
    // Higher score first, then earlier Qur'an order
    (final_score as f64, -node.data.quran_order)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler_v2::types::CandidateNode;
    use std::collections::HashMap;

    #[test]
    fn test_calculate_days_overdue_zero_when_not_due() {
        let now_ts = 1700000000000; // Some timestamp
        let next_due_ts = now_ts + 86400000; // Due tomorrow

        let days_overdue = calculate_days_overdue(next_due_ts, now_ts);
        assert_eq!(days_overdue, 0.0);
    }

    #[test]
    fn test_calculate_days_overdue_correct_value() {
        let now_ts = 1700000000000i64;
        let next_due_ts = now_ts - (3 * 86400000); // 3 days ago

        let days_overdue = calculate_days_overdue(next_due_ts, now_ts);
        assert_eq!(days_overdue, 3.0);
    }

    #[test]
    fn test_calculate_readiness_no_parents() {
        let parent_ids: Vec<i64> = vec![];
        let parent_energies = HashMap::new();

        let readiness = calculate_readiness(&parent_ids, &parent_energies);
        assert_eq!(readiness, 1.0);
    }

    #[test]
    fn test_calculate_readiness_with_parents() {
        let parent_ids = vec![1, 2, 3];
        let mut parent_energies = HashMap::new();
        parent_energies.insert(1, 0.6);
        parent_energies.insert(2, 0.9);
        parent_energies.insert(3, 0.3);

        let readiness = calculate_readiness(&parent_ids, &parent_energies);
        let expected = (0.6 + 0.9 + 0.3) / 3.0;
        assert!((readiness - expected).abs() < 0.001);
    }

    #[test]
    fn test_calculate_readiness_missing_parent_treated_as_zero() {
        let parent_ids = vec![1, 2];
        let mut parent_energies = HashMap::new();
        parent_energies.insert(1, 0.6);
        // 2 is missing

        let readiness = calculate_readiness(&parent_ids, &parent_energies);
        let expected = (0.6 + 0.0) / 2.0;
        assert!((readiness - expected).abs() < 0.001);
    }

    #[test]
    fn test_count_unsatisfied_parents_all_satisfied() {
        let parent_ids = vec![1, 2];
        let mut parent_energies = HashMap::new();
        parent_energies.insert(1, 0.5);
        parent_energies.insert(2, 0.8);

        let count = count_unsatisfied_parents(&parent_ids, &parent_energies);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_unsatisfied_parents_some_unsatisfied() {
        let parent_ids = vec![1, 2, 3];
        let mut parent_energies = HashMap::new();
        parent_energies.insert(1, 0.5); // Satisfied (>= 0.3)
        parent_energies.insert(2, 0.2); // Unsatisfied (< 0.3)
        parent_energies.insert(3, 0.1); // Unsatisfied (< 0.3)

        let count = count_unsatisfied_parents(&parent_ids, &parent_energies);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_calculate_priority_score_no_urgency() {
        let candidate = CandidateNode {
            id: 1,
            foundational_score: 0.5,
            influence_score: 0.3,
            difficulty_score: 0.2,
            energy: 0.0,
            next_due_ts: 0,
            quran_order: 1001000,
            review_count: 0,
            predicted_recall: 0.0,
        };
        let node = InMemNode::new(candidate);
        let profile = UserProfile::balanced();
        let readiness = 1.0;
        let days_overdue = 0.0;

        let (score, tie_breaker) =
            calculate_priority_score(&node, &profile, readiness, days_overdue);

        // ISS v2.5: Updated formula with success probability weighting
        // base_score = 41.1 (from urgency × coverage × learning_potential)
        // success_prob = 0.05 (minimum for energy=0.0, review_count=0)
        // weight = 0.1 + 0.9 × 0.05 = 0.145
        // final_score = 41.1 × 0.145 ≈ 6.0
        assert!(
            score > 4.0 && score < 10.0,
            "Expected ~6.0 (dampened by low energy), got {}",
            score
        );
        assert_eq!(tie_breaker, -1001000);
    }

    #[test]
    fn test_calculate_priority_score_with_urgency() {
        let candidate = CandidateNode {
            id: 1,
            foundational_score: 0.5,
            influence_score: 0.3,
            difficulty_score: 0.2,
            energy: 0.5,
            next_due_ts: 0,
            quran_order: 2001000,
            review_count: 2,
            predicted_recall: 0.8,
        };
        let node = InMemNode::new(candidate);
        let profile = UserProfile::balanced();
        let readiness = 0.8;
        let days_overdue = 5.0;

        let (score, tie_breaker) =
            calculate_priority_score(&node, &profile, readiness, days_overdue);

        // ISS v2.5: Updated formula with success probability weighting
        // base_score ≈ 64.3 (from urgency × coverage × learning_potential)
        // success_prob = 0.5 × (0.7 + 0.3 × 2/5) = 0.41 (E=0.5, count=2)
        // weight = 0.1 + 0.9 × 0.41 = 0.469
        // final_score = 64.3 × 0.469 ≈ 30.2
        assert!(
            score > 25.0 && score < 40.0,
            "Expected ~30.2 (dampened by moderate energy), got {}",
            score
        );
        assert_eq!(tie_breaker, -2001000);
    }

    #[test]
    fn test_priority_score_monotonicity_with_urgency() {
        let candidate = CandidateNode {
            id: 1,
            foundational_score: 0.5,
            influence_score: 0.3,
            difficulty_score: 0.2,
            energy: 0.5,
            next_due_ts: 0,
            quran_order: 1001000,
            review_count: 3,
            predicted_recall: 0.7,
        };
        let node = InMemNode::new(candidate);
        let profile = UserProfile::balanced();
        let readiness = 0.8;

        let (score1, _) = calculate_priority_score(&node, &profile, readiness, 0.0);
        let (score2, _) = calculate_priority_score(&node, &profile, readiness, 5.0);
        let (score3, _) = calculate_priority_score(&node, &profile, readiness, 10.0);

        // More days overdue should increase score
        assert!(score2 > score1);
        assert!(score3 > score2);
    }

    // ========================================================================
    // ISS v2.5: Success Probability Weighting Tests
    // ========================================================================

    #[test]
    fn test_success_probability_estimation() {
        // Young item, low energy: very low success
        let prob = estimate_success_probability(0.10, 1);
        assert!(prob < 0.15, "Expected < 0.15, got {}", prob);

        // Young item, medium energy: moderate success
        let prob = estimate_success_probability(0.50, 2);
        assert!(
            prob > 0.30 && prob < 0.50,
            "Expected 0.30-0.50, got {}",
            prob
        );

        // Mature item, medium energy: good success
        let prob = estimate_success_probability(0.50, 6);
        assert!(
            prob > 0.45 && prob < 0.60,
            "Expected 0.45-0.60, got {}",
            prob
        );

        // Mature item, high energy: very high success
        let prob = estimate_success_probability(0.85, 8);
        assert!(prob > 0.80, "Expected > 0.80, got {}", prob);
    }

    #[test]
    fn test_success_probability_boundaries() {
        // Very low energy, young item: minimum ~5%
        let prob = estimate_success_probability(0.05, 0);
        assert!(prob >= 0.05 && prob <= 0.10);

        // Very high energy, mature item: maximum ~95%
        let prob = estimate_success_probability(0.95, 10);
        assert!(prob >= 0.90 && prob <= 0.95);
    }

    #[test]
    fn test_weighted_urgency_dampens_failures() {
        // Scenario: Two items, both with high base urgency
        // Item A: E=0.08 (failing), review_count=3
        // Item B: E=0.45 (viable), review_count=3

        let urgency_a = 10.0;
        let urgency_b = 7.0; // Lower base urgency

        let weighted_a = compute_weighted_urgency(urgency_a, 0.08, 3, true);
        let weighted_b = compute_weighted_urgency(urgency_b, 0.45, 3, true);

        // After weighting, item B should rank higher despite lower base urgency
        assert!(
            weighted_b > weighted_a,
            "Viable item should outrank failing item: B={} vs A={}",
            weighted_b,
            weighted_a
        );
    }

    #[test]
    fn test_weighted_urgency_disabled() {
        let base = 10.0;
        let weighted = compute_weighted_urgency(base, 0.10, 2, false);
        assert_eq!(
            weighted, base,
            "Disabled weighting should return base urgency"
        );
    }

    #[test]
    fn test_maturity_affects_probability() {
        let energy = 0.50;

        let young = estimate_success_probability(energy, 1);
        let mature = estimate_success_probability(energy, 8);

        // Mature items should have higher success probability
        assert!(
            mature > young + 0.05,
            "Mature should exceed young: {} vs {}",
            mature,
            young
        );
    }
}
