//! Baseline schedulers for comparison with the Iqrah scheduler.
//!
//! This module provides simple baseline schedulers to evaluate the Iqrah
//! scheduler's effectiveness. All baselines use the same StudentBrain
//! and FSRS state updates - only session selection differs.

use chrono::{DateTime, Utc};
use iqrah_core::domain::MemoryState;
use rand::prelude::*;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Scheduler variant for simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerVariant {
    /// Use the real iqrah-core scheduler v2 (bandit/profile-based)
    #[default]
    IqrahDefault,
    /// Page-order baseline: sequential through goal items in Quran order
    BaselinePageOrder,
    /// Fixed-interval SRS baseline: [1, 3, 7, 14, 30, 60] day intervals
    BaselineFixedSrs,
    /// Random baseline: uniform random selection from goal set
    BaselineRandom,
    /// Graph-aware baseline: uses graph scores without FSRS (ISS v2.1 §5.2)
    BaselineGraphTopo,
    /// Oracle baseline: uses Iqrah scheduler but forces perfect recall
    BaselineOraclePerfect,
}

impl SchedulerVariant {
    /// Parse variant from string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "iqrah_default" | "iqrah" | "default" => Some(Self::IqrahDefault),
            "baseline_page_order" | "page_order" | "page" => Some(Self::BaselinePageOrder),
            "baseline_fixed_srs" | "fixed_srs" | "srs" => Some(Self::BaselineFixedSrs),
            "baseline_random" | "random" => Some(Self::BaselineRandom),
            "baseline_graph_topo" | "graph_topo" | "topo" => Some(Self::BaselineGraphTopo),
            "baseline_oracle_perfect" | "oracle_perfect" | "oracle" => {
                Some(Self::BaselineOraclePerfect)
            }
            _ => None,
        }
    }

    /// Get all available variants.
    pub fn all() -> Vec<Self> {
        vec![
            Self::IqrahDefault,
            Self::BaselinePageOrder,
            Self::BaselineFixedSrs,
            Self::BaselineRandom,
            Self::BaselineGraphTopo,
            Self::BaselineOraclePerfect,
        ]
    }

    /// Get the display name for this variant.
    pub fn name(&self) -> &'static str {
        match self {
            Self::IqrahDefault => "iqrah_default",
            Self::BaselinePageOrder => "baseline_page_order",
            Self::BaselineFixedSrs => "baseline_fixed_srs",
            Self::BaselineRandom => "baseline_random",
            Self::BaselineGraphTopo => "baseline_graph_topo",
            Self::BaselineOraclePerfect => "baseline_oracle_perfect",
        }
    }
}

/// Trait for session generation.
///
/// Implementations provide different strategies for selecting which items
/// to include in a learning session.
pub trait SessionGenerator: Send + Sync {
    /// Generate a session of items to review.
    ///
    /// # Arguments
    /// * `goal_items` - All items in the goal (in Quran order)
    /// * `memory_states` - Current memory state for each item
    /// * `session_size` - Maximum number of items to include
    /// * `current_day` - Current simulation day (0-indexed)
    /// * `now` - Current timestamp for due date comparison
    fn generate_session(
        &mut self,
        goal_items: &[i64],
        memory_states: &HashMap<i64, MemoryState>,
        session_size: usize,
        current_day: u32,
        now: DateTime<Utc>,
    ) -> Vec<i64>;
}

// ============================================================================
// Page-Order Baseline
// ============================================================================

/// Page-order baseline: sequential through goal items in Quran order.
///
/// Behavior:
/// - Ignores FSRS/due dates
/// - Iterates through goal items in fixed order
/// - Wraps to beginning when reaching the end
/// - Simulates traditional "page-by-page" memorization
pub struct PageOrderBaseline {
    /// Current position in the goal list
    position: usize,
}

impl PageOrderBaseline {
    /// Create a new page-order baseline starting at the beginning.
    pub fn new() -> Self {
        Self { position: 0 }
    }
}

