/// Core types for Scheduler v2.0
///
/// This module defines the fundamental data structures used by the prerequisite-aware
/// scheduler with bandit-driven personalization.
use std::collections::HashMap;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Mastery threshold for prerequisite gate.
/// A node is considered "mastered" if its energy >= MASTERY_THRESHOLD.
pub const MASTERY_THRESHOLD: f32 = 0.3;

// ============================================================================
// USER PROFILE
// ============================================================================

/// Represents a user's learning profile with different weight preferences.
///
/// These weights control the priority scoring function:
/// - w_urgency: Weight for time-based urgency (days overdue)
/// - w_readiness: Weight for readiness (parent mastery)
/// - w_foundation: Weight for foundational importance (PageRank on forward graph)
/// - w_influence: Weight for influence (PageRank on reversed graph)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UserProfile {
    pub w_urgency: f32,
    pub w_readiness: f32,
    pub w_foundation: f32,
    pub w_influence: f32,
}

impl UserProfile {
    /// Creates a balanced profile with all weights equal to 1.0
    pub fn balanced() -> Self {
        Self {
            w_urgency: 1.0,
            w_readiness: 1.0,
            w_foundation: 1.0,
            w_influence: 1.0,
        }
    }

    /// Blends this profile with another using a ratio.
    ///
    /// # Arguments
    /// * `other` - The other profile to blend with
    /// * `ratio` - Ratio of self to other (e.g., 0.8 means 80% self, 20% other)
    pub fn blend(&self, other: &Self, ratio: f32) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);
        let other_ratio = 1.0 - ratio;

        Self {
            w_urgency: self.w_urgency * ratio + other.w_urgency * other_ratio,
            w_readiness: self.w_readiness * ratio + other.w_readiness * other_ratio,
            w_foundation: self.w_foundation * ratio + other.w_foundation * other_ratio,
            w_influence: self.w_influence * ratio + other.w_influence * other_ratio,
        }
    }
}

// ============================================================================
// CANDIDATE NODE
// ============================================================================

/// Represents a candidate node for scheduling with all its metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateNode {
    /// Node ID (e.g., "1:1", "WORD:1:1:1")
    pub id: String,

    /// Foundational score: PageRank on forward graph (0.0-1.0)
    pub foundational_score: f32,

    /// Influence score: PageRank on reversed graph (0.0-1.0)
    pub influence_score: f32,

    /// Content difficulty: 0.0 (easy) to 1.0 (hard)
    pub difficulty_score: f32,

    /// User's current mastery energy for this node (0.0-1.0)
    /// 0.0 if new/never seen
    pub energy: f32,

    /// Next due timestamp in MILLISECONDS (epoch)
    /// 0 if new/never scheduled
    pub next_due_ts: i64,

    /// Qur'an ordering: (surah * 1_000_000) + (ayah * 1000) + word_idx
    /// Used as tie-breaker in priority scoring
    pub quran_order: i64,
}

// ============================================================================
// IN-MEMORY NODE
// ============================================================================

/// In-memory representation of a node with its prerequisite relationships.
#[derive(Debug, Clone)]
pub struct InMemNode {
    /// The candidate node data
    pub data: CandidateNode,

    /// IDs of prerequisite parents (nodes that must be mastered first)
    pub parent_ids: Vec<String>,
}

impl InMemNode {
    /// Creates a new in-memory node with no parents.
    pub fn new(data: CandidateNode) -> Self {
        Self {
            data,
            parent_ids: Vec::new(),
        }
    }

    /// Creates a new in-memory node with specified parents.
    pub fn with_parents(data: CandidateNode, parent_ids: Vec<String>) -> Self {
        Self { data, parent_ids }
    }
}

// ============================================================================
// PARENT ENERGY MAP
// ============================================================================

/// Map of node_id -> energy for looking up parent energies.
pub type ParentEnergyMap = HashMap<String, f32>;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile_balanced() {
        let profile = UserProfile::balanced();
        assert_eq!(profile.w_urgency, 1.0);
        assert_eq!(profile.w_readiness, 1.0);
        assert_eq!(profile.w_foundation, 1.0);
        assert_eq!(profile.w_influence, 1.0);
    }

    #[test]
    fn test_user_profile_blend() {
        let profile1 = UserProfile {
            w_urgency: 1.0,
            w_readiness: 1.0,
            w_foundation: 2.0,
            w_influence: 1.0,
        };
        let profile2 = UserProfile::balanced();

        // 80% profile1, 20% profile2
        let blended = profile1.blend(&profile2, 0.8);

        assert!((blended.w_urgency - 1.0).abs() < 0.001);
        assert!((blended.w_readiness - 1.0).abs() < 0.001);
        assert!((blended.w_foundation - 1.8).abs() < 0.001); // 0.8*2.0 + 0.2*1.0 = 1.8
        assert!((blended.w_influence - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_in_mem_node_creation() {
        let candidate = CandidateNode {
            id: "1:1".to_string(),
            foundational_score: 0.5,
            influence_score: 0.3,
            difficulty_score: 0.2,
            energy: 0.0,
            next_due_ts: 0,
            quran_order: 1001000,
        };

        let node = InMemNode::new(candidate.clone());
        assert_eq!(node.data.id, "1:1");
        assert!(node.parent_ids.is_empty());

        let node_with_parents = InMemNode::with_parents(
            candidate,
            vec!["ROOT:ktb".to_string(), "LEMMA:kataba".to_string()],
        );
        assert_eq!(node_with_parents.parent_ids.len(), 2);
    }
}
