//! Translation exercises: Independent item accuracy-based evaluation.
//!
//! For vocabulary and translation axes, items are tested independently
//! (no sequential dependencies). Scoring is based on accuracy:
//! - Item is "correct" if P(recall) > 0.50
//! - Overall score = correct_count / total_items

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
// Translation Exercise Types
// ============================================================================

/// Translation exercise evaluates independent comprehension.
pub struct TranslationExercise {
    pub exercise_type: TranslationExerciseType,
}

/// Subtypes of translation exercises.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype", rename_all = "snake_case")]
pub enum TranslationExerciseType {
    /// Recall meanings of random ayahs.
    MeaningRecall {
        sample_size: usize,
        sampling_strategy: SamplingStrategy,
    },

    /// Recall word meanings (vocabulary test).
    VocabularyRecall {
        sample_size: usize,
        sampling_strategy: SamplingStrategy,
    },
}

impl TranslationExercise {
    /// Create a new translation exercise.
    pub fn new(exercise_type: TranslationExerciseType) -> Self {
        Self { exercise_type }
    }

    /// Create a meaning recall exercise.
    pub fn meaning_recall(sample_size: usize, strategy: SamplingStrategy) -> Self {
        Self::new(TranslationExerciseType::MeaningRecall {
            sample_size,
            sampling_strategy: strategy,
        })
    }

    /// Create a vocabulary recall exercise.
    pub fn vocabulary_recall(sample_size: usize, strategy: SamplingStrategy) -> Self {
        Self::new(TranslationExerciseType::VocabularyRecall {
            sample_size,
            sampling_strategy: strategy,
        })
    }
}

// ============================================================================
// Exercise Implementation
// ============================================================================

impl Exercise for TranslationExercise {
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
                details: ExerciseDetails::Translation {
                    accuracy: 0.0,
                    items_correct: 0,
                    items_tested: 0,
                    item_probabilities: vec![],
                },
                summary: "No items to test".to_string(),
                // v2.9 fields
                sampled: 0,
                attempted: 0,
                unavailable: 0,
                availability_ratio: 0.0,
                mean_trials_attempted: None,
                grade_attempted: None,
            });
        }

        let mut items_correct = 0;
        let items_tested = test_nodes.len();
        let mut item_probabilities = Vec::new();

        // v2.9: Track availability
        let mut items_attempted = 0;
        let mut items_unavailable = 0;

        for node_id in &test_nodes {
            let p_recall = if let Some(state) = memory_states.get(node_id) {
                items_attempted += 1;
                compute_recall_probability(state, brain, current_day)
            } else {
                items_unavailable += 1;
                0.0 // Never learned
            };

            item_probabilities.push(p_recall);

            // Binary: correct if P(recall) > 0.50
            if p_recall > CORRECT_THRESHOLD {
                items_correct += 1;
            }
        }

        let accuracy = items_correct as f64 / items_tested.max(1) as f64;
        let availability_ratio = items_attempted as f64 / items_tested.max(1) as f64;

        // Map accuracy to grade
        let grade = accuracy_to_grade(accuracy);
        let score = grade_to_score(grade);

        // v2.9: Compute attempted-only metrics
        let accuracy_attempted = if items_attempted > 0 {
            items_correct as f64 / items_attempted as f64
        } else {
            0.0
        };
        let grade_attempted = if items_attempted > 0 {
            Some(accuracy_to_grade(accuracy_attempted))
        } else {
            None
        };

        let summary = if items_unavailable > 0 {
            format!(
                "Translation: {} sampled, {} attempted ({:.1}% availability)\n  {}/{} correct ({:.1}%), grade: {:?}",
                items_tested, items_attempted, availability_ratio * 100.0,
                items_correct, items_attempted, accuracy_attempted * 100.0,
                grade_attempted.unwrap_or(ReviewGrade::Again)
            )
        } else {
            format!(
                "Translation: {}/{} correct ({:.1}% accuracy), grade: {:?}",
                items_correct,
                items_tested,
                accuracy * 100.0,
                grade
            )
        };

        Ok(ExerciseResult {
            score,
            grade,
            details: ExerciseDetails::Translation {
                accuracy,
                items_correct,
                items_tested,
                item_probabilities,
            },
            summary,
            // v2.9 fields
            sampled: items_tested,
            attempted: items_attempted,
            unavailable: items_unavailable,
            availability_ratio,
            mean_trials_attempted: None, // Not applicable for translation (no trials)
            grade_attempted,
        })
    }

    fn metadata(&self) -> ExerciseMetadata {
        ExerciseMetadata {
            name: match &self.exercise_type {
                TranslationExerciseType::MeaningRecall { .. } => "Meaning Recall",
                TranslationExerciseType::VocabularyRecall { .. } => "Vocabulary Recall",
            }
            .to_string(),
            axis: AxisKind::Translation,
            difficulty: 4.0,
            description: "Tests independent item comprehension without sequential dependencies"
                .to_string(),
        }
    }
}

