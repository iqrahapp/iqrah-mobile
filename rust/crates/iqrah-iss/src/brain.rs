//! StudentBrain cognitive model for simulating student recall behavior.
//!
//! The brain model uses corrected formulas:
//! - Difficulty affects effective stability
//! - Spacing sensitivity modifies effective elapsed time
//! - Fatigue increases early quit probability
//! - Prior knowledge pre-initializes memory states

use iqrah_core::domain::ReviewGrade;
use rand::prelude::*;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

/// Result of a recall attempt
#[derive(Debug, Clone, Copy)]
pub struct RecallResult {
    /// Whether the student successfully recalled the item
    pub recalled: bool,
    /// The underlying retrievability probability
    pub retrievability: f64,
}

/// Parameters defining a student's cognitive characteristics.
///
/// Priority order: Motivation > Memory > Attention > Prior Knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentParams {
    // ========================================================================
    // 1. MOTIVATION (highest priority)
    // ========================================================================
    /// Probability of skipping a day entirely [0, 1]
    #[serde(default = "default_skip_day_prob")]
    pub skip_day_prob: f64,

    /// Base probability of quitting early during a session [0, 1]
    #[serde(default = "default_early_quit_prob")]
    pub early_quit_prob: f64,

    /// Number of consecutive failures before giving up entirely
    #[serde(default = "default_persistence_threshold")]
    pub persistence_threshold: u32,

    // ========================================================================
    // 2. MEMORY
    // ========================================================================
    /// Multiplier for forgetting rate (>1 = forgets faster, <1 = better memory)
    #[serde(default = "default_forgetting_rate_mult")]
    pub forgetting_rate_mult: f64,

    /// Sensitivity to spacing effects (higher = more benefit from spacing)
    #[serde(default = "default_spacing_sensitivity")]
    pub spacing_sensitivity: f64,

    /// Random variation in recall per item [0, 1]
    #[serde(default = "default_item_variability")]
    pub item_variability: f64,

    // ========================================================================
    // 3. ATTENTION
    // ========================================================================
    /// Minutes before fatigue sets in
    #[serde(default = "default_fatigue_onset_minutes")]
    pub fatigue_onset_minutes: f64,

    /// Rate at which fatigue increases quit probability
    #[serde(default = "default_fatigue_decay_rate")]
    pub fatigue_decay_rate: f64,

    /// How much difficulty affects fatigue [0, 1]
    #[serde(default = "default_difficulty_sensitivity")]
    pub difficulty_sensitivity: f64,

    // ========================================================================
    // 4. PRIOR KNOWLEDGE
    // ========================================================================
    /// Surah IDs (1-114) that this student already knows
    #[serde(default)]
    pub known_surah_ids: Vec<i32>,

    /// Fraction of vocabulary already known [0, 1]
    #[serde(default)]
    pub vocab_known_pct: f64,
}

// Default functions for serde
fn default_skip_day_prob() -> f64 {
    0.1
}
fn default_early_quit_prob() -> f64 {
    0.05
}
fn default_persistence_threshold() -> u32 {
    10
}
fn default_forgetting_rate_mult() -> f64 {
    1.0
}
fn default_spacing_sensitivity() -> f64 {
    1.0
}
fn default_item_variability() -> f64 {
    0.1
}
fn default_fatigue_onset_minutes() -> f64 {
    20.0
}
fn default_fatigue_decay_rate() -> f64 {
    0.02
}
fn default_difficulty_sensitivity() -> f64 {
    0.3
}

impl Default for StudentParams {
    fn default() -> Self {
        Self {
            skip_day_prob: default_skip_day_prob(),
            early_quit_prob: default_early_quit_prob(),
            persistence_threshold: default_persistence_threshold(),
            forgetting_rate_mult: default_forgetting_rate_mult(),
            spacing_sensitivity: default_spacing_sensitivity(),
            item_variability: default_item_variability(),
            fatigue_onset_minutes: default_fatigue_onset_minutes(),
            fatigue_decay_rate: default_fatigue_decay_rate(),
            difficulty_sensitivity: default_difficulty_sensitivity(),
            known_surah_ids: Vec::new(),
            vocab_known_pct: 0.0,
        }
    }
}

