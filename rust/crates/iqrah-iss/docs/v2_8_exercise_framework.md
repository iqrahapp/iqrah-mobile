# ISS v2.8: Exercise-Based Evaluation Framework

**Status**: Design Phase
**Priority**: HIGH - Addresses fundamental evaluation limitations
**Target**: Ship 3-4 weeks after v2.7 production validation
**Effort Estimate**: 32-41 hours

---

## Executive Summary

### The Problem with v2.7

**v2.7 has two critical evaluation limitations**:

1. **Binary recall model** (unrealistic):
```
   Can you recall this ayah? Yes/No

   Reality: Recalls with hesitation (2-3 attempts)
   v2.7: Treats as simple success

   Result: Cannot distinguish smooth recall from struggling recall
```

2. **Hardcoded evaluation** (not extensible):
```
   Evaluation logic scattered in simulator.rs
   Adding new test = core code changes
   Cannot easily test different cognitive skills
```

3. **Axis-agnostic evaluation** (inappropriate):
```
   Sequential recitation applied to vocabulary
   Independent item testing applied to memorization

   Result: Wrong evaluation strategy for axis type
```

### The Solution (v2.8)

**Exercise Framework**: Pluggable, axis-aware cognitive evaluations
```rust
trait Exercise {
    fn sample_nodes(graph, config) -> Vec<node_id>;
    fn evaluate(graph, memory_states, brain) -> ExerciseResult;
}

// Memory exercise (axis: memorization)
MemoryExercise::continuous_recitation() → trials-based score

// Translation exercise (axis: vocabulary/translation)
TranslationExercise::vocabulary_recall() → accuracy-based score

// Grammar exercise (axis: grammar)
GrammarExercise::case_endings() → rule application score
```

**Key innovation: Trials-based modeling**
```
Instead of: Can you recall word X? Yes/No

Model reality: How many attempts until successful recall?

Word 1: energy=0.90 → P(recall)=0.90 → E(trials)=1.11 (immediate)
Word 2: energy=0.70 → P(recall)=0.70 → E(trials)=1.43 (1-2 hesitations)
Word 3: energy=0.50 → P(recall)=0.50 → E(trials)=2.00 (2-3 attempts)
Word 4: energy=0.30 → P(recall)=0.30 → E(trials)=3.33 (severe struggle)

Total: 7.87 trials for 4 words
Average: 1.97 trials/word
Grade: Good (some hesitation but eventual success)
```

### Expected Impact

**Memorization axis (Juz 30)**:
```
Before v2.8 (binary recall):
- iqrah: 0.580 score (40% coverage)
- random: 0.609 score (80% coverage)
- Winner: random (+5%)

After v2.8 (trials-based):
- iqrah: 0.750 score (avg 1.5 trials/word, smooth recitation)
- random: 0.380 score (avg 4.2 trials/word, cannot maintain sequence)
- Winner: iqrah (+97% improvement)

Reason: Random learns many items but weak sequential links
        → high trial counts → low exercise score
```

**Vocabulary axis (500 common words)**:
```
Before v2.8:
- iqrah: 0.580 score
- random: 0.609 score (random better on scattered items)

After v2.8 (independent item testing):
- iqrah: 0.590 score (frequency-based introduction)
- random: 0.610 score (distributed practice)
- Winner: random (+3%, acceptable)

Reason: Both strategies valid for independent items
        Exercise confirms this is correct behavior
```

---

## Architecture

### Core Abstraction: Exercise Trait