impl TranslationExercise {
    /// Get test nodes based on exercise type.
    fn get_test_nodes(
        &self,
        goal_items: &[i64],
        memory_states: &HashMap<i64, MemoryState>,
    ) -> Result<Vec<i64>> {
        match &self.exercise_type {
            TranslationExerciseType::MeaningRecall {
                sample_size,
                sampling_strategy,
            } => {
                // Sample ayahs for meaning recall
                use super::helpers::sample_ayahs;
                let mut rng = rand::thread_rng();
                sample_ayahs(
                    goal_items,
                    *sample_size,
                    *sampling_strategy,
                    memory_states,
                    &mut rng,
                )
            }

            TranslationExerciseType::VocabularyRecall {
                sample_size,
                sampling_strategy,
            } => {
                // Sample items (could be words or ayahs)
                let mut items = goal_items.to_vec();

                match sampling_strategy {
                    SamplingStrategy::Random => {
                        let mut rng = rand::thread_rng();
                        items.shuffle(&mut rng);
                    }
                    SamplingStrategy::Urgency => {
                        items.sort_by(|a, b| {
                            let e_a = memory_states.get(a).map(|s| s.energy).unwrap_or(0.0);
                            let e_b = memory_states.get(b).map(|s| s.energy).unwrap_or(0.0);
                            e_a.partial_cmp(&e_b).unwrap_or(std::cmp::Ordering::Equal)
                        });
                    }
                    SamplingStrategy::Coverage => {
                        items.sort_by(|a, b| {
                            let r_a = memory_states.get(a).map(|s| s.review_count).unwrap_or(0);
                            let r_b = memory_states.get(b).map(|s| s.review_count).unwrap_or(0);
                            r_a.cmp(&r_b)
                        });
                    }
                    _ => {}
                }

                Ok(items.into_iter().take(*sample_size).collect())
            }
        }
    }
}

// ============================================================================
// Accuracy-Based Cognitive Model
// ============================================================================

/// Threshold for considering an item "correct".
/// Item is correct if P(recall) > 0.50.
const CORRECT_THRESHOLD: f64 = 0.50;

/// Compute recall probability from memory state and brain parameters.
fn compute_recall_probability(state: &MemoryState, brain: &StudentBrain, current_day: u32) -> f64 {
    let last_reviewed_day = state.last_reviewed.timestamp() / (24 * 60 * 60);
    let elapsed = (current_day as i64).saturating_sub(last_reviewed_day) as f64;

    // Use brain's recall probability model
    let base_recall = brain.compute_recall_probability(
        state.stability,
        elapsed,
        0.5, // Assume mid-exercise
    );

    // Adjust by energy
    let energy_factor = state.energy as f64;

    // Final probability
    (base_recall * energy_factor).clamp(0.01, 0.99)
}

/// Map accuracy to FSRS grade.
///
/// Thresholds:
/// - 90%+ accuracy → Easy
/// - 70-90% accuracy → Good
/// - 50-70% accuracy → Hard
/// - <50% accuracy → Again
pub fn accuracy_to_grade(accuracy: f64) -> ReviewGrade {
    if accuracy >= 0.90 {
        ReviewGrade::Easy
    } else if accuracy >= 0.70 {
        ReviewGrade::Good
    } else if accuracy >= 0.50 {
        ReviewGrade::Hard
    } else {
        ReviewGrade::Again
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accuracy_to_grade_easy() {
        assert_eq!(accuracy_to_grade(1.0), ReviewGrade::Easy);
        assert_eq!(accuracy_to_grade(0.95), ReviewGrade::Easy);
        assert_eq!(accuracy_to_grade(0.90), ReviewGrade::Easy);
    }

    #[test]
    fn test_accuracy_to_grade_good() {
        assert_eq!(accuracy_to_grade(0.89), ReviewGrade::Good);
        assert_eq!(accuracy_to_grade(0.80), ReviewGrade::Good);
        assert_eq!(accuracy_to_grade(0.70), ReviewGrade::Good);
    }

    #[test]
    fn test_accuracy_to_grade_hard() {
        assert_eq!(accuracy_to_grade(0.69), ReviewGrade::Hard);
        assert_eq!(accuracy_to_grade(0.60), ReviewGrade::Hard);
        assert_eq!(accuracy_to_grade(0.50), ReviewGrade::Hard);
    }

    #[test]
    fn test_accuracy_to_grade_again() {
        assert_eq!(accuracy_to_grade(0.49), ReviewGrade::Again);
        assert_eq!(accuracy_to_grade(0.25), ReviewGrade::Again);
        assert_eq!(accuracy_to_grade(0.0), ReviewGrade::Again);
    }

    #[test]
    fn test_correct_threshold() {
        // P(recall) = 0.51 → correct
        assert!(0.51 > CORRECT_THRESHOLD);

        // P(recall) = 0.49 → incorrect
        assert!(0.49 <= CORRECT_THRESHOLD);
    }

    #[test]
    fn test_accuracy_calculation() {
        // 40/50 correct = 80% accuracy = Good
        let accuracy: f64 = 40.0 / 50.0;
        assert!((accuracy - 0.80_f64).abs() < 0.01);
        assert_eq!(accuracy_to_grade(accuracy), ReviewGrade::Good);
    }
}