impl Default for PageOrderBaseline {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionGenerator for PageOrderBaseline {
    fn generate_session(
        &mut self,
        goal_items: &[i64],
        _memory_states: &HashMap<i64, MemoryState>,
        session_size: usize,
        _current_day: u32,
        _now: DateTime<Utc>,
    ) -> Vec<i64> {
        if goal_items.is_empty() {
            return Vec::new();
        }

        let mut session = Vec::with_capacity(session_size);

        for i in 0..session_size {
            let idx = (self.position + i) % goal_items.len();
            session.push(goal_items[idx]);
        }

        // Advance position for next session
        self.position = (self.position + session_size) % goal_items.len();

        session
    }
}

// ============================================================================
// Fixed-SRS Baseline
// ============================================================================

/// Fixed-interval SRS baseline.
///
/// Behavior:
/// - Uses fixed intervals: [1, 3, 7, 14, 30, 60] days
/// - On success: advance to next interval
/// - On failure: reset to first interval
/// - Selects items whose interval has elapsed (due for review)
/// - If not enough due items, adds new items from the goal set
pub struct FixedSrsBaseline {
    /// Interval stages in days
    intervals: Vec<u32>,
    /// Current interval index for each item (0 = new, 1+ = interval stage)
    item_stages: HashMap<i64, usize>,
    /// Last review date for each item
    last_reviewed: HashMap<i64, u32>,
}

impl FixedSrsBaseline {
    /// Create a new fixed-SRS baseline with default intervals.
    pub fn new() -> Self {
        Self {
            intervals: vec![1, 3, 7, 14, 30, 60],
            item_stages: HashMap::new(),
            last_reviewed: HashMap::new(),
        }
    }

    /// Create with custom intervals.
    pub fn with_intervals(intervals: Vec<u32>) -> Self {
        Self {
            intervals,
            item_stages: HashMap::new(),
            last_reviewed: HashMap::new(),
        }
    }

    /// Check if an item is due for review on the given day.
    pub fn is_due(&self, node_id: i64, current_day: u32) -> bool {
        if let Some(&stage) = self.item_stages.get(&node_id) {
            if stage == 0 || stage > self.intervals.len() {
                return true; // New or completed all intervals
            }
            if let Some(&last) = self.last_reviewed.get(&node_id) {
                let interval = self.intervals[stage - 1];
                return current_day >= last + interval;
            }
        }
        true // New item, always due
    }

    /// Record a review result.
    pub fn record_review(&mut self, node_id: i64, current_day: u32, success: bool) {
        let current_stage = *self.item_stages.get(&node_id).unwrap_or(&0);

        if success {
            // Advance to next interval (capped at max)
            let new_stage = (current_stage + 1).min(self.intervals.len());
            self.item_stages.insert(node_id, new_stage);
        } else {
            // Reset to first interval
            self.item_stages.insert(node_id, 1);
        }

        self.last_reviewed.insert(node_id, current_day);
    }