**File**: `crates/iqrah-iss/src/exercises/mod.rs`
```rust
//! Exercise framework for axis-specific cognitive evaluation

use crate::brain::StudentBrain;
use crate::memory_state::MemoryState;
use iqrah_core::domain::{AxisType, ReviewGrade};
use iqrah_knowledge_graph::Graph;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Exercise trait: Pluggable evaluation of specific cognitive skills
pub trait Exercise: Send + Sync {
    /// Sample nodes from graph for this exercise
    ///
    /// Examples:
    /// - Memory exercise: All words in ayah (sequential)
    /// - Vocabulary exercise: Random 50 words (independent)
    /// - Grammar exercise: Verbs matching pattern (rule-based)
    fn sample_nodes(
        &self,
        graph: &Graph,
        config: &ExerciseConfig,
    ) -> Result<Vec<i64>>;

    /// Evaluate student performance on this exercise
    ///
    /// Returns:
    /// - Overall score (0.0-1.0)
    /// - FSRS-style grade (Easy/Good/Hard/Again)
    /// - Exercise-specific metrics
    fn evaluate(
        &self,
        graph: &Graph,
        memory_states: &[MemoryState],
        brain: &StudentBrain,
        current_day: u32,
    ) -> Result<ExerciseResult>;

    /// Exercise metadata (name, axis, difficulty)
    fn metadata(&self) -> ExerciseMetadata;
}

/// Result of exercise evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseResult {
    /// Overall score (0.0 = total failure, 1.0 = perfect)
    pub score: f64,

    /// FSRS-style grade
    pub grade: ReviewGrade,

    /// Exercise-specific detailed metrics
    pub details: ExerciseDetails,

    /// Human-readable summary
    pub summary: String,
}

/// Exercise-specific evaluation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExerciseDetails {
    /// Memory exercise: Trials-based sequential recall
    Memory {
        /// Expected trials per item (geometric distribution)
        trials_per_item: Vec<f64>,

        /// Total expected trials across all items
        total_trials: f64,

        /// Average trials per item
        avg_trials: f64,

        /// Number of items tested
        items_tested: usize,

        /// Number of items that would fail (trials > threshold)
        items_failed: usize,

        /// Longest continuous streak (no failures)
        longest_streak: usize,
    },

    /// Translation exercise: Independent item accuracy
    Translation {
        /// Overall accuracy (0.0-1.0)
        accuracy: f64,

        /// Items correctly recalled
        items_correct: usize,

        /// Total items tested
        items_tested: usize,

        /// Per-item recall probabilities
        item_probabilities: Vec<f64>,
    },

    /// Grammar exercise: Rule application success
    Grammar {
        /// Rules correctly applied
        rules_correct: usize,

        /// Total rules tested
        rules_total: usize,

        /// Accuracy per rule category
        category_accuracy: Vec<(String, f64)>,
    },
}

/// Exercise configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseConfig {
    /// Exercise type identifier
    pub exercise_type: String,

    /// Exercise subtype (type-specific)
    pub subtype: String,

    /// Frequency (how often to run, in days)
    pub frequency_days: u32,

    /// Sampling strategy
    pub sampling_strategy: SamplingStrategy,

    /// Sample size (if applicable)
    pub sample_size: Option<usize>,

    /// Type-specific parameters (JSON object)
    pub parameters: serde_json::Value,
}

/// Sampling strategy for node selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SamplingStrategy {
    /// Sample randomly from goal
    Random,

    /// Use full range (start to end)
    FullRange,

    /// Sample based on urgency (lowest energy first)
    Urgency,

    /// Sample least-reviewed items
    Coverage,

    /// Sample by frequency (most common first)
    Frequency,
}

/// Exercise metadata
#[derive(Debug, Clone)]
pub struct ExerciseMetadata {
    /// Human-readable name
    pub name: String,

    /// Which axis this evaluates
    pub axis: AxisType,

    /// Base difficulty (1.0-10.0)
    pub difficulty: f64,

    /// Description
    pub description: String,
}
```

---

## Memory Exercise: Trials-Based Sequential Recall

### Cognitive Model

**Key insight**: Recall is not binary - it's a process with variable attempts.

**Geometric distribution modeling**:
```
P(recall on first attempt) = p
P(recall on second attempt) = (1-p) × p
P(recall on kth attempt) = (1-p)^(k-1) × p

Expected trials until success: E[T] = 1/p

Examples:
p=0.90 (high energy) → E[T]=1.11 trials (immediate recall)
p=0.70 (medium)      → E[T]=1.43 trials (1-2 hesitations)
p=0.50 (borderline)  → E[T]=2.00 trials (multiple attempts)
p=0.30 (weak)        → E[T]=3.33 trials (severe struggle)
p=0.10 (very weak)   → E[T]=10.0 trials (effectively fails)
```

**Failure threshold**: E[T] > 6.0 trials = item fails (cannot recall reliably)

### Implementation