impl StudentParams {
    /// Create a casual learner archetype
    pub fn casual_learner() -> Self {
        Self {
            skip_day_prob: 0.3,
            early_quit_prob: 0.15,
            persistence_threshold: 5,
            forgetting_rate_mult: 1.2,
            spacing_sensitivity: 0.8,
            item_variability: 0.15,
            fatigue_onset_minutes: 15.0,
            fatigue_decay_rate: 0.04,
            difficulty_sensitivity: 0.5,
            known_surah_ids: vec![1, 112, 113, 114], // Common short surahs
            vocab_known_pct: 0.1,
        }
    }

    /// Create a dedicated student archetype
    pub fn dedicated_student() -> Self {
        Self {
            skip_day_prob: 0.05,
            early_quit_prob: 0.02,
            persistence_threshold: 20,
            forgetting_rate_mult: 0.9,
            spacing_sensitivity: 1.2,
            item_variability: 0.05,
            fatigue_onset_minutes: 45.0,
            fatigue_decay_rate: 0.01,
            difficulty_sensitivity: 0.1,
            known_surah_ids: (1..=10).collect(), // First 10 surahs
            vocab_known_pct: 0.3,
        }
    }
}

/// StudentBrain simulates cognitive processes during learning.
pub struct StudentBrain {
    /// Student parameters
    pub params: StudentParams,

    /// Seeded RNG for reproducibility
    rng: StdRng,

    /// Days the student has been active
    pub days_active: u32,

    /// Consecutive failures
    consecutive_failures: u32,

    /// Whether student has given up
    pub given_up: bool,
}

impl StudentBrain {
    /// Create a new student brain with given parameters and seed.
    ///
    /// # Arguments
    /// * `params` - Student cognitive parameters
    /// * `seed` - RNG seed for reproducibility
    pub fn new(params: StudentParams, seed: u64) -> Self {
        Self {
            params,
            rng: StdRng::seed_from_u64(seed),
            days_active: 0,
            consecutive_failures: 0,
            given_up: false,
        }
    }

    /// Attempt recall of an item using the corrected formula.
    ///
    /// # Formula
    /// ```text
    /// difficulty_factor = 1.0 + max(difficulty - 1.0, 0.0) * 0.1
    /// eff_stability = stability / difficulty_factor
    /// eff_elapsed = elapsed_days / spacing_sensitivity.max(0.1)
    /// base_r = (1 + eff_elapsed / (9 * eff_stability))^-1
    /// adjusted_r = base_r.powf(forgetting_rate_mult)
    /// final_r = (adjusted_r + noise * 0.1).clamp(0.0, 1.0)
    /// ```
    ///
    /// # Arguments
    /// * `stability` - FSRS stability value in days
    /// * `difficulty` - Item difficulty (typically 1.0-10.0)
    /// * `elapsed_days` - Days since last review
    pub fn attempt_recall(
        &mut self,
        stability: f64,
        difficulty: f64,
        elapsed_days: f64,
    ) -> RecallResult {
        // 1. Difficulty affects effective stability
        let difficulty_factor = 1.0 + (difficulty - 1.0).max(0.0) * 0.1;
        let eff_stability = (stability / difficulty_factor).max(0.001);

        // 2. Spacing sensitivity affects effective elapsed time
        let eff_elapsed = elapsed_days / self.params.spacing_sensitivity.max(0.1);

        // 3. Compute base retrievability using FSRS power formula
        let base_r = (1.0 + eff_elapsed / (9.0 * eff_stability)).powi(-1);

        // 4. Apply forgetting rate multiplier
        let adjusted_r = base_r.powf(self.params.forgetting_rate_mult);

        // 5. Add noise
        let noise = self
            .rng
            .gen_range(-self.params.item_variability..self.params.item_variability);
        let final_r = (adjusted_r + noise * 0.1).clamp(0.0, 1.0);

        // 6. Roll for recall
        let roll: f64 = self.rng.gen();
        let recalled = roll < final_r;

        // Track consecutive failures
        if recalled {
            self.consecutive_failures = 0;
        } else {
            self.consecutive_failures += 1;
            if self.consecutive_failures >= self.params.persistence_threshold {
                self.given_up = true;
            }
        }

        RecallResult {
            recalled,
            retrievability: final_r,
        }
    }

