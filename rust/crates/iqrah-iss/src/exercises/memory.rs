//! Memory exercises: Sequential recall with trials-based evaluation.
//!
//! The key innovation is modeling recall as a **process with variable attempts**,
//! not just a binary success/failure outcome.
//!
//! **Geometric distribution modeling**:
//! ```text
//! P(recall on first attempt) = p
//! Expected trials until success: E[T] = 1/p
//!
//! Examples:
//! p=0.90 (high energy) → E[T]=1.11 trials (immediate recall)
//! p=0.70 (medium)      → E[T]=1.43 trials (1-2 hesitations)
//! p=0.50 (borderline)  → E[T]=2.00 trials (multiple attempts)
//! p=0.30 (weak)        → E[T]=3.33 trials (severe struggle)
//! ```
//!
//! This captures the cognitive reality that even successful recalls can
//! involve hesitation and multiple attempts.

use super::{grade_to_score, ExerciseDetails, ExerciseMetadata, ExerciseResult, SamplingStrategy};
use crate::axis::AxisKind;
use crate::brain::StudentBrain;
use crate::exercises::Exercise;
use anyhow::Result;
use iqrah_core::domain::{MemoryState, ReviewGrade};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Memory Exercise Types
// ============================================================================

/// Memory exercise evaluates sequential recall ability.
pub struct MemoryExercise {
    pub exercise_type: MemoryExerciseType,
}

/// Subtypes of memory exercises.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype", rename_all = "snake_case")]
pub enum MemoryExerciseType {
    /// Recite a single ayah word-by-word.
    AyahRecitation {
        /// Specific ayah ID (None = sample from goal).
        ayah_id: Option<i64>,
    },

    /// Recite continuous sequence of ayahs.
    ContinuousRecitation {
        /// Chapter number.
        chapter: i32,
        /// Start verse number.
        start_verse: i32,
        /// End verse number.
        end_verse: i32,
    },

    /// Recite all words on a page.
    PageRecitation { page_number: u16 },

    /// Sample N random ayahs and test each.
    SampleRecitation {
        sample_size: usize,
        sampling_strategy: SamplingStrategy,
    },
}

impl MemoryExercise {
    /// Create a new memory exercise.
    pub fn new(exercise_type: MemoryExerciseType) -> Self {
        Self { exercise_type }
    }

    /// Create an ayah recitation exercise.
    pub fn ayah_recitation(ayah_id: Option<i64>) -> Self {
        Self::new(MemoryExerciseType::AyahRecitation { ayah_id })
    }

    /// Create a sample recitation exercise.
    pub fn sample_recitation(sample_size: usize, strategy: SamplingStrategy) -> Self {
        Self::new(MemoryExerciseType::SampleRecitation {
            sample_size,
            sampling_strategy: strategy,
        })
    }
}

// ============================================================================
// Exercise Implementation
// ============================================================================