**File**: `crates/iqrah-iss/src/exercises/memory.rs`
```rust
//! Memory exercises: Sequential recall, recitation, continuous memory

use super::*;
use rand::Rng;

/// Memory exercise evaluates sequential recall ability
pub struct MemoryExercise {
    pub exercise_type: MemoryExerciseType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype", rename_all = "snake_case")]
pub enum MemoryExerciseType {
    /// Recite a single ayah word-by-word
    AyahRecitation {
        /// Specific ayah ID (None = sample from goal)
        ayah_id: Option<i64>,
    },

    /// Recite continuous sequence of ayahs
    ContinuousRecitation {
        start_node: i64,
        end_node: i64,
    },

    /// Recite all words on a page
    PageRecitation {
        page_number: u16,
    },

    /// Sample N random ayahs and test each
    SampleRecitation {
        sample_size: usize,
        sampling_strategy: SamplingStrategy,
    },
}

impl Exercise for MemoryExercise {
    fn sample_nodes(
        &self,
        graph: &Graph,
        config: &ExerciseConfig,
    ) -> Result<Vec<i64>> {
        match &self.exercise_type {
            MemoryExerciseType::AyahRecitation { ayah_id } => {
                let ayah = if let Some(id) = ayah_id {
                    *id
                } else {
                    // Sample random ayah from goal
                    sample_random_ayah(graph, config)?
                };

                // Get all word instances in sequential order
                get_word_sequence_for_ayah(graph, ayah)
            }

            MemoryExerciseType::ContinuousRecitation { start_node, end_node } => {
                // Get sequential path from graph
                // This returns all word instances from start to end
                graph.get_sequential_path(*start_node, *end_node)
            }

            MemoryExerciseType::PageRecitation { page_number } => {
                // Get all words on this page in reading order
                graph.get_words_on_page(*page_number)
            }

            MemoryExerciseType::SampleRecitation { sample_size, sampling_strategy } => {
                // Sample N ayahs according to strategy
                let ayahs = sample_ayahs(graph, *sample_size, *sampling_strategy, config)?;

                // Get all words from all sampled ayahs
                let mut all_words = Vec::new();
                for ayah_id in ayahs {
                    let words = get_word_sequence_for_ayah(graph, ayah_id)?;
                    all_words.extend(words);
                }

                Ok(all_words)
            }
        }
    }

    fn evaluate(
        &self,
        graph: &Graph,
        memory_states: &[MemoryState],
        brain: &StudentBrain,
        current_day: u32,
    ) -> Result<ExerciseResult> {
        // Sample nodes for this exercise
        let nodes = self.sample_nodes(graph, &ExerciseConfig::default())?;

        let mut trials_per_item = Vec::new();
        let mut total_trials = 0.0;
        let mut items_failed = 0;
        let mut current_streak = 0;
        let mut longest_streak = 0;

        for node_id in &nodes {
            // Find memory state for this node
            let state = memory_states.iter().find(|s| s.node_id == *node_id);

            let expected_trials = if let Some(state) = state {
                // Compute recall probability from memory state
                let p_recall = compute_recall_probability(
                    state,
                    brain,
                    current_day,
                );

                // Expected trials until success (geometric distribution)
                // E[T] = 1/p, clamped to avoid division by zero
                let trials = 1.0 / p_recall.max(0.01);

                // Failure threshold: > 6.0 trials = cannot recall reliably
                if trials > 6.0 {
                    items_failed += 1;
                    current_streak = 0;
                    trials.min(20.0)  // Cap for scoring
                } else {
                    current_streak += 1;
                    longest_streak = longest_streak.max(current_streak);
                    trials
                }
            } else {
                // Item never introduced → infinite trials (fail)
                items_failed += 1;
                current_streak = 0;
                20.0  // Capped for scoring
            };

            trials_per_item.push(expected_trials);
            total_trials += expected_trials;
        }

        let items_tested = nodes.len();
        let avg_trials = total_trials / items_tested as f64;

        // Map average trials to FSRS grade
        let grade = trials_to_grade(avg_trials);
        let score = grade_to_score(grade);

        // Generate summary
        let summary = format!(
            "Recitation: {} items, avg {:.2} trials/item, {} failures, grade: {:?}",
            items_tested, avg_trials, items_failed, grade
        );

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
            },
            summary,
        })
    }

    fn metadata(&self) -> ExerciseMetadata {
        ExerciseMetadata {
            name: match &self.exercise_type {
                MemoryExerciseType::AyahRecitation { .. } => "Ayah Recitation",
                MemoryExerciseType::ContinuousRecitation { .. } => "Continuous Recitation",
                MemoryExerciseType::PageRecitation { .. } => "Page Recitation",
                MemoryExerciseType::SampleRecitation { .. } => "Sample Recitation",
            }.to_string(),
            axis: AxisType::Memorization,
            difficulty: 5.0,  // Medium difficulty
            description: "Tests ability to recall sequential content with minimal hesitation".to_string(),
        }
    }
}

/// Compute recall probability from memory state and brain parameters
fn compute_recall_probability(
    state: &MemoryState,
    brain: &StudentBrain,
    current_day: u32,
) -> f64 {
    let elapsed = current_day.saturating_sub(state.last_reviewed_day.unwrap_or(0));

    // Use ISS v2.7 recall probability model
    let base_recall = brain.compute_recall_probability(
        state.fsrs_state.stability,
        elapsed as f64,
        0.5,  // Assume mid-exercise
    );

    // Adjust by energy (ISS model: energy reflects short-term readiness)
    let energy_factor = state.energy;

    // Final probability (with safety clamps)
    (base_recall * energy_factor).clamp(0.01, 0.99)
}

/// Map average trials per item to FSRS grade
///
/// Thresholds calibrated to cognitive reality:
/// - 1.0-1.5 trials: Smooth, confident recall (Easy)
/// - 1.5-3.0 trials: Some hesitation but successful (Good)
/// - 3.0-6.0 trials: Significant struggle but completes (Hard)
/// - 6.0+ trials: Cannot recall reliably (Again)
fn trials_to_grade(avg_trials: f64) -> ReviewGrade {
    if avg_trials.is_infinite() || avg_trials > 6.0 {
        ReviewGrade::Again
    } else if avg_trials <= 1.5 {
        ReviewGrade::Easy
    } else if avg_trials <= 3.0 {
        ReviewGrade::Good
    } else {
        ReviewGrade::Hard
    }
}

/// Map FSRS grade to normalized score
fn grade_to_score(grade: ReviewGrade) -> f64 {
    match grade {
        ReviewGrade::Easy => 1.0,
        ReviewGrade::Good => 0.75,
        ReviewGrade::Hard => 0.50,
        ReviewGrade::Again => 0.0,
    }
}

/// Helper: Get all word instances for an ayah in sequential order
fn get_word_sequence_for_ayah(graph: &Graph, ayah_id: i64) -> Result<Vec<i64>> {
    // Query graph for word instances
    // Return in position order (word 1, word 2, word 3, ...)
    graph.get_word_instances_for_ayah(ayah_id)
}

/// Helper: Sample random ayah from goal range
fn sample_random_ayah(graph: &Graph, config: &ExerciseConfig) -> Result<i64> {
    let ayahs = graph.get_ayahs_in_goal_range()?;
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..ayahs.len());
    Ok(ayahs[idx])
}

/// Helper: Sample multiple ayahs according to strategy
fn sample_ayahs(
    graph: &Graph,
    sample_size: usize,
    strategy: SamplingStrategy,
    config: &ExerciseConfig,
) -> Result<Vec<i64>> {
    let mut ayahs = graph.get_ayahs_in_goal_range()?;

    match strategy {
        SamplingStrategy::Random => {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            ayahs.shuffle(&mut rng);
            Ok(ayahs.into_iter().take(sample_size).collect())
        }

        SamplingStrategy::FullRange => {
            // Take first N ayahs
            Ok(ayahs.into_iter().take(sample_size).collect())
        }

        SamplingStrategy::Urgency => {
            // Would need memory states to sort by energy
            // For now, fallback to random
            // TODO: Pass memory states to sample_nodes
            let mut rng = rand::thread_rng();
            ayahs.shuffle(&mut rng);
            Ok(ayahs.into_iter().take(sample_size).collect())
        }

        _ => {
            // Other strategies: implement as needed
            Err(anyhow::anyhow!("Sampling strategy {:?} not implemented for memory exercise", strategy))
        }
    }
}
```