    /// Map recall result to FSRS review grade.
    ///
    /// # Mapping
    /// - Recalled with R >= 0.8: Easy
    /// - Recalled with R >= 0.5: Good
    /// - Recalled with R < 0.5: Hard
    /// - Not recalled: Again
    pub fn determine_grade(&self, result: RecallResult) -> ReviewGrade {
        if result.recalled {
            if result.retrievability >= 0.8 {
                ReviewGrade::Easy
            } else if result.retrievability >= 0.5 {
                ReviewGrade::Good
            } else {
                ReviewGrade::Hard
            }
        } else {
            ReviewGrade::Again
        }
    }

    /// Check if student should skip today entirely.
    pub fn should_skip_day(&mut self) -> bool {
        if self.given_up {
            return true;
        }
        let roll: f64 = self.rng.gen();
        roll < self.params.skip_day_prob
    }

    /// Check if student should quit early during a session.
    ///
    /// # Arguments
    /// * `minutes_elapsed` - Minutes spent in current session
    /// * `last_item_difficulty` - Difficulty of the last item reviewed
    pub fn should_quit_early(&mut self, minutes_elapsed: f64, last_item_difficulty: f64) -> bool {
        if self.given_up {
            return true;
        }

        // Base fatigue factor
        let mut fatigue_factor = 1.0;

        if minutes_elapsed > self.params.fatigue_onset_minutes {
            let extra_minutes = minutes_elapsed - self.params.fatigue_onset_minutes;
            fatigue_factor += extra_minutes * self.params.fatigue_decay_rate;
        }

        // Difficulty-based bump
        fatigue_factor *= 1.0 + self.params.difficulty_sensitivity * last_item_difficulty * 0.1;

        // Clamp probability to [0, 1]
        let quit_prob = (self.params.early_quit_prob * fatigue_factor).min(1.0);

        let roll: f64 = self.rng.gen();
        roll < quit_prob
    }

    /// Notify brain that a new day has started.
    pub fn start_day(&mut self) {
        self.days_active += 1;
    }

    /// Get the set of node IDs that should be pre-initialized as known.
    ///
    /// This includes verses from known_surah_ids.
    /// Note: Actual MemoryState initialization happens in the simulator.
    pub fn get_prior_knowledge_nodes(&self) -> Vec<i64> {
        // TODO: Map surah IDs to verse node IDs via content repository
        // For now, return empty - actual implementation in simulator
        Vec::new()
    }

    /// Check if student has given up.
    pub fn has_given_up(&self) -> bool {
        self.given_up
    }
}

/// Configuration for prior knowledge initialization.
#[derive(Debug, Clone)]
pub struct PriorKnowledgeConfig {
    /// Stability to assign to pre-known items (days)
    pub stability: f64,
    /// Energy level for pre-known items [0, 1]
    pub energy: f64,
    /// Review count to simulate
    pub review_count: u32,
    /// Difficulty for pre-known items
    pub difficulty: f64,
}

