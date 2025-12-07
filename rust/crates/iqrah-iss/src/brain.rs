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

    // ========================================================================
    // 5. SCHEDULING SENSITIVITY (new)
    // ========================================================================
    /// How strongly "should have remembered but failed" affects frustration [0, 1]
    /// Higher = more sensitive to badly-scheduled reviews
    #[serde(default = "default_overdue_sensitivity")]
    pub overdue_sensitivity: f64,

    /// How strongly within-session momentum modulates quit probability [0, 1]
    /// Higher = good/bad streaks have more effect on continuing
    #[serde(default = "default_momentum_sensitivity")]
    pub momentum_sensitivity: f64,
}

// Default functions for serde
fn default_skip_day_prob() -> f64 {
    0.1
}
fn default_early_quit_prob() -> f64 {
    0.05
}
fn default_persistence_threshold() -> u32 {
    200 // Calibrated for |R - R*| penalty logic
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
fn default_overdue_sensitivity() -> f64 {
    0.5
}
fn default_momentum_sensitivity() -> f64 {
    0.4
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
            overdue_sensitivity: default_overdue_sensitivity(),
            momentum_sensitivity: default_momentum_sensitivity(),
        }
    }
}

impl StudentParams {
    /// Create a casual learner archetype.
    ///
    /// Calibrated for |R - R*| penalty logic:
    /// - persistence_threshold=100: with base weight 1.0 per fail, allows ~100 weighted fails
    /// - overdue_sensitivity=0.4: moderate sensitivity to bad scheduling (|R - 0.85|)
    /// - skip_day_prob=0.15: skips ~15% of days
    /// - early_quit_prob=0.05: 5% base quit probability
    pub fn casual_learner() -> Self {
        Self {
            skip_day_prob: 0.15,
            early_quit_prob: 0.05,
            persistence_threshold: 200, // ~200 weighted failures before give-up
            forgetting_rate_mult: 1.2,
            spacing_sensitivity: 0.8,
            item_variability: 0.15,
            fatigue_onset_minutes: 15.0,
            fatigue_decay_rate: 0.04,
            difficulty_sensitivity: 0.5,
            known_surah_ids: vec![1, 112, 113, 114], // Common short surahs
            vocab_known_pct: 0.1,
            overdue_sensitivity: 0.4,  // Sensitivity to |R - R*| penalty
            momentum_sensitivity: 0.5, // Moderate momentum effect
        }
    }

    /// Create a dedicated student archetype.
    ///
    /// Calibrated for |R - R*| penalty logic:
    /// - persistence_threshold=200: very resilient, ~200 weighted fails
    /// - overdue_sensitivity=0.3: low sensitivity to bad scheduling
    /// - skip_day_prob=0.05: rarely skips days
    /// - early_quit_prob=0.02: low base quit probability
    pub fn dedicated_student() -> Self {
        Self {
            skip_day_prob: 0.05,
            early_quit_prob: 0.02,
            persistence_threshold: 350, // Very resilient
            forgetting_rate_mult: 0.9,
            spacing_sensitivity: 1.2,
            item_variability: 0.05,
            fatigue_onset_minutes: 45.0,
            fatigue_decay_rate: 0.01,
            difficulty_sensitivity: 0.1,
            known_surah_ids: (1..=10).collect(), // First 10 surahs
            vocab_known_pct: 0.3,
            overdue_sensitivity: 0.3,  // Low sensitivity to |R - R*| penalty
            momentum_sensitivity: 0.3, // Low momentum effect
        }
    }
}

// ============================================================================
// Heterogeneous Student Populations
// ============================================================================

/// Range specification for parameter variation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamRange {
    /// Uniform distribution [min, max]
    Uniform { min: f64, max: f64 },
    /// Normal distribution with clamping
    Normal {
        mean: f64,
        stddev: f64,
        min: f64,
        max: f64,
    },
}