### Example Evaluation

**Scenario**: Recite Surah Al-Fatiha (7 ayahs, 29 words)
```
Day 30 evaluation:

Ayah 1 (4 words):
  Word 1: E=0.95, P(recall)=0.92 → E[T]=1.09
  Word 2: E=0.88, P(recall)=0.85 → E[T]=1.18
  Word 3: E=0.90, P(recall)=0.87 → E[T]=1.15
  Word 4: E=0.85, P(recall)=0.82 → E[T]=1.22
  Subtotal: 4.64 trials (1.16 avg)

Ayah 2 (4 words):
  Word 1: E=0.82, P(recall)=0.78 → E[T]=1.28
  Word 2: E=0.75, P(recall)=0.70 → E[T]=1.43
  Word 3: E=0.80, P(recall)=0.75 → E[T]=1.33
  Word 4: E=0.78, P(recall)=0.72 → E[T]=1.39
  Subtotal: 5.43 trials (1.36 avg)

... (ayahs 3-7 similar)

Total: 29 words, 42.7 total trials
Average: 1.47 trials/word
Grade: Easy (smooth recitation, minimal hesitation)
Score: 1.0
```

**Comparison: Random strategy**
```
Day 30 evaluation (random learned scattered ayahs):

Ayah 1: Learned
  Word 1: E=0.70 → 1.43 trials
  Word 2: E=0.68 → 1.47 trials
  Word 3: E=0.30 → 3.33 trials (struggle)
  Word 4: E=0.25 → 4.00 trials (severe struggle)

Ayah 2: Not learned
  All words: E=0.05 → 20.0 trials (fail)

Ayah 3: Partially learned
  Mixed energies, high trial counts

Total: 29 words, 187.4 total trials
Average: 6.46 trials/word
Grade: Again (cannot recite continuously)
Score: 0.0
```

**Result**: iqrah wins decisively (1.0 vs 0.0)

---

## Translation Exercise: Independent Item Accuracy

### Cognitive Model

**Key difference from memory**: Items are independent, no sequential dependencies.

**Accuracy-based scoring**:
```
For each item:
  P(correct) = recall_probability

Binary outcome:
  Correct if P(recall) > threshold (typically 0.50)

Accuracy = correct_count / total_items

Grade mapping:
  90%+ accuracy → Easy
  70-90% accuracy → Good
  50-70% accuracy → Hard
  <50% accuracy → Again
```

### Implementation

**File**: `crates/iqrah-iss/src/exercises/translation.rs`
```rust
//! Translation exercises: Independent meaning recall, vocabulary

use super::*;

/// Translation exercise evaluates independent comprehension
pub struct TranslationExercise {
    pub exercise_type: TranslationExerciseType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype", rename_all = "snake_case")]
pub enum TranslationExerciseType {
    /// Recall meanings of random ayahs
    MeaningRecall {
        sample_size: usize,
        sampling_strategy: SamplingStrategy,
    },

    /// Recall word meanings (vocabulary test)
    VocabularyRecall {
        sample_size: usize,
        frequency_threshold: Option<u32>,
        sampling_strategy: SamplingStrategy,
    },
}

impl Exercise for TranslationExercise {
    fn sample_nodes(
        &self,
        graph: &Graph,
        config: &ExerciseConfig,
    ) -> Result<Vec<i64>> {
        match &self.exercise_type {
            TranslationExerciseType::MeaningRecall { sample_size, sampling_strategy } => {
                // Sample random ayahs from goal
                sample_ayahs(graph, *sample_size, *sampling_strategy, config)
            }

            TranslationExerciseType::VocabularyRecall {
                sample_size,
                frequency_threshold,
                sampling_strategy
            } => {
                let mut words = graph.get_all_words_in_goal()?;

                // Filter by frequency if specified
                if let Some(threshold) = frequency_threshold {
                    words.retain(|w| w.frequency >= *threshold);
                }

                // Apply sampling strategy
                match sampling_strategy {
                    SamplingStrategy::Frequency => {
                        // Sort by frequency descending (most common first)
                        words.sort_by(|a, b| b.frequency.cmp(&a.frequency));
                        Ok(words.into_iter().take(*sample_size).map(|w| w.id).collect())
                    }

                    SamplingStrategy::Random => {
                        use rand::seq::SliceRandom;
                        let mut rng = rand::thread_rng();
                        words.shuffle(&mut rng);
                        Ok(words.into_iter().take(*sample_size).map(|w| w.id).collect())
                    }

                    _ => {
                        // Other strategies as needed
                        Err(anyhow::anyhow!("Sampling strategy {:?} not implemented for translation", sampling_strategy))
                    }
                }
            }
        }
    }

    fn evaluate(
        &self,
        graph: &Graph,
        memory_states: &[MemoryState],
        brain: &StudentBrain,
        current_day: u32,
    ) -> Result<ExerciseResult> {
        let nodes = self.sample_nodes(graph, &ExerciseConfig::default())?;

        let mut items_correct = 0;
        let items_tested = nodes.len();
        let mut item_probabilities = Vec::new();

        for node_id in &nodes {
            let state = memory_states.iter().find(|s| s.node_id == *node_id);

            let p_recall = if let Some(state) = state {
                compute_recall_probability(state, brain, current_day)
            } else {
                0.0  // Never learned
            };

            item_probabilities.push(p_recall);

            // Binary: correct if P(recall) > 0.50
            if p_recall > 0.50 {
                items_correct += 1;
            }
        }

        let accuracy = items_correct as f64 / items_tested as f64;

        // Map accuracy to grade
        let grade = if accuracy >= 0.90 {
            ReviewGrade::Easy
        } else if accuracy >= 0.70 {
            ReviewGrade::Good
        } else if accuracy >= 0.50 {
            ReviewGrade::Hard
        } else {
            ReviewGrade::Again
        };

        let summary = format!(
            "Translation: {}/{} correct ({:.1}% accuracy), grade: {:?}",
            items_correct, items_tested, accuracy * 100.0, grade
        );

        Ok(ExerciseResult {
            score: accuracy,
            grade,
            details: ExerciseDetails::Translation {
                accuracy,
                items_correct,
                items_tested,
                item_probabilities,
            },
            summary,
        })
    }

    fn metadata(&self) -> ExerciseMetadata {
        ExerciseMetadata {
            name: match &self.exercise_type {
                TranslationExerciseType::MeaningRecall { .. } => "Meaning Recall",
                TranslationExerciseType::VocabularyRecall { .. } => "Vocabulary Recall",
            }.to_string(),
            axis: AxisType::Translation,  // Or Vocabulary depending on subtype
            difficulty: 4.0,
            description: "Tests independent item comprehension without sequential dependencies".to_string(),
        }
    }
}
```