impl Exercise for MemoryExercise {
    fn evaluate(
        &self,
        memory_states: &HashMap<i64, MemoryState>,
        goal_items: &[i64],
        brain: &StudentBrain,
        current_day: u32,
    ) -> Result<ExerciseResult> {
        // Get nodes to test based on exercise type
        let test_nodes = self.get_test_nodes(goal_items, memory_states)?;

        if test_nodes.is_empty() {
            return Ok(ExerciseResult {
                score: 0.0,
                grade: ReviewGrade::Again,
                details: ExerciseDetails::Memory {
                    trials_per_item: vec![],
                    total_trials: 0.0,
                    avg_trials: 0.0,
                    items_tested: 0,
                    items_failed: 0,
                    longest_streak: 0,
                    items_attempted: 0,
                    items_unavailable: 0,
                    attempted_trials: 0.0,
                },
                summary: "No items to test".to_string(),
                sampled: 0,
                attempted: 0,
                unavailable: 0,
                availability_ratio: 0.0,
                mean_trials_attempted: None,
                grade_attempted: None,
            });
        }

        // ISS v2.9: Track attempted vs unavailable items separately
        let mut trials_per_item = Vec::new();
        let mut total_trials = 0.0;
        let mut items_failed: usize = 0;
        let mut current_streak = 0;
        let mut longest_streak = 0;

        // v2.9 tracking
        let mut items_attempted: usize = 0;
        let mut items_unavailable: usize = 0;
        let mut attempted_trials = 0.0;

        for node_id in &test_nodes {
            if let Some(state) = memory_states.get(node_id) {
                // Item was introduced - compute trials from memory state
                items_attempted += 1;
                let p_recall = compute_recall_probability(state, brain, current_day);
                let trials = 1.0 / p_recall.max(0.01);

                // Track trials for attempted items only
                let capped_trials = trials.min(MAX_TRIALS);
                attempted_trials += capped_trials;

                if trials > FAILURE_THRESHOLD {
                    items_failed += 1;
                    current_streak = 0;
                    trials_per_item.push(capped_trials);
                    total_trials += capped_trials;
                } else {
                    current_streak += 1;
                    longest_streak = longest_streak.max(current_streak);
                    trials_per_item.push(trials);
                    total_trials += trials;
                }
            } else {
                // ISS v2.9: Track as unavailable instead of "failed"
                items_unavailable += 1;
                // Still contribute to total for backward compat, but clearly marked
                trials_per_item.push(MAX_TRIALS);
                total_trials += MAX_TRIALS;
                items_failed += 1;
                current_streak = 0;
            }
        }

        let items_tested = test_nodes.len();
        let avg_trials = total_trials / items_tested.max(1) as f64;

        // v2.9: Compute attempted-only metrics
        let mean_trials_attempted = if items_attempted > 0 {
            Some(attempted_trials / items_attempted as f64)
        } else {
            None
        };

        let grade_attempted = mean_trials_attempted.map(trials_to_grade);
        let availability_ratio = items_attempted as f64 / items_tested.max(1) as f64;

        // Map average trials to FSRS grade (includes unavailable for backward compat)
        let grade = trials_to_grade(avg_trials);
        let score = grade_to_score(grade);

        // ISS v2.9: Improved summary showing availability
        let summary = if items_unavailable > 0 {
            format!(
                "Recitation: {} sampled, {} attempted ({:.1}% availability)\n  Attempted: avg {:.2} trials/item, {} failures, grade: {:?}\n  Unavailable: {} items",
                items_tested,
                items_attempted,
                availability_ratio * 100.0,
                mean_trials_attempted.unwrap_or(0.0),
                items_failed.saturating_sub(items_unavailable as usize),
                grade_attempted.unwrap_or(ReviewGrade::Again),
                items_unavailable
            )
        } else {
            format!(
                "Recitation: {} items, avg {:.2} trials/item, {} failures, grade: {:?}",
                items_tested, avg_trials, items_failed, grade
            )
        };

        Ok(ExerciseResult {
            score,
            grade,
            details: ExerciseDetails::Memory {
                trials_per_item,
                total_trials,
                avg_trials,
                items_tested,
                items_failed,
                longest_streak,
                items_attempted,
                items_unavailable,
                attempted_trials,
            },
            summary,
            sampled: items_tested,
            attempted: items_attempted,
            unavailable: items_unavailable,
            availability_ratio,
            mean_trials_attempted,
            grade_attempted,
        })
    }

    fn metadata(&self) -> ExerciseMetadata {
        ExerciseMetadata {
            name: match &self.exercise_type {
                MemoryExerciseType::AyahRecitation { .. } => "Ayah Recitation",
                MemoryExerciseType::ContinuousRecitation { .. } => "Continuous Recitation",
                MemoryExerciseType::PageRecitation { .. } => "Page Recitation",
                MemoryExerciseType::SampleRecitation { .. } => "Sample Recitation",
            }
            .to_string(),
            axis: AxisKind::Memorization,
            difficulty: 5.0, // Medium difficulty
            description: "Tests ability to recall sequential content with minimal hesitation"
                .to_string(),
        }
    }
}

impl MemoryExercise {
    /// Get test nodes based on exercise type.
    fn get_test_nodes(
        &self,
        goal_items: &[i64],
        memory_states: &HashMap<i64, MemoryState>,
    ) -> Result<Vec<i64>> {
        use super::helpers::{get_ayahs_from_goal, sample_ayahs};

        match &self.exercise_type {
            MemoryExerciseType::AyahRecitation { ayah_id } => {
                // For ayah recitation, test the words in the ayah
                // If no specific ayah, sample one
                let ayah = if let Some(id) = ayah_id {
                    *id
                } else {
                    let ayahs = get_ayahs_from_goal(goal_items);
                    if ayahs.is_empty() {
                        return Ok(vec![]);
                    }
                    let mut rng = rand::thread_rng();
                    ayahs[rng.gen_range(0..ayahs.len())]
                };

                // For now, return the ayah itself (word-level would need async)
                Ok(vec![ayah])
            }

            MemoryExerciseType::ContinuousRecitation { .. } => {
                // For continuous recitation, test all goal items in the range
                // (Would need async to get actual words)
                Ok(goal_items.to_vec())
            }

            MemoryExerciseType::PageRecitation { .. } => {
                // Would need async lookup
                Ok(goal_items.to_vec())
            }

            MemoryExerciseType::SampleRecitation {
                sample_size,
                sampling_strategy,
            } => {
                let mut rng = rand::thread_rng();
                sample_ayahs(
                    goal_items,
                    *sample_size,
                    *sampling_strategy,
                    memory_states,
                    &mut rng,
                )
            }
        }
    }
}

// ============================================================================
// Trials-Based Cognitive Model
// ============================================================================

/// Failure threshold in expected trials.
/// Items requiring > 6 trials are considered failures.
const FAILURE_THRESHOLD: f64 = 6.0;