impl ParamRange {
    /// Sample a value from this range.
    pub fn sample(&self, rng: &mut StdRng) -> f64 {
        use rand_distr::{Distribution, Normal, Uniform};

        match self {
            ParamRange::Uniform { min, max } => {
                let dist = Uniform::new(*min, *max);
                dist.sample(rng)
            }
            ParamRange::Normal {
                mean,
                stddev,
                min,
                max,
            } => {
                let dist = Normal::new(*mean, *stddev)
                    .unwrap_or_else(|_| Normal::new(*mean, 0.1).unwrap());
                dist.sample(rng).clamp(*min, *max)
            }
        }
    }
}

/// Specification for which parameters to vary and how.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParamVariation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_day_prob: Option<ParamRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub early_quit_prob: Option<ParamRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forgetting_rate_mult: Option<ParamRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing_sensitivity: Option<ParamRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_variability: Option<ParamRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fatigue_onset_minutes: Option<ParamRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fatigue_decay_rate: Option<ParamRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty_sensitivity: Option<ParamRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocab_known_pct: Option<ParamRange>,
}

/// Selector for student parameters - either fixed or from a distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum StudentParamsSelector {
    /// Use the same fixed parameters for all students
    Fixed { value: StudentParams },
    /// Sample parameters from distributions
    Distribution {
        base: StudentParams,
        variation: ParamVariation,
    },
}

impl Default for StudentParamsSelector {
    fn default() -> Self {
        StudentParamsSelector::Fixed {
            value: StudentParams::default(),
        }
    }
}

impl StudentParamsSelector {
    /// Create a fixed selector from params.
    pub fn fixed(params: StudentParams) -> Self {
        StudentParamsSelector::Fixed { value: params }
    }

    /// Sample parameters for a specific student.
    ///
    /// The RNG is seeded deterministically from base_seed + student_index.
    pub fn sample_for_student(&self, student_index: usize, base_seed: u64) -> StudentParams {
        match self {
            StudentParamsSelector::Fixed { value } => value.clone(),
            StudentParamsSelector::Distribution { base, variation } => {
                // Use deterministic seed for this student
                let student_seed = base_seed
                    .wrapping_add(student_index as u64)
                    .wrapping_mul(31337);
                let mut rng = StdRng::seed_from_u64(student_seed);

                let mut params = base.clone();

                // Apply variations
                if let Some(range) = &variation.skip_day_prob {
                    params.skip_day_prob = range.sample(&mut rng).clamp(0.0, 1.0);
                }
                if let Some(range) = &variation.early_quit_prob {
                    params.early_quit_prob = range.sample(&mut rng).clamp(0.0, 1.0);
                }
                if let Some(range) = &variation.forgetting_rate_mult {
                    params.forgetting_rate_mult = range.sample(&mut rng).max(0.1);
                }
                if let Some(range) = &variation.spacing_sensitivity {
                    params.spacing_sensitivity = range.sample(&mut rng).max(0.1);
                }
                if let Some(range) = &variation.item_variability {
                    params.item_variability = range.sample(&mut rng).clamp(0.0, 1.0);
                }
                if let Some(range) = &variation.fatigue_onset_minutes {
                    params.fatigue_onset_minutes = range.sample(&mut rng).max(1.0);
                }
                if let Some(range) = &variation.fatigue_decay_rate {
                    params.fatigue_decay_rate = range.sample(&mut rng).clamp(0.0, 1.0);
                }
                if let Some(range) = &variation.difficulty_sensitivity {
                    params.difficulty_sensitivity = range.sample(&mut rng).clamp(0.0, 1.0);
                }
                if let Some(range) = &variation.vocab_known_pct {
                    params.vocab_known_pct = range.sample(&mut rng).clamp(0.0, 1.0);
                }

                params
            }
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

    /// Whether student has given up
    pub given_up: bool,

    // ========================================================================
    // Latent mood/cognitive states (scheduling-sensitive)
    // ========================================================================
    /// Frustration level [0, 1] - longer-term mood affected by surprise failures
    pub frustration: f64,

    /// Weighted failure score - accumulates severity of failures
    /// Higher for "should have remembered" fails, lower for new-item fails
    pub weighted_failure_score: f64,

    /// Successes in current session
    session_successes: u32,

    /// Failures in current session
    session_failures: u32,

    /// Fatigue level [0, 1] - increases with time and difficulty
    fatigue: f64,

    /// Running average difficulty in session (for fatigue calculation)
    avg_difficulty: f64,

    /// Total reviews in current session (for avg calculation)
    session_review_count: u32,
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
            given_up: false,
            // New latent states
            frustration: 0.0,
            weighted_failure_score: 0.0,
            session_successes: 0,
            session_failures: 0,
            fatigue: 0.0,
            avg_difficulty: 5.0, // Start at mid difficulty
            session_review_count: 0,
        }
    }