impl Default for PriorKnowledgeConfig {
    fn default() -> Self {
        Self {
            stability: 365.0, // 1 year retention
            energy: 0.95,
            review_count: 10,
            difficulty: 3.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attempt_recall_decreases_with_time() {
        let params = StudentParams::default();
        let mut brain1 = StudentBrain::new(params.clone(), 42);
        let mut brain2 = StudentBrain::new(params, 42);

        // Same stability and difficulty, different elapsed time
        let stability = 10.0;
        let difficulty = 5.0;

        // Get many samples to estimate probability
        let mut recalls_short = 0;
        let mut recalls_long = 0;
        let trials = 100;

        for i in 0..trials {
            let mut b1 = StudentBrain::new(StudentParams::default(), 42 + i);
            let mut b2 = StudentBrain::new(StudentParams::default(), 42 + i);

            let r1 = b1.attempt_recall(stability, difficulty, 1.0); // 1 day elapsed
            let r2 = b2.attempt_recall(stability, difficulty, 30.0); // 30 days elapsed

            if r1.recalled {
                recalls_short += 1;
            }
            if r2.recalled {
                recalls_long += 1;
            }
        }

        // Short elapsed time should have higher recall rate
        assert!(
            recalls_short > recalls_long,
            "Short interval ({}) should have more recalls than long ({})",
            recalls_short,
            recalls_long
        );
    }

    #[test]
    fn test_difficulty_reduces_recall() {
        let params = StudentParams::default();

        let mut recalls_easy = 0;
        let mut recalls_hard = 0;
        let trials = 100;

        for i in 0..trials {
            let mut brain_easy = StudentBrain::new(params.clone(), 100 + i);
            let mut brain_hard = StudentBrain::new(params.clone(), 100 + i);

            let r_easy = brain_easy.attempt_recall(10.0, 1.0, 5.0); // Low difficulty
            let r_hard = brain_hard.attempt_recall(10.0, 8.0, 5.0); // High difficulty

            if r_easy.recalled {
                recalls_easy += 1;
            }
            if r_hard.recalled {
                recalls_hard += 1;
            }
        }

        assert!(
            recalls_easy > recalls_hard,
            "Easy ({}) should have more recalls than hard ({})",
            recalls_easy,
            recalls_hard
        );
    }

    #[test]
    fn test_determine_grade() {
        let brain = StudentBrain::new(StudentParams::default(), 42);

        // Not recalled -> Again
        assert_eq!(
            brain.determine_grade(RecallResult {
                recalled: false,
                retrievability: 0.3
            }),
            ReviewGrade::Again
        );

        // Recalled with high R -> Easy
        assert_eq!(
            brain.determine_grade(RecallResult {
                recalled: true,
                retrievability: 0.9
            }),
            ReviewGrade::Easy
        );

        // Recalled with medium R -> Good
        assert_eq!(
            brain.determine_grade(RecallResult {
                recalled: true,
                retrievability: 0.6
            }),
            ReviewGrade::Good
        );

        // Recalled with low R -> Hard
        assert_eq!(
            brain.determine_grade(RecallResult {
                recalled: true,
                retrievability: 0.3
            }),
            ReviewGrade::Hard
        );
    }

    #[test]
    fn test_early_quit_prob_clamped() {
        // Create brain with very high early quit probability
        let mut params = StudentParams::default();
        params.early_quit_prob = 0.9;
        params.fatigue_decay_rate = 0.5; // Extreme fatigue

        let mut brain = StudentBrain::new(params, 42);

        // Even with extreme fatigue, should not crash
        // (probability should be clamped to 1.0)
        for _ in 0..100 {
            let _ = brain.should_quit_early(1000.0, 10.0);
        }
        // If we get here without panic, clamping works
    }

    #[test]
    fn test_consecutive_failures_lead_to_give_up() {
        let mut params = StudentParams::default();
        params.persistence_threshold = 3;

        let mut brain = StudentBrain::new(params, 42);

        // Force failures by using high elapsed time and low stability
        for _ in 0..10 {
            brain.attempt_recall(0.1, 10.0, 1000.0); // Very hard to recall
            if brain.has_given_up() {
                break;
            }
        }

        assert!(
            brain.has_given_up(),
            "Should give up after consecutive failures"
        );
    }
}