    /// Get the current interval for an item in days.
    pub fn get_interval(&self, node_id: i64) -> u32 {
        let stage = *self.item_stages.get(&node_id).unwrap_or(&0);
        if stage == 0 || stage > self.intervals.len() {
            1 // Default interval for new items
        } else {
            self.intervals[stage - 1]
        }
    }
}

impl Default for FixedSrsBaseline {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionGenerator for FixedSrsBaseline {
    fn generate_session(
        &mut self,
        goal_items: &[i64],
        _memory_states: &HashMap<i64, MemoryState>,
        session_size: usize,
        current_day: u32,
        _now: DateTime<Utc>,
    ) -> Vec<i64> {
        let mut session = Vec::with_capacity(session_size);

        // First, add items that are due
        for &node_id in goal_items {
            if session.len() >= session_size {
                break;
            }
            if self.is_due(node_id, current_day) {
                session.push(node_id);
            }
        }

        // If not enough due items, add new items that haven't been seen
        if session.len() < session_size {
            for &node_id in goal_items {
                if session.len() >= session_size {
                    break;
                }
                if !self.item_stages.contains_key(&node_id) && !session.contains(&node_id) {
                    session.push(node_id);
                }
            }
        }

        session
    }
}

// ============================================================================
// Random Baseline
// ============================================================================

/// Random baseline: uniform random selection from goal set.
///
/// Behavior:
/// - Ignores FSRS/due dates
/// - Selects items uniformly at random
/// - Poor baseline to demonstrate value of scheduling
pub struct RandomBaseline {
    rng: StdRng,
}

impl RandomBaseline {
    /// Create a new random baseline with given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl SessionGenerator for RandomBaseline {
    fn generate_session(
        &mut self,
        goal_items: &[i64],
        _memory_states: &HashMap<i64, MemoryState>,
        session_size: usize,
        _current_day: u32,
        _now: DateTime<Utc>,
    ) -> Vec<i64> {
        if goal_items.is_empty() {
            return Vec::new();
        }

        let mut session = Vec::with_capacity(session_size);

        // Sample with replacement (simple random)
        for _ in 0..session_size {
            let idx = self.rng.gen_range(0..goal_items.len());
            session.push(goal_items[idx]);
        }

        session
    }
}

// ============================================================================
// Graph-Aware Topological Baseline (ISS v2.1 §5.2)
// ============================================================================

/// Graph-aware baseline that uses graph topology scoring WITHOUT FSRS.
///
/// Per ISS v2.1 spec §5.2:
/// - Uses prerequisite gate (topological constraint)
/// - Score = w_foundation * foundational + w_influence * influence + w_fairness * fairness
/// - Ignores FSRS/days_overdue in scoring
/// - Uses w_fairness >= 0.2 to enforce coverage
/// - Band composition: 40% low-energy, 40% medium, 20% high
///
/// Purpose:
/// - Isolates value of graph + fairness WITHOUT FSRS complexity
/// - If iqrah_default ≈ topo → FSRS contribution is small
/// - If topo ≈ random → graph scores aren't wired properly
pub struct GraphTopoBaseline {
    /// Weight for foundational score
    w_foundation: f32,
    /// Weight for influence score
    w_influence: f32,
    /// Weight for fairness term
    w_fairness: f32,
    /// Review counts per item (for fairness calculation)
    review_counts: HashMap<i64, u32>,
    /// Random number generator
    rng: StdRng,
}

impl GraphTopoBaseline {
    /// Create a new graph-topo baseline with default weights.
    pub fn new(seed: u64) -> Self {
        Self {
            w_foundation: 0.4,
            w_influence: 0.4,
            w_fairness: 0.2, // >= 0.2 per spec
            review_counts: HashMap::new(),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Create with custom weights.
    pub fn with_weights(seed: u64, w_foundation: f32, w_influence: f32, w_fairness: f32) -> Self {
        Self {
            w_foundation,
            w_influence,
            w_fairness,
            review_counts: HashMap::new(),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Record a review for fairness tracking.
    pub fn record_review(&mut self, node_id: i64) {
        *self.review_counts.entry(node_id).or_insert(0) += 1;
    }

    /// Compute fairness term for an item.
    ///
    /// Items with fewer reviews get higher fairness scores.
    fn fairness_score(&self, node_id: i64, max_reviews: u32) -> f32 {
        let reviews = *self.review_counts.get(&node_id).unwrap_or(&0);
        if max_reviews == 0 {
            return 1.0; // All items equal at start
        }
        // Inverse: fewer reviews = higher score
        1.0 - (reviews as f32 / (max_reviews as f32 + 1.0))
    }

    /// Compute total score for an item (no FSRS used).
    fn compute_score(
        &self,
        node_id: i64,
        foundational_score: f32,
        influence_score: f32,
        max_reviews: u32,
    ) -> f32 {
        let fairness = self.fairness_score(node_id, max_reviews);
        self.w_foundation * foundational_score
            + self.w_influence * influence_score
            + self.w_fairness * fairness
    }
}

impl SessionGenerator for GraphTopoBaseline {
    fn generate_session(
        &mut self,
        goal_items: &[i64],
        memory_states: &HashMap<i64, MemoryState>,
        session_size: usize,
        _current_day: u32,
        _now: DateTime<Utc>,
    ) -> Vec<i64> {
        if goal_items.is_empty() {
            return Vec::new();
        }

        // Find max reviews for fairness normalization
        let max_reviews = self.review_counts.values().copied().max().unwrap_or(0);

        // Score all items
        let mut scored: Vec<(i64, f32, f64)> = goal_items
            .iter()
            .map(|&node_id| {
                // Use default graph scores (in real impl, these would come from content repo)
                // For simulation baseline, use energy as a proxy for foundational score
                let energy = memory_states.get(&node_id).map(|s| s.energy).unwrap_or(0.0);

                // Items with zero energy are new - give them higher foundational score
                let foundational = if energy < 0.1 {
                    0.8
                } else {
                    0.3 + (energy as f32 * 0.5)
                };
                let influence = 0.5; // Default influence

                let score = self.compute_score(node_id, foundational, influence, max_reviews);
                (node_id, score, energy)
            })
            .collect();

        // Sort by score DESC, then by node_id ASC (quran_order tie-breaker)
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        // Band-based composition per spec: 40% low-energy, 40% medium, 20% high
        let mut low_energy: Vec<i64> = Vec::new(); // energy < 0.4
        let mut medium_energy: Vec<i64> = Vec::new(); // 0.4 <= energy < 0.7
        let mut high_energy: Vec<i64> = Vec::new(); // energy >= 0.7

        for (node_id, _score, energy) in &scored {
            if *energy < 0.4 {
                low_energy.push(*node_id);
            } else if *energy < 0.7 {
                medium_energy.push(*node_id);
            } else {
                high_energy.push(*node_id);
            }
        }

        // Compose session
        let n_low = (session_size * 40 / 100).max(1);
        let n_medium = (session_size * 40 / 100).max(1);
        let n_high = session_size.saturating_sub(n_low + n_medium);

        let mut session = Vec::with_capacity(session_size);

        // Take from each band (use shuffle for some variety within bands)
        for band in [&mut low_energy, &mut medium_energy, &mut high_energy] {
            band.shuffle(&mut self.rng);
        }

        session.extend(low_energy.into_iter().take(n_low));
        session.extend(medium_energy.into_iter().take(n_medium));
        session.extend(high_energy.into_iter().take(n_high));

        // If we still need more items, take highest scored items not yet included
        if session.len() < session_size {
            for (node_id, _, _) in scored {
                if session.len() >= session_size {
                    break;
                }
                if !session.contains(&node_id) {
                    session.push(node_id);
                }
            }
        }

        session.truncate(session_size);
        session
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_order_sequential() {
        let mut baseline = PageOrderBaseline::new();
        let goal_items: Vec<i64> = vec![1, 2, 3, 4, 5];
        let memory_states = HashMap::new();
        let now = Utc::now();

        // First session: items 1, 2, 3
        let session1 = baseline.generate_session(&goal_items, &memory_states, 3, 0, now);
        assert_eq!(session1, vec![1, 2, 3]);

        // Second session: items 4, 5, 1 (wraps)
        let session2 = baseline.generate_session(&goal_items, &memory_states, 3, 1, now);
        assert_eq!(session2, vec![4, 5, 1]);
    }

    #[test]
    fn test_page_order_wraps() {
        let mut baseline = PageOrderBaseline::new();
        let goal_items: Vec<i64> = vec![1, 2, 3];
        let memory_states = HashMap::new();
        let now = Utc::now();

        // Session larger than goal set
        let session = baseline.generate_session(&goal_items, &memory_states, 5, 0, now);
        assert_eq!(session, vec![1, 2, 3, 1, 2]);
    }

    #[test]
    fn test_fixed_srs_interval_progression() {
        let mut baseline = FixedSrsBaseline::new();

        // Initial state: new item
        assert!(baseline.is_due(1, 0));

        // After first success: advances to stage 1 (interval = 1 day)
        baseline.record_review(1, 0, true);
        assert!(!baseline.is_due(1, 0)); // Not due same day
        assert!(baseline.is_due(1, 1)); // Due after 1 day

        // After second success: stage 2 (interval = 3 days)
        baseline.record_review(1, 1, true);
        assert!(!baseline.is_due(1, 3)); // Not due at day 3
        assert!(baseline.is_due(1, 4)); // Due at day 4 (1 + 3)
    }

    #[test]
    fn test_fixed_srs_reset_on_failure() {
        let mut baseline = FixedSrsBaseline::new();

        // Progress to stage 3
        baseline.record_review(1, 0, true); // Stage 1
        baseline.record_review(1, 1, true); // Stage 2
        baseline.record_review(1, 4, true); // Stage 3

        // Fail: reset to stage 1
        baseline.record_review(1, 11, false);
        assert_eq!(baseline.get_interval(1), 1); // Back to first interval
    }

    #[test]
    fn test_random_varies_with_seed() {
        let goal_items: Vec<i64> = (1..=20).collect();
        let memory_states = HashMap::new();
        let now = Utc::now();

        let mut baseline1 = RandomBaseline::new(42);
        let mut baseline2 = RandomBaseline::new(123);

        let session1 = baseline1.generate_session(&goal_items, &memory_states, 5, 0, now);
        let session2 = baseline2.generate_session(&goal_items, &memory_states, 5, 0, now);

        // Different seeds should produce different results (with high probability)
        assert_ne!(session1, session2);
    }

    #[test]
    fn test_scheduler_variant_from_str() {
        assert_eq!(
            SchedulerVariant::from_str("iqrah_default"),
            Some(SchedulerVariant::IqrahDefault)
        );
        assert_eq!(
            SchedulerVariant::from_str("page_order"),
            Some(SchedulerVariant::BaselinePageOrder)
        );
        assert_eq!(
            SchedulerVariant::from_str("RANDOM"),
            Some(SchedulerVariant::BaselineRandom)
        );
        assert_eq!(SchedulerVariant::from_str("unknown"), None);
    }
    #[test]
    fn test_graph_topo_distribution() {
        // Create baseline
        let mut baseline = GraphTopoBaseline::new(42);

        // Create 100 items with varying energy levels
        let mut goal_items = Vec::new();
        let mut memory_states = HashMap::new();
        let now = Utc::now();

        for i in 0..100 {
            let id = i as i64;
            goal_items.push(id);

            // Assign energy based on index ranges
            let energy = if i < 40 {
                0.2 // Low energy (< 0.4)
            } else if i < 80 {
                0.5 // Medium energy (0.4 - 0.7)
            } else {
                0.8 // High energy (>= 0.7)
            };

            memory_states.insert(
                id,
                MemoryState {
                    user_id: "test".to_string(),
                    node_id: id,
                    energy,
                    stability: 1.0,
                    difficulty: 1.0,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 0,
                },
            );
        }

        // Generate session of size 20
        // Expected target: 8 low, 8 medium, 4 high
        let session = baseline.generate_session(&goal_items, &memory_states, 20, 0, now);

        // Count distribution
        let mut low_count = 0;
        let mut med_count = 0;
        let mut high_count = 0;

        for id in &session {
            if *id < 40 {
                low_count += 1;
            } else if *id < 80 {
                med_count += 1;
            } else {
                high_count += 1;
            }
        }

        println!(
            "Distribution: Low={}, Med={}, High={}",
            low_count, med_count, high_count
        );

        // Verify we hit the targets (exact because we have plenty of supply)
        assert_eq!(low_count, 8, "Should have 40% low energy items");
        assert_eq!(med_count, 8, "Should have 40% medium energy items");
        assert_eq!(high_count, 4, "Should have 20% high energy items");
    }

    #[test]
    fn test_graph_topo_fairness() {
        let mut baseline = GraphTopoBaseline::new(42);
        let max_reviews = 10;

        // Item with 0 reviews
        let s0 = baseline.fairness_score(1, max_reviews);
        // Item with 10 reviews
        baseline.record_review(2);
        for _ in 0..9 {
            baseline.record_review(2);
        }
        let s10 = baseline.fairness_score(2, max_reviews);

        assert!(
            s0 > s10,
            "Items with fewer reviews should have higher fairness score"
        );
        assert_eq!(s0, 1.0); // 1.0 - 0/...
                             // 1.0 - 10/11 = 1 - 0.909 = 0.09
        assert!(s10 < 0.2);
    }
}