    /// Get mutable reference to the RNG for external sampling.
    pub fn rng_mut(&mut self) -> &mut StdRng {
        &mut self.rng
    }

    /// Attempt recall of an item using FSRS-aligned retrievability model.
    ///
    /// Now also updates frustration, weighted failures, and session counters
    /// based on whether the failure was "expected" vs "surprise".
    ///
    /// # Arguments
    /// * `stability` - FSRS stability value in days
    /// * `difficulty` - Item difficulty (typically 1.0-10.0)
    /// * `elapsed_days` - Days since last review
    /// * `review_count` - Number of previous reviews (0 = new item)
    pub fn attempt_recall(
        &mut self,
        stability: f64,
        difficulty: f64,
        elapsed_days: f64,
        review_count: u32,
    ) -> RecallResult {
        // 1. Difficulty affects effective stability
        let difficulty_factor = 1.0 + (difficulty - 1.0).max(0.0) * 0.1;
        let eff_stability = (stability / difficulty_factor).max(0.001);

        // 2. Spacing sensitivity affects effective elapsed time
        let eff_elapsed = elapsed_days / self.params.spacing_sensitivity.max(0.1);

        // 3. Compute retrievability R(t) using FSRS power formula
        // This is the "expected" recall probability
        let r = (1.0 + eff_elapsed / (9.0 * eff_stability)).powi(-1);

        // 4. Apply forgetting rate multiplier
        let adjusted_r = r.powf(self.params.forgetting_rate_mult);

        // 5. Add noise
        let noise = self
            .rng
            .gen_range(-self.params.item_variability..self.params.item_variability);
        let final_r = (adjusted_r + noise * 0.1).clamp(0.0, 1.0);

        // 6. Roll for recall
        let roll: f64 = self.rng.gen();
        let recalled = roll < final_r;

        // 7. Update latent states based on outcome and context
        self.update_after_review(r, recalled, review_count, difficulty);

        RecallResult {
            recalled,
            retrievability: final_r,
        }
    }

