//! Exercise framework for axis-specific cognitive evaluation.
//!
//! ISS v2.8 introduces pluggable exercises that evaluate student performance
//! using axis-appropriate methods:
//! - Memory exercises use trials-based evaluation (models hesitation)
//! - Translation exercises use accuracy-based evaluation (independent items)

mod helpers;
mod loader;
mod memory;
mod translation;

pub use helpers::*;
pub use loader::load_exercises;
pub use memory::{MemoryExercise, MemoryExerciseType};
pub use translation::{TranslationExercise, TranslationExerciseType};

use crate::axis::AxisKind;
use crate::brain::StudentBrain;
use anyhow::Result;
use iqrah_core::domain::{MemoryState, ReviewGrade};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Exercise Trait
// ============================================================================

/// Exercise trait: Pluggable evaluation of specific cognitive skills.
///
/// Each exercise type models a different aspect of learning:
/// - Memory: Sequential recall with trials-based scoring
/// - Translation: Independent item accuracy
pub trait Exercise: Send + Sync {
    /// Evaluate student performance on this exercise.
    ///
    /// # Arguments
    /// * `memory_states` - Current memory states (node_id -> state)
    /// * `goal_items` - All goal node IDs (for sampling)
    /// * `brain` - Student cognitive model
    /// * `current_day` - Simulation day
    ///
    /// # Returns
    /// Exercise result with score, grade, and detailed metrics.
    fn evaluate(
        &self,
        memory_states: &HashMap<i64, MemoryState>,
        goal_items: &[i64],
        brain: &StudentBrain,
        current_day: u32,
    ) -> Result<ExerciseResult>;

    /// Exercise metadata (name, axis, difficulty).
    fn metadata(&self) -> ExerciseMetadata;
}

// ============================================================================
// Exercise Result
// ============================================================================

/// Result of exercise evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseResult {
    /// Overall score (0.0 = total failure, 1.0 = perfect).
    pub score: f64,

    /// FSRS-style grade.
    pub grade: ReviewGrade,

    /// Exercise-specific detailed metrics.
    pub details: ExerciseDetails,

    /// Human-readable summary.
    pub summary: String,

    // =========================================================================
    // ISS v2.9: Availability tracking (separates learning progress from recall)
    // =========================================================================
    /// Total items sampled for testing.
    #[serde(default)]
    pub sampled: usize,

    /// Items that have a MemoryState (were introduced).
    #[serde(default)]
    pub attempted: usize,

    /// Items without MemoryState (never introduced).
    #[serde(default)]
    pub unavailable: usize,

    /// Ratio of attempted to sampled (0.0 to 1.0).
    #[serde(default)]
    pub availability_ratio: f64,

    /// Mean trials computed only on attempted items (None if attempted==0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_trials_attempted: Option<f64>,

    /// Grade computed only on attempted items (None if attempted==0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grade_attempted: Option<ReviewGrade>,
}

/// Exercise-specific evaluation metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExerciseDetails {
    /// Memory exercise: Trials-based sequential recall.
    Memory {
        /// Expected trials per item (geometric distribution).
        trials_per_item: Vec<f64>,

        /// Total expected trials across all items.
        total_trials: f64,

        /// Average trials per item.
        avg_trials: f64,

        /// Number of items tested.
        items_tested: usize,

        /// Number of items that would fail (trials > threshold).
        items_failed: usize,

        /// Longest continuous streak (no failures).
        longest_streak: usize,

        // ISS v2.9: Track attempted vs unavailable
        /// Items that were attempted (had MemoryState).
        #[serde(default)]
        items_attempted: usize,

        /// Items unavailable (no MemoryState).
        #[serde(default)]
        items_unavailable: usize,

        /// Total trials for attempted items only.
        #[serde(default)]
        attempted_trials: f64,
    },

    /// Translation exercise: Independent item accuracy.
    Translation {
        /// Overall accuracy (0.0-1.0).
        accuracy: f64,

        /// Items correctly recalled.
        items_correct: usize,

        /// Total items tested.
        items_tested: usize,

        /// Per-item recall probabilities.
        item_probabilities: Vec<f64>,
    },
}

// ============================================================================
// Exercise Configuration
// ============================================================================