### Example Evaluation

**Scenario**: Vocabulary test (50 common words)
```
Day 60 evaluation:

Word 1 "wa" (and): E=0.92, P(recall)=0.88 → Correct ✓
Word 2 "Allah": E=0.95, P(recall)=0.91 → Correct ✓
Word 3 "al-Rahman": E=0.78, P(recall)=0.72 → Correct ✓
Word 4 "yaum" (day): E=0.68, P(recall)=0.62 → Correct ✓
Word 5 "qala" (said): E=0.45, P(recall)=0.42 → Incorrect ✗
...
Word 50: E=0.55, P(recall)=0.51 → Correct ✓

Total: 50 words
Correct: 42 words
Accuracy: 84%
Grade: Good
Score: 0.75
```

**Comparison: iqrah vs random**
```
iqrah strategy (frequency-based):
- Learns most common words first
- Day 60: 42/50 correct (84%)
- High-frequency words well-mastered

random strategy (scattered):
- Learns random words
- Day 60: 45/50 correct (90%)
- Distributed practice effective for independent items

Result: Random slightly better (acceptable for vocabulary axis)
```

---

## Integration with Simulator

### Exercise Management

**File**: `crates/iqrah-iss/src/simulator.rs`
```rust
use crate::exercises::{Exercise, ExerciseResult, ExerciseSchedule};
use std::collections::HashMap;

pub struct Simulator {
    // ... existing fields ...

    /// Registered exercises
    exercises: Vec<Box<dyn Exercise>>,

    /// Exercise schedules (when to run each)
    exercise_schedules: HashMap<String, ExerciseSchedule>,
}

/// Exercise schedule tracking
#[derive(Debug, Clone)]
struct ExerciseSchedule {
    /// Exercise index in exercises vec
    exercise_idx: usize,

    /// Frequency in days
    frequency: u32,

    /// Last evaluation day (None if never run)
    last_run: Option<u32>,

    /// Next scheduled day
    next_run: u32,
}

impl Simulator {
    pub fn new(config: ScenarioConfig) -> Result<Self> {
        // ... existing init ...

        // Load exercises from config
        let (exercises, schedules) = load_exercises(&config)?;

        Ok(Self {
            // ... existing fields ...
            exercises,
            exercise_schedules: schedules,
        })
    }

    fn simulate_day(&mut self, day: u32) -> Result<()> {
        // ... existing daily simulation logic ...

        // Check which exercises are due
        self.run_scheduled_exercises(day)?;

        Ok(())
    }

    fn run_scheduled_exercises(&mut self, day: u32) -> Result<()> {
        // Collect exercises due today
        let mut due_exercises = Vec::new();

        for (name, schedule) in &self.exercise_schedules {
            if day >= schedule.next_run {
                due_exercises.push((name.clone(), schedule.exercise_idx));
            }
        }

        // Run each due exercise
        for (name, idx) in due_exercises {
            let exercise = &self.exercises[idx];

            let result = exercise.evaluate(
                &self.graph,
                &self.items,
                &self.brain,
                day,
            )?;

            // Emit event
            self.event_tx.send(SimulationEvent::ExerciseEvaluation {
                day,
                exercise_name: name.clone(),
                exercise_axis: exercise.metadata().axis,
                result: result.clone(),
            })?;

            // Update schedule
            if let Some(schedule) = self.exercise_schedules.get_mut(&name) {
                schedule.last_run = Some(day);
                schedule.next_run = day + schedule.frequency;
            }
        }

        Ok(())
    }
}

/// Load exercises from scenario configuration
fn load_exercises(
    config: &ScenarioConfig,
) -> Result<(Vec<Box<dyn Exercise>>, HashMap<String, ExerciseSchedule>)> {
    let mut exercises: Vec<Box<dyn Exercise>> = Vec::new();
    let mut schedules = HashMap::new();

    for (idx, ex_config) in config.exercises.iter().enumerate() {
        // Parse exercise type
        let exercise: Box<dyn Exercise> = match ex_config.exercise_type.as_str() {
            "memory" => {
                let subtype = parse_memory_subtype(&ex_config.subtype, &ex_config.parameters)?;
                Box::new(MemoryExercise {
                    exercise_type: subtype,
                })
            }

            "translation" => {
                let subtype = parse_translation_subtype(&ex_config.subtype, &ex_config.parameters)?;
                Box::new(TranslationExercise {
                    exercise_type: subtype,
                })
            }

            _ => bail!("Unknown exercise type: {}", ex_config.exercise_type),
        };

        let name = format!("{}_{}", ex_config.exercise_type, ex_config.subtype);

        // Create schedule
        let schedule = ExerciseSchedule {
            exercise_idx: idx,
            frequency: ex_config.frequency_days,
            last_run: None,
            next_run: ex_config.frequency_days,  // First run at day N
        };

        exercises.push(exercise);
        schedules.insert(name, schedule);
    }

    Ok((exercises, schedules))
}
```