    /// Update frustration, weighted failure score, and session counters after a review.
    ///
    /// Uses distance from ideal retrievability R* (~0.85) to measure scheduling quality.
    /// A fail near R* (well-timed) is less painful than a fail far from R* (mistimed).
    fn update_after_review(&mut self, r: f64, recalled: bool, _review_count: u32, difficulty: f64) {
        const R_TARGET: f64 = 0.85; // Ideal "sweet spot" retrievability

        // Update running average difficulty
        self.session_review_count += 1;
        let n = self.session_review_count as f64;
        self.avg_difficulty = self.avg_difficulty * (n - 1.0) / n + difficulty / n;

        // Distance from ideal R* (0..1, small if near sweet spot, big if far)
        let r_penalty = (r - R_TARGET).abs();

        if recalled {
            // Success
            self.session_successes += 1;

            // Successful reviews reduce frustration, especially if R is in good band
            let in_band = r_penalty < 0.15; // Within 0.70 - 1.0
            let decay = if in_band { 0.05 } else { 0.02 };
            self.frustration *= 1.0 - decay;

            // Any success gradually reduces weighted failure burden
            self.weighted_failure_score *= 0.9;
        } else {
            // Failure
            self.session_failures += 1;

            // Base pain from failing at all - ANY fail hurts (no "expected fail" escape)
            let base = 1.0;

            // R-distance penalty: failing far from the ideal band is worse
            // overdue_sensitivity now means "sensitivity to bad scheduling"
            let schedule_term = 1.0 + self.params.overdue_sensitivity * r_penalty;

            let weight = base * schedule_term;
            self.weighted_failure_score += weight;

            // Frustration increase depends on scheduling quality
            // Failing far from R* (bad scheduling) causes more frustration
            let delta = self.params.overdue_sensitivity * 0.2 * r_penalty;
            self.frustration = (self.frustration + delta).min(1.0);

            // Give-up condition: weighted failures exceed threshold
            if self.weighted_failure_score > self.params.persistence_threshold as f64 {
                self.given_up = true;
            }
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
    ///
    /// Skip probability increases with frustration.
    pub fn should_skip_day(&mut self) -> bool {
        if self.given_up {
            return true;
        }

        // Frustration increases skip probability (up to +50%)
        let multiplier = 1.0 + self.frustration * 0.5;
        let effective_skip = (self.params.skip_day_prob * multiplier).clamp(0.0, 0.9);

        // Apply natural decay to frustration at day start
        self.frustration *= 0.98;

        let roll: f64 = self.rng.gen();
        roll < effective_skip
    }

    /// Check if student should quit early during a session.
    ///
    /// Uses fatigue (time + difficulty), momentum (session success rate), and frustration.
    ///
    /// # Arguments
    /// * `minutes_elapsed` - Minutes spent in current session
    pub fn should_quit_early(&mut self, minutes_elapsed: f64) -> bool {
        if self.given_up {
            return true;
        }

        // 1. Update fatigue based on time and difficulty
        self.update_fatigue(minutes_elapsed);

        // 2. Compute session momentum (success rate)
        let total = self.session_successes + self.session_failures;
        let success_rate = if total == 0 {
            1.0 // Optimistic at start
        } else {
            self.session_successes as f64 / total as f64
        };

        // Momentum effect: success_rate > 0.5 reduces quit prob, < 0.5 increases it
        let centered = success_rate - 0.5; // -0.5 to 0.5
        let momentum_effect = 1.0 - self.params.momentum_sensitivity * centered;

        // 3. Compute effective quit probability
        let mut quit_prob = self.params.early_quit_prob;

        // Apply fatigue (up to 3x multiplier per spec)
        quit_prob *= (1.0 + self.fatigue).min(3.0);

        // Apply momentum
        quit_prob *= momentum_effect.clamp(0.3, 2.0);

        // Apply frustration (up to +50%)
        quit_prob *= 1.0 + self.frustration * 0.5;

        quit_prob = quit_prob.clamp(0.0, 1.0);

        let roll: f64 = self.rng.gen();
        roll < quit_prob
    }

    /// Update fatigue based on time in session and average difficulty.
    fn update_fatigue(&mut self, minutes_elapsed: f64) {
        // Time-based fatigue (linear increase after onset)
        let over_minutes = (minutes_elapsed - self.params.fatigue_onset_minutes).max(0.0);
        let time_fatigue = over_minutes * self.params.fatigue_decay_rate;

        // Difficulty-based component (only extra fatigue for hard sessions)
        let difficulty_center = 5.0;
        let diff_delta = (self.avg_difficulty - difficulty_center) / difficulty_center;
        let diff_fatigue = self.params.difficulty_sensitivity * diff_delta.max(0.0);

        let raw = time_fatigue + diff_fatigue;

        // Smooth update to avoid harsh jumps
        self.fatigue = (self.fatigue * 0.8 + raw * 0.2).clamp(0.0, 1.0);
    }

    /// Reset session counters at start of a new session/day.
    pub fn start_session(&mut self) {
        self.session_successes = 0;
        self.session_failures = 0;
        self.session_review_count = 0;
        self.avg_difficulty = 5.0;
        self.fatigue = 0.0;
    }

    /// Notify brain that a new day has started.
    ///
    /// Applies daily decay to weighted_failure_score and frustration,
    /// allowing recovery from bad periods if scheduling improves.
    pub fn start_day(&mut self) {
        self.days_active += 1;

        // Daily decay: allows past pain to fade if scheduler improves
        self.weighted_failure_score *= 0.99;
        self.frustration *= 0.98;

        self.start_session();
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

            let r1 = b1.attempt_recall(stability, difficulty, 1.0, 5); // 1 day elapsed
            let r2 = b2.attempt_recall(stability, difficulty, 30.0, 5); // 30 days elapsed

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

            let r_easy = brain_easy.attempt_recall(10.0, 1.0, 5.0, 5); // Low difficulty
            let r_hard = brain_hard.attempt_recall(10.0, 8.0, 5.0, 5); // High difficulty

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
            let _ = brain.should_quit_early(1000.0);
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
            brain.attempt_recall(0.1, 10.0, 1000.0, 5); // Very hard to recall
            if brain.has_given_up() {
                break;
            }
        }

        assert!(
            brain.has_given_up(),
            "Should give up after consecutive failures"
        );
    }

    #[test]
    fn test_student_params_selector_fixed() {
        let params = StudentParams::default();
        let selector = StudentParamsSelector::fixed(params.clone());

        // Fixed selector should return the same params for all students
        let p1 = selector.sample_for_student(0, 42);
        let p2 = selector.sample_for_student(1, 42);
        let p3 = selector.sample_for_student(100, 42);

        assert_eq!(p1.skip_day_prob, params.skip_day_prob);
        assert_eq!(p2.skip_day_prob, params.skip_day_prob);
        assert_eq!(p3.skip_day_prob, params.skip_day_prob);
    }

    #[test]
    fn test_student_params_selector_distribution() {
        let selector = StudentParamsSelector::Distribution {
            base: StudentParams::default(),
            variation: ParamVariation {
                skip_day_prob: Some(ParamRange::Uniform { min: 0.1, max: 0.5 }),
                forgetting_rate_mult: Some(ParamRange::Normal {
                    mean: 1.0,
                    stddev: 0.2,
                    min: 0.5,
                    max: 1.5,
                }),
                ..Default::default()
            },
        };

        // Different students should get different params
        let p1 = selector.sample_for_student(0, 42);
        let p2 = selector.sample_for_student(1, 42);
        let p3 = selector.sample_for_student(2, 42);

        // Sampled values should be within range
        assert!(p1.skip_day_prob >= 0.1 && p1.skip_day_prob <= 0.5);
        assert!(p2.skip_day_prob >= 0.1 && p2.skip_day_prob <= 0.5);
        assert!(p3.skip_day_prob >= 0.1 && p3.skip_day_prob <= 0.5);

        // Different students should have different values (very unlikely to be equal)
        assert!(p1.skip_day_prob != p2.skip_day_prob || p2.skip_day_prob != p3.skip_day_prob);
    }

    #[test]
    fn test_student_params_selector_deterministic() {
        let selector = StudentParamsSelector::Distribution {
            base: StudentParams::default(),
            variation: ParamVariation {
                skip_day_prob: Some(ParamRange::Uniform { min: 0.1, max: 0.5 }),
                ..Default::default()
            },
        };

        // Same student_index and seed should produce same params
        let p1a = selector.sample_for_student(5, 42);
        let p1b = selector.sample_for_student(5, 42);

        assert_eq!(p1a.skip_day_prob, p1b.skip_day_prob);
        assert_eq!(p1a.early_quit_prob, p1b.early_quit_prob);
    }

    #[test]
    fn test_param_range_uniform() {
        let range = ParamRange::Uniform {
            min: 10.0,
            max: 20.0,
        };
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..100 {
            let val = range.sample(&mut rng);
            assert!(val >= 10.0 && val < 20.0, "Value {} out of range", val);
        }
    }

    #[test]
    fn test_param_range_normal() {
        let range = ParamRange::Normal {
            mean: 1.0,
            stddev: 0.2,
            min: 0.5,
            max: 1.5,
        };
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..100 {
            let val = range.sample(&mut rng);
            assert!(
                val >= 0.5 && val <= 1.5,
                "Value {} out of clamped range",
                val
            );
        }
    }

    // ========================================================================
    // Scheduling Sensitivity Tests
    // ========================================================================

    #[test]
    fn test_overdue_fail_increases_frustration() {
        let mut brain = StudentBrain::new(StudentParams::default(), 42);
        let initial_frustration = brain.frustration;

        // Overdue failure: low R (item was left to rot), mature item
        brain.update_after_review(0.2, false, 5, 5.0); // R=0.2, failed, mature item

        assert!(
            brain.frustration > initial_frustration,
            "Overdue failure should increase frustration: {} vs {}",
            brain.frustration,
            initial_frustration
        );
    }

    #[test]
    fn test_well_timed_fail_minimal_frustration() {
        let mut brain = StudentBrain::new(StudentParams::default(), 42);
        let initial_frustration = brain.frustration;

        // Well-timed failure: high R (item scheduled correctly), failure is acceptable
        brain.update_after_review(0.85, false, 5, 5.0); // R=0.85, failed, mature item

        // Frustration should not increase for well-timed failures (good scheduling)
        assert!(
            brain.frustration <= initial_frustration + 0.01,
            "Well-timed failure should not significantly increase frustration"
        );
    }

    #[test]
    fn test_weighted_failure_overdue_heavier() {
        let mut brain1 = StudentBrain::new(StudentParams::default(), 42);
        let mut brain2 = StudentBrain::new(StudentParams::default(), 42);

        // Overdue failure (low R, mature item) - bad scheduling
        brain1.update_after_review(0.2, false, 10, 5.0);

        // Well-timed failure (high R, mature item) - good scheduling
        brain2.update_after_review(0.85, false, 10, 5.0);

        assert!(
            brain1.weighted_failure_score > brain2.weighted_failure_score,
            "Overdue failure should have heavier weight: {} vs {}",
            brain1.weighted_failure_score,
            brain2.weighted_failure_score
        );
    }

    #[test]
    fn test_session_momentum_tracked() {
        let mut brain = StudentBrain::new(StudentParams::default(), 42);

        brain.update_after_review(0.5, true, 5, 5.0); // Success
        brain.update_after_review(0.5, true, 5, 5.0); // Success
        brain.update_after_review(0.5, false, 5, 5.0); // Failure

        assert_eq!(brain.session_successes, 2);
        assert_eq!(brain.session_failures, 1);
    }

    #[test]
    fn test_start_session_resets_counters() {
        let mut brain = StudentBrain::new(StudentParams::default(), 42);

        brain.update_after_review(0.5, true, 5, 5.0);
        brain.update_after_review(0.5, false, 5, 5.0);

        brain.start_session();

        assert_eq!(brain.session_successes, 0);
        assert_eq!(brain.session_failures, 0);
        assert_eq!(brain.fatigue, 0.0);
    }

    #[test]
    fn test_frustration_affects_skip_probability() {
        // High frustration should increase effective skip probability
        let mut high_frust = StudentBrain::new(StudentParams::default(), 42);
        high_frust.frustration = 0.9;

        let mut low_frust = StudentBrain::new(StudentParams::default(), 42);
        low_frust.frustration = 0.0;

        // Count skips over many trials
        let trials = 1000;
        let mut high_skips = 0;
        let mut low_skips = 0;

        for i in 0..trials {
            let mut hf = StudentBrain::new(StudentParams::default(), 1000 + i);
            hf.frustration = 0.9;
            if hf.should_skip_day() {
                high_skips += 1;
            }

            let mut lf = StudentBrain::new(StudentParams::default(), 1000 + i);
            lf.frustration = 0.0;
            if lf.should_skip_day() {
                low_skips += 1;
            }
        }

        assert!(
            high_skips > low_skips,
            "High frustration should cause more skips: {} vs {}",
            high_skips,
            low_skips
        );
    }

    #[test]
    fn test_good_momentum_reduces_quit_prob() {
        // Good momentum (high success rate) should reduce quit probability
        let mut params = StudentParams::default();
        params.early_quit_prob = 0.2;
        params.momentum_sensitivity = 0.5;

        let trials = 1000;
        let mut good_momentum_quits = 0;
        let mut bad_momentum_quits = 0;

        for i in 0..trials {
            // Good momentum: many successes
            let mut good = StudentBrain::new(params.clone(), 2000 + i);
            good.session_successes = 10;
            good.session_failures = 1;
            if good.should_quit_early(10.0) {
                good_momentum_quits += 1;
            }

            // Bad momentum: many failures
            let mut bad = StudentBrain::new(params.clone(), 2000 + i);
            bad.session_successes = 1;
            bad.session_failures = 10;
            if bad.should_quit_early(10.0) {
                bad_momentum_quits += 1;
            }
        }

        assert!(
            good_momentum_quits < bad_momentum_quits,
            "Good momentum should result in fewer quits: {} vs {}",
            good_momentum_quits,
            bad_momentum_quits
        );
    }

    #[test]
    fn test_bad_momentum_increases_quit_prob() {
        // Same as above but verifying the inverse
        let mut params = StudentParams::default();
        params.early_quit_prob = 0.2;
        params.momentum_sensitivity = 0.6;

        let mut brain = StudentBrain::new(params.clone(), 42);
        brain.session_successes = 1;
        brain.session_failures = 20; // Very bad session

        // Success rate = 1/21 ≈ 0.05, centered = -0.45
        // momentum_effect = 1.0 - 0.6 * (-0.45) = 1.27 → increases quit prob
        let total = brain.session_successes + brain.session_failures;
        let success_rate = brain.session_successes as f64 / total as f64;
        assert!(success_rate < 0.1, "Should have low success rate");

        let centered = success_rate - 0.5;
        let momentum_effect = 1.0 - params.momentum_sensitivity * centered;
        assert!(
            momentum_effect > 1.0,
            "Bad momentum should increase quit prob multiplier: {}",
            momentum_effect
        );
    }

    #[test]
    fn test_fatigue_increases_with_time_and_difficulty() {
        let mut brain = StudentBrain::new(StudentParams::default(), 42);

        // Start with zero fatigue
        assert_eq!(brain.fatigue, 0.0);

        // Simulate a long session with high difficulty
        brain.avg_difficulty = 8.0; // Above center of 5.0
        brain.update_fatigue(60.0); // 60 minutes, well past fatigue_onset (default 20)

        // Fatigue should have increased
        assert!(
            brain.fatigue > 0.0,
            "Fatigue should increase with long session: {}",
            brain.fatigue
        );

        // Higher difficulty should cause more fatigue
        let mut brain_easy = StudentBrain::new(StudentParams::default(), 42);
        brain_easy.avg_difficulty = 3.0; // Below center
        brain_easy.update_fatigue(60.0);

        let mut brain_hard = StudentBrain::new(StudentParams::default(), 42);
        brain_hard.avg_difficulty = 8.0; // Above center
        brain_hard.update_fatigue(60.0);

        assert!(
            brain_hard.fatigue >= brain_easy.fatigue,
            "Hard session should cause more fatigue: {} vs {}",
            brain_hard.fatigue,
            brain_easy.fatigue
        );
    }
}