/// Exercise configuration (parsed from YAML).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseConfig {
    /// Exercise type identifier ("memory" or "translation").
    #[serde(rename = "type")]
    pub exercise_type: String,

    /// Exercise subtype (type-specific).
    pub subtype: String,

    /// Frequency (how often to run, in days).
    pub frequency_days: u32,

    /// Optional axis filter.
    #[serde(default)]
    pub axis_filter: Option<AxisKind>,

    /// Type-specific parameters (JSON object).
    #[serde(default)]
    pub parameters: serde_json::Value,
}

impl Default for ExerciseConfig {
    fn default() -> Self {
        Self {
            exercise_type: "memory".to_string(),
            subtype: "sample_recitation".to_string(),
            frequency_days: 30,
            axis_filter: None,
            parameters: serde_json::Value::Null,
        }
    }
}

/// Sampling strategy for node selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SamplingStrategy {
    /// Sample randomly from goal.
    #[default]
    Random,

    /// Use full range (start to end).
    FullRange,

    /// Sample based on urgency (lowest energy first).
    Urgency,

    /// Sample least-reviewed items.
    Coverage,

    /// Sample by frequency (most common first).
    Frequency,
}

/// Exercise metadata.
#[derive(Debug, Clone)]
pub struct ExerciseMetadata {
    /// Human-readable name.
    pub name: String,

    /// Which axis this evaluates.
    pub axis: AxisKind,

    /// Base difficulty (1.0-10.0).
    pub difficulty: f64,

    /// Description.
    pub description: String,
}

// ============================================================================
// Exercise Schedule
// ============================================================================

/// Schedule for tracking when to run an exercise.
#[derive(Debug, Clone)]
pub struct ExerciseSchedule {
    /// Exercise name (for identification).
    pub name: String,

    /// Frequency in days.
    pub frequency: u32,

    /// Last evaluation day (None if never run).
    pub last_run: Option<u32>,

    /// Next scheduled day.
    pub next_run: u32,
}

impl ExerciseSchedule {
    /// Create a new schedule.
    pub fn new(name: String, frequency: u32) -> Self {
        Self {
            name,
            frequency,
            last_run: None,
            next_run: frequency, // First run at day N
        }
    }

    /// Check if exercise is due on given day.
    pub fn is_due(&self, day: u32) -> bool {
        day >= self.next_run
    }

    /// Record that exercise was run.
    pub fn record_run(&mut self, day: u32) {
        self.last_run = Some(day);
        self.next_run = day + self.frequency;
    }
}

// ============================================================================
// Grade Utilities
// ============================================================================

/// Map FSRS grade to normalized score (0.0-1.0).
pub fn grade_to_score(grade: ReviewGrade) -> f64 {
    match grade {
        ReviewGrade::Easy => 1.0,
        ReviewGrade::Good => 0.75,
        ReviewGrade::Hard => 0.50,
        ReviewGrade::Again => 0.0,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_to_score() {
        assert_eq!(grade_to_score(ReviewGrade::Easy), 1.0);
        assert_eq!(grade_to_score(ReviewGrade::Good), 0.75);
        assert_eq!(grade_to_score(ReviewGrade::Hard), 0.50);
        assert_eq!(grade_to_score(ReviewGrade::Again), 0.0);
    }

    #[test]
    fn test_exercise_schedule_due() {
        let schedule = ExerciseSchedule::new("test".to_string(), 30);

        assert!(!schedule.is_due(0));
        assert!(!schedule.is_due(29));
        assert!(schedule.is_due(30));
        assert!(schedule.is_due(31));
    }

    #[test]
    fn test_exercise_schedule_record_run() {
        let mut schedule = ExerciseSchedule::new("test".to_string(), 30);

        schedule.record_run(30);
        assert_eq!(schedule.last_run, Some(30));
        assert_eq!(schedule.next_run, 60);

        assert!(!schedule.is_due(30));
        assert!(!schedule.is_due(59));
        assert!(schedule.is_due(60));
    }

    #[test]
    fn test_exercise_config_default() {
        let config = ExerciseConfig::default();
        assert_eq!(config.exercise_type, "memory");
        assert_eq!(config.frequency_days, 30);
    }

    #[test]
    fn test_exercise_config_yaml_round_trip() {
        let yaml = r#"
type: memory
subtype: continuous_recitation
frequency_days: 15
parameters:
  start_node: 1001
  end_node: 1007
"#;
        let config: ExerciseConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.exercise_type, "memory");
        assert_eq!(config.subtype, "continuous_recitation");
        assert_eq!(config.frequency_days, 15);
        assert_eq!(config.parameters["start_node"].as_i64(), Some(1001));
    }
}