---

## Event System

**File**: `crates/iqrah-iss/src/events/types.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SimulationEvent {
    // ... existing events ...

    /// Exercise evaluation result
    ExerciseEvaluation {
        day: u32,
        exercise_name: String,
        exercise_axis: AxisType,
        result: ExerciseResult,
    },
}
```

---

## Configuration

### Scenario YAML Format

**File**: `crates/iqrah-iss/configs/juz_amma_dedicated.yaml`
```yaml
name: "Memorize Juz 30 with Exercises"

goal:
  axis: memorization
  type: juz
  juz_number: 30

# Student parameters (unchanged from v2.7)
student_params:
  # ... existing parameters ...
  max_working_set: 350
  cluster_stability_threshold: 0.20
  drift_alpha_max: 0.01
  drift_alpha_min: 0.002

# Exercise evaluation framework (NEW in v2.8)
exercises:
  # Main evaluation: Continuous recitation
  - type: memory
    subtype: continuous_recitation
    frequency_days: 30
    parameters:
      start_node: ${goal.start_node}
      end_node: ${goal.end_node}

  # Supplementary: Sample random ayahs
  - type: memory
    subtype: sample_recitation
    frequency_days: 15
    parameters:
      sample_size: 5
      sampling_strategy: random

  # Optional: Test vocabulary comprehension
  - type: translation
    subtype: vocabulary_recall
    frequency_days: 30
    parameters:
      sample_size: 30
      frequency_threshold: 20  # Only test common words
      sampling_strategy: frequency

# Reporting configuration
reporting:
  include_exercise_details: true
  exercise_breakdown: true
```

### Vocabulary-Focused Configuration

**File**: `crates/iqrah-iss/configs/vocab_500_common.yaml`
```yaml
name: "Learn 500 Most Common Words"

goal:
  axis: vocabulary
  type: custom
  target: most_common_500_words

student_params:
  # Different strategy for vocabulary
  max_working_set: 500  # No artificial limit
  cluster_stability_threshold: 0.10  # Fast expansion
  drift_alpha_max: 0.015  # Moderate drift

# Vocabulary-specific exercises
exercises:
  # Primary evaluation: Meaning recall
  - type: translation
    subtype: vocabulary_recall
    frequency_days: 15
    parameters:
      sample_size: 50
      frequency_threshold: 10
      sampling_strategy: frequency

  # Secondary: Random sample (test retention)
  - type: translation
    subtype: vocabulary_recall
    frequency_days: 30
    parameters:
      sample_size: 100
      sampling_strategy: random
```

### Mixed Goal Configuration

**File**: `crates/iqrah-iss/configs/juz_amma_holistic.yaml`
```yaml
name: "Master Juz 30 Holistically"

goal:
  type: composite
  axes:
    - name: memorization
      axis: memorization
      target: juz:30
      weight: 0.5

    - name: vocabulary
      axis: vocabulary
      target: juz:30:words
      weight: 0.3

    - name: translation
      axis: translation
      target: juz:30:meanings
      weight: 0.2

# Exercises for each axis
exercises:
  # Memorization axis
  - type: memory
    subtype: continuous_recitation
    frequency_days: 30
    axis_filter: memorization
    parameters:
      start_node: ${goal.memorization.start_node}
      end_node: ${goal.memorization.end_node}

  # Vocabulary axis
  - type: translation
    subtype: vocabulary_recall
    frequency_days: 15
    axis_filter: vocabulary
    parameters:
      sample_size: 30
      sampling_strategy: frequency

  # Translation axis
  - type: translation
    subtype: meaning_recall
    frequency_days: 30
    axis_filter: translation
    parameters:
      sample_size: 20
      sampling_strategy: random

# Composite scoring
scoring:
  per_axis: true  # Compute separate scores per axis
  composite_weight_by_axis: true
```