/// Maximum trials for scoring (prevents infinite values).
const MAX_TRIALS: f64 = 20.0;

/// Compute recall probability from memory state and brain parameters.
///
/// Uses the ISS energy model directly as the recall probability estimate.
/// Energy represents the current cognitive strength (0-1) and already
/// incorporates FSRS stability and time decay through daily drift.
fn compute_recall_probability(
    state: &MemoryState,
    _brain: &StudentBrain,
    _current_day: u32,
) -> f64 {
    // The ISS energy model already tracks cognitive strength over time.
    // Energy ranges from 0 (never seen) to 1.0 (perfect recall).
    // Daily drift decays energy based on stability and time.
    //
    // Using energy directly as recall probability:
    // - E = 0.0 → P(recall) = 0.01 (clamped)
    // - E = 0.3 → P(recall) = 0.30 (struggling)
    // - E = 0.6 → P(recall) = 0.60 (decent)
    // - E = 0.9 → P(recall) = 0.90 (confident)

    let energy = state.energy as f64;

    // Clamp to valid probability range
    energy.clamp(0.01, 0.99)
}

/// Map average trials per item to FSRS grade.
///
/// Thresholds calibrated to cognitive reality:
/// - 1.0-1.5 trials: Smooth, confident recall (Easy)
/// - 1.5-3.0 trials: Some hesitation but successful (Good)
/// - 3.0-6.0 trials: Significant struggle but completes (Hard)
/// - 6.0+ trials: Cannot recall reliably (Again)
///
/// # Threshold Math
/// ```text
/// avg_trials ≤ 1.5 → Easy   // P(recall) ≥ 0.67 (67%+)
/// avg_trials ≤ 3.0 → Good   // P(recall) ≥ 0.33 (33%+)
/// avg_trials ≤ 6.0 → Hard   // P(recall) ≥ 0.17 (17%+)
/// avg_trials > 6.0 → Again  // P(recall) < 0.17 (< 17%)
/// ```
pub fn trials_to_grade(avg_trials: f64) -> ReviewGrade {
    if avg_trials.is_infinite() || avg_trials > FAILURE_THRESHOLD {
        ReviewGrade::Again
    } else if avg_trials <= 1.5 {
        ReviewGrade::Easy
    } else if avg_trials <= 3.0 {
        ReviewGrade::Good
    } else {
        ReviewGrade::Hard
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trials_to_grade_easy() {
        assert_eq!(trials_to_grade(1.0), ReviewGrade::Easy);
        assert_eq!(trials_to_grade(1.11), ReviewGrade::Easy);
        assert_eq!(trials_to_grade(1.5), ReviewGrade::Easy);
    }

    #[test]
    fn test_trials_to_grade_good() {
        assert_eq!(trials_to_grade(1.51), ReviewGrade::Good);
        assert_eq!(trials_to_grade(2.0), ReviewGrade::Good);
        assert_eq!(trials_to_grade(3.0), ReviewGrade::Good);
    }

    #[test]
    fn test_trials_to_grade_hard() {
        assert_eq!(trials_to_grade(3.01), ReviewGrade::Hard);
        assert_eq!(trials_to_grade(5.0), ReviewGrade::Hard);
        assert_eq!(trials_to_grade(6.0), ReviewGrade::Hard);
    }

    #[test]
    fn test_trials_to_grade_again() {
        assert_eq!(trials_to_grade(6.01), ReviewGrade::Again);
        assert_eq!(trials_to_grade(10.0), ReviewGrade::Again);
        assert_eq!(trials_to_grade(f64::INFINITY), ReviewGrade::Again);
    }

    #[test]
    fn test_expected_trials_from_probability() {
        // p=0.90 → E[T]=1.11
        let trials: f64 = 1.0 / 0.90;
        assert!((trials - 1.11_f64).abs() < 0.01);

        // p=0.50 → E[T]=2.0
        let trials: f64 = 1.0 / 0.50;
        assert!((trials - 2.0_f64).abs() < 0.01);

        // p=0.30 → E[T]=3.33
        let trials: f64 = 1.0 / 0.30;
        assert!((trials - 3.33_f64).abs() < 0.01);
    }

    #[test]
    fn test_high_energy_produces_easy_grade() {
        // E=0.80, p~0.80 → trials=1.25 → Easy
        let p_recall = 0.80;
        let trials = 1.0 / p_recall;
        assert!(trials < 1.5);
        assert_eq!(trials_to_grade(trials), ReviewGrade::Easy);
    }

    #[test]
    fn test_low_energy_produces_again_grade() {
        // E=0.15, p~0.15 → trials=6.67 → Again
        let p_recall = 0.15;
        let trials = 1.0 / p_recall;
        assert!(trials > 6.0);
        assert_eq!(trials_to_grade(trials), ReviewGrade::Again);
    }
}
