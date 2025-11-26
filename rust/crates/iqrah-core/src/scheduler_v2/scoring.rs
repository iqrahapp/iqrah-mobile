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
pub fn count_unsatisfied_parents(
    parent_ids: &[i64],
    parent_energies: &ParentEnergyMap,
) -> usize {
    parent_ids
        .iter()
        .filter(|id| {
            let energy = parent_energies.get(id).copied().unwrap_or(0.0);
            energy < MASTERY_THRESHOLD
        })
        .count()
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

    // Learning potential: weighted sum of readiness, foundation, and influence
    let learning_potential = profile.w_readiness * readiness
        + profile.w_foundation * node.data.foundational_score
        + profile.w_influence * node.data.influence_score;

    let final_score = urgency_factor * learning_potential;

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
        };
        let node = InMemNode::new(candidate);
        let profile = UserProfile::balanced();
        let readiness = 1.0;
        let days_overdue = 0.0;

        let (score, tie_breaker) =
            calculate_priority_score(&node, &profile, readiness, days_overdue);

        // urgency_factor = 1.0 + 1.0 * ln(1.0 + 0.0) = 1.0
        // learning_potential = 1.0 * 1.0 + 1.0 * 0.5 + 1.0 * 0.3 = 1.8
        // final_score = 1.0 * 1.8 = 1.8
        assert!((score - 1.8).abs() < 0.001);
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
        };
        let node = InMemNode::new(candidate);
        let profile = UserProfile::balanced();
        let readiness = 0.8;
        let days_overdue = 5.0;

        let (score, tie_breaker) =
            calculate_priority_score(&node, &profile, readiness, days_overdue);

        // urgency_factor = 1.0 + 1.0 * ln(1.0 + 5.0) = 1.0 + ln(6.0) ≈ 1.0 + 1.79 ≈ 2.79
        // learning_potential = 1.0 * 0.8 + 1.0 * 0.5 + 1.0 * 0.3 = 1.6
        // final_score = 2.79 * 1.6 ≈ 4.46
        assert!(score > 4.0 && score < 5.0);
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
}