---

## Reporting & Analysis

### Exercise Analysis

**File**: `crates/iqrah-iss/src/events/analyzer.rs`
```rust
/// Analyze exercise performance over time
pub fn analyze_exercise_performance(events: &[SimulationEvent]) -> ExerciseAnalysis {
    let mut analysis = ExerciseAnalysis::default();

    for event in events {
        if let SimulationEvent::ExerciseEvaluation { day, exercise_name, result, .. } = event {
            analysis.add_evaluation(*day, exercise_name.clone(), result.clone());
        }
    }

    analysis
}

#[derive(Debug, Default)]
pub struct ExerciseAnalysis {
    /// Per-exercise history
    pub exercise_history: HashMap<String, Vec<(u32, ExerciseResult)>>,

    /// Per-axis aggregates
    pub axis_performance: HashMap<AxisType, AxisPerformance>,
}

#[derive(Debug, Default)]
pub struct AxisPerformance {
    /// Latest score for this axis
    pub latest_score: f64,

    /// Score progression over time
    pub score_history: Vec<(u32, f64)>,

    /// Average grade
    pub avg_grade: f64,

    /// Trend (improving, stable, declining)
    pub trend: Trend,
}

#[derive(Debug, Clone, Copy)]
pub enum Trend {
    Improving,
    Stable,
    Declining,
}

impl ExerciseAnalysis {
    fn add_evaluation(&mut self, day: u32, name: String, result: ExerciseResult) {
        // Add to exercise history
        self.exercise_history
            .entry(name.clone())
            .or_default()
            .push((day, result.clone()));

        // Update axis aggregates
        let axis = result.exercise_axis();
        let perf = self.axis_performance.entry(axis).or_default();
        perf.latest_score = result.score;
        perf.score_history.push((day, result.score));
        perf.update_trend();
    }
}

/// Generate exercise report section
pub fn generate_exercise_report(analysis: &ExerciseAnalysis) -> String {
    let mut report = String::new();

    report.push_str("\n## Exercise Evaluation Results\n\n");

    // Per-axis summary
    report.push_str("### Performance by Axis\n\n");
    report.push_str("| Axis | Latest Score | Trend | Evaluations |\n");
    report.push_str("|------|--------------|-------|-------------|\n");

    for (axis, perf) in &analysis.axis_performance {
        let trend_icon = match perf.trend {
            Trend::Improving => "📈",
            Trend::Stable => "➡️",
            Trend::Declining => "📉",
        };

        report.push_str(&format!(
            "| {:?} | {:.2} | {} | {} |\n",
            axis,
            perf.latest_score,
            trend_icon,
            perf.score_history.len(),
        ));
    }

    // Detailed exercise breakdown
    report.push_str("\n### Detailed Exercise Results\n\n");

    for (name, history) in &analysis.exercise_history {
        report.push_str(&format!("**{}**:\n", name));

        for (day, result) in history {
            report.push_str(&format!("- Day {}: {} (grade: {:?})\n",
                day, result.summary, result.grade));

            // Include detailed metrics
            match &result.details {
                ExerciseDetails::Memory { avg_trials, items_failed, longest_streak, .. } => {
                    report.push_str(&format!(
                        "  - Avg trials: {:.2}, failures: {}, longest streak: {}\n",
                        avg_trials, items_failed, longest_streak
                    ));
                }

                ExerciseDetails::Translation { accuracy, items_correct, items_tested, .. } => {
                    report.push_str(&format!(
                        "  - Accuracy: {:.1}%, correct: {}/{}\n",
                        accuracy * 100.0, items_correct, items_tested
                    ));
                }

                _ => {}
            }
        }

        report.push_str("\n");
    }

    report
}
```

---

## Validation Strategy

### Test Scenarios

**1. Memorization Axis (Juz 30)**

Expected behavior:
```
iqrah (cluster gate):
- Day 30: Continuous recitation score: 0.85 (15 ayahs, avg 1.4 trials/word)
- Day 90: Continuous recitation score: 0.90 (45 ayahs, avg 1.3 trials/word)
- Day 180: Continuous recitation score: 0.88 (80 ayahs, avg 1.5 trials/word)

random (scattered):
- Day 30: Continuous recitation score: 0.15 (many items, weak links, avg 5.2 trials/word)
- Day 90: Continuous recitation score: 0.20 (coverage high, sequence weak)
- Day 180: Continuous recitation score: 0.18 (cannot maintain continuous recitation)

Result: iqrah wins decisively (4-5x better)
```

**2. Vocabulary Axis (500 common words)**

Expected behavior:
```
iqrah (frequency-based):
- Day 30: Vocabulary recall: 75% accuracy (38/50 words)
- Day 90: Vocabulary recall: 82% accuracy (123/150 words)
- Day 180: Vocabulary recall: 85% accuracy (255/300 words)

random (scattered):
- Day 30: Vocabulary recall: 78% accuracy (39/50 words)
- Day 90: Vocabulary recall: 85% accuracy (128/150 words)
- Day 180: Vocabulary recall: 88% accuracy (264/300 words)

Result: Random slightly better (acceptable, both strategies valid)
```

**3. Mixed Goal (Memorization + Vocabulary)**

