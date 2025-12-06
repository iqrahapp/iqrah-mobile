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
}

impl SchedulerVariant {
    /// Parse variant from string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "iqrah_default" | "iqrah" | "default" => Some(Self::IqrahDefault),
            "baseline_page_order" | "page_order" | "page" => Some(Self::BaselinePageOrder),
            "baseline_fixed_srs" | "fixed_srs" | "srs" => Some(Self::BaselineFixedSrs),
            "baseline_random" | "random" => Some(Self::BaselineRandom),
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
        ]
    }

    /// Get the display name for this variant.
    pub fn name(&self) -> &'static str {
        match self {
            Self::IqrahDefault => "iqrah_default",
            Self::BaselinePageOrder => "baseline_page_order",
            Self::BaselineFixedSrs => "baseline_fixed_srs",
            Self::BaselineRandom => "baseline_random",
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
}