Expected behavior:
```
Composite scoring:
- Memorization axis (weight 0.6): iqrah excels
- Vocabulary axis (weight 0.4): random excels

Final composite:
- iqrah: 0.6×0.88 + 0.4×0.82 = 0.856
- random: 0.6×0.18 + 0.4×0.88 = 0.460

Result: iqrah wins overall due to memorization dominance
```

### Success Criteria

- [ ] Memory exercise shows iqrah >> random (2x+ margin) on memorization axis
- [ ] Translation exercise shows iqrah ≈ random (±15%) on vocabulary axis
- [ ] Trials-based scores correlate with continuous recitation ability
- [ ] Exercise framework extensible (easy to add new exercise types)
- [ ] No v2.7 regressions (existing metrics stable)

---

## Implementation Plan

### Phase 1: Core Framework (8-10 hours)

**Tasks**:
- [ ] Define Exercise trait and types
- [ ] Implement ExerciseResult, ExerciseDetails enums
- [ ] Create ExerciseConfig parsing
- [ ] Integrate with simulator (scheduling, execution)
- [ ] Add ExerciseEvaluation event
- [ ] Unit tests for framework

**Files**:
- `crates/iqrah-iss/src/exercises/mod.rs` (new)
- `crates/iqrah-iss/src/simulator.rs` (update)
- `crates/iqrah-iss/src/events/types.rs` (update)

---

### Phase 2: Memory Exercise (10-12 hours)

**Tasks**:
- [ ] Implement MemoryExercise struct
- [ ] Implement all memory subtypes (ayah, continuous, page, sample)
- [ ] Implement trials-based evaluation logic
- [ ] Implement trials_to_grade mapping
- [ ] Graph helper functions (get_word_sequence, etc.)
- [ ] Unit tests for memory exercise

**Files**:
- `crates/iqrah-iss/src/exercises/memory.rs` (new)

---

### Phase 3: Translation Exercise (6-8 hours)

**Tasks**:
- [ ] Implement TranslationExercise struct
- [ ] Implement translation subtypes (meaning, vocabulary)
- [ ] Implement accuracy-based evaluation
- [ ] Sampling strategies (random, frequency-based)
- [ ] Unit tests for translation exercise

**Files**:
- `crates/iqrah-iss/src/exercises/translation.rs` (new)

---

### Phase 4: Reporting & Analysis (4-6 hours)

**Tasks**:
- [ ] Implement ExerciseAnalysis struct
- [ ] Implement per-axis aggregation
- [ ] Implement trend detection
- [ ] Generate exercise report section
- [ ] Add to benchmark comparison output

**Files**:
- `crates/iqrah-iss/src/events/analyzer.rs` (update)

---

### Phase 5: Configuration & Documentation (4-5 hours)

**Tasks**:
- [ ] Create example scenario configs
- [ ] Document exercise configuration format
- [ ] Update tuning guide with exercise parameters
- [ ] Create exercise design guide (for new exercise types)
- [ ] Update CHANGELOG.md

**Files**:
- `crates/iqrah-iss/configs/juz_amma_dedicated.yaml` (update)
- `crates/iqrah-iss/configs/vocab_500_common.yaml` (new)
- `crates/iqrah-iss/docs/exercise_guide.md` (new)

---

### Phase 6: Validation (6-8 hours)

**Tasks**:
- [ ] Run Juz 30 memorization benchmark
- [ ] Run 500 vocab benchmark
- [ ] Run mixed goal benchmark
- [ ] Verify iqrah >> random on memorization
- [ ] Verify iqrah ≈ random on vocabulary
- [ ] Tune trials thresholds if needed
- [ ] Generate comparison report

---

## Timeline

**Optimistic**: 38-49 hours
**Realistic**: Add 20% buffer = 46-59 hours
**Deliverable**: 2-3 weeks (10-15 hours/week)

**Phases**:
- Week 1: Core framework + Memory exercise (Phases 1-2)
- Week 2: Translation exercise + Reporting (Phases 3-4)
- Week 3: Config + Validation (Phases 5-6)

---

## Migration from v2.7

### Breaking Changes

**None** - v2.8 is fully backward compatible:
- Exercises are additive (optional feature)
- Existing v2.7 evaluation continues to work
- New events ignored by v2.7 analyzers
- Configs without exercises run as v2.7

### Enabling Exercises

**Add to existing v2.7 config**:
```yaml
# Add this section to enable exercises
exercises:
  - type: memory
    subtype: continuous_recitation
    frequency_days: 30
    parameters:
      start_node: ${goal.start_node}
      end_node: ${goal.end_node}
```

**That's it!** No other changes needed.

---

## Future Extensions (v2.9+)

### Grammar Exercise
```rust
impl Exercise for GrammarExercise {
    // Test rule application in novel contexts
    // e.g., conjugate verb in correct case/tense
}
```

### Listening Exercise
```rust
impl Exercise for ListeningExercise {
    // Audio playback simulation
    // Test auditory recognition
}
```

### Writing Exercise
```rust
impl Exercise for WritingExercise {
    // Test production (not just recognition)
    // Requires different cognitive model
}
```

---

## Conclusion

v2.8 introduces a clean, extensible exercise framework that:

1. **Models reality**: Trials-based evaluation matches cognitive processes
2. **Axis-aware**: Different exercises for different learning types
3. **Extensible**: Easy to add new exercise types
4. **Validates design**: Proves cluster gate superiority for memorization

**This is the evaluation system ISS should have had from the start.**

---

**End of v2.8 Design Document**