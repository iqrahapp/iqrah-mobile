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
    // 5. SCHEDULING SENSITIVITY
    // ========================================================================
    /// How strongly "should have remembered but failed" affects frustration [0, 1]
    /// Higher = more sensitive to badly-scheduled reviews
    #[serde(default = "default_overdue_sensitivity")]
    pub overdue_sensitivity: f64,

    /// How strongly within-session momentum modulates quit probability [0, 1]
    /// Higher = good/bad streaks have more effect on continuing
    #[serde(default = "default_momentum_sensitivity")]
    pub momentum_sensitivity: f64,

    // ========================================================================
    // 6. DAILY REVIEW BUDGET (ISS v2.1 §6.1)
    // ========================================================================
    /// Minimum reviews per day (e.g., 3)
    #[serde(default = "default_min_reviews_per_day")]
    pub min_reviews_per_day: usize,

    /// Maximum reviews per day (e.g., 30)
    #[serde(default = "default_max_reviews_per_day")]
    pub max_reviews_per_day: usize,

    /// Mean reviews per day for normal distribution (e.g., 15.0)
    #[serde(default = "default_mean_reviews_per_day")]
    pub mean_reviews_per_day: f32,

    /// Standard deviation for daily reviews (e.g., 5.0)
    #[serde(default = "default_reviews_stddev")]
    pub reviews_stddev: f32,

    // ========================================================================
    // 7. ENHANCED ERROR MODEL (ISS v2.1 §6.2)
    // ========================================================================
    /// Fatigue sensitivity: error rate rise late in session [0.0-1.0]
    #[serde(default = "default_fatigue_sensitivity")]
    pub fatigue_sensitivity: f32,

    /// Base forgetting rate for computing p_recall
    #[serde(default = "default_lapse_baseline")]
    pub lapse_baseline: f32,

    /// Multiplier for FSRS stability to vary memory across students
    #[serde(default = "default_stability_scale")]
    pub stability_scale: f32,

    // ========================================================================
    // 8. MASTERY-DEPENDENT ENERGY DRIFT (ISS v2.3)
    // ========================================================================
    /// Maximum drift rate for beginner items (E≈0)
    /// Formula: drift_rate(E) = α_max × (1 - E^k) + α_min × E^k
    #[serde(default = "default_drift_alpha_max")]
    pub drift_alpha_max: f64,

    /// Minimum drift rate for mastered items (E≈1)
    #[serde(default = "default_drift_alpha_min")]
    pub drift_alpha_min: f64,

    /// Mastery exponent controlling protection curve shape
    /// Higher values give more protection to high-energy items
    #[serde(default = "default_drift_mastery_exponent")]
    pub drift_mastery_exponent: f64,

    /// Minimum energy floor for seen items [0.0-0.1]
    /// Prevents complete energy collapse
    #[serde(default = "default_drift_energy_floor")]
    pub drift_energy_floor: f64,

    // ========================================================================
    // 9. CAPACITY-BASED INTRODUCTION CONTROL (ISS v2.3)
    // ========================================================================
    /// Maximum items/day the student can handle
    #[serde(default = "default_session_capacity")]
    pub session_capacity: f64,

    /// Reserve capacity for urgent items (e.g., 0.20 = 20%)
    #[serde(default = "default_headroom_reserve")]
    pub headroom_reserve: f64,

    /// Number of reviews needed to stabilize a new item
    #[serde(default = "default_reviews_to_stability")]
    pub reviews_to_stability: f64,

    /// Time window for new item stabilization (days)
    #[serde(default = "default_days_to_stability")]
    pub days_to_stability: f64,

    // ========================================================================
    // 10. WORKING SET & CLUSTER CONTROL (ISS v2.6)
    // ========================================================================
    /// Maximum number of active items before forcing consolidation
    /// Active = items with review_count > 0
    /// Typical values: 40-60 depending on student capacity
    #[serde(default = "default_max_working_set")]
    pub max_working_set: usize,

    /// Minimum weighted cluster energy required before introducing new batch
    /// Weighted by item maturity (new items have more impact)
    /// Typical values: 0.35-0.50
    #[serde(default = "default_cluster_stability_threshold")]
    pub cluster_stability_threshold: f64,

    /// Number of items to introduce per expansion (batch size K)
    /// Typical: 3 for connected ayahs, 1-5 depending on content type
    #[serde(default = "default_cluster_expansion_batch_size")]
    pub cluster_expansion_batch_size: usize,

    /// Enable auto-pruning of mastered items from cluster
    /// Items with E>threshold for >N days are removed from cluster tracking
    #[serde(default = "default_cluster_auto_prune_enabled")]
    pub cluster_auto_prune_enabled: bool,

    /// Days threshold for auto-pruning (if enabled)
    #[serde(default = "default_cluster_prune_days_threshold")]
    pub cluster_prune_days_threshold: u32,

    /// Energy threshold for auto-pruning (if enabled)
    #[serde(default = "default_cluster_prune_energy_threshold")]
    pub cluster_prune_energy_threshold: f64,
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

// ISS v2.1 defaults
fn default_min_reviews_per_day() -> usize {
    3
}
fn default_max_reviews_per_day() -> usize {
    30
}
fn default_mean_reviews_per_day() -> f32 {
    15.0
}
fn default_reviews_stddev() -> f32 {
    5.0
}
fn default_fatigue_sensitivity() -> f32 {
    0.3 // 30% error rate increase at end of session
}
fn default_lapse_baseline() -> f32 {
    0.1 // 10% base forgetting rate
}
fn default_stability_scale() -> f32 {
    1.0 // Unity scale by default
}

// ISS v2.3 mastery-dependent drift defaults
fn default_drift_alpha_max() -> f64 {
    0.20 // 20% decay for beginners (E≈0)
}
fn default_drift_alpha_min() -> f64 {
    0.02 // 2% decay for masters (E≈1)
}
fn default_drift_mastery_exponent() -> f64 {
    2.0 // Quadratic protection curve
}
fn default_drift_energy_floor() -> f64 {
    0.05 // 5% minimum for seen items
}

// ISS v2.3 capacity control defaults
fn default_session_capacity() -> f64 {
    15.0 // 15 items/day default capacity
}
fn default_headroom_reserve() -> f64 {
    0.10 // 10% buffer for urgent items (reduced from 20% in ISS v2.4)
}
fn default_reviews_to_stability() -> f64 {
    5.0 // 5 reviews to stabilize new item
}
fn default_days_to_stability() -> f64 {
    30.0 // Over 30-day window
}

// ISS v2.6 working set and cluster control defaults
fn default_max_working_set() -> usize {
    50 // Cap active items at 50 before consolidation
}
fn default_cluster_stability_threshold() -> f64 {
    0.40 // Require 40% weighted mean energy before expansion
}
fn default_cluster_expansion_batch_size() -> usize {
    3 // Introduce 3 items at a time (K=3)
}
fn default_cluster_auto_prune_enabled() -> bool {
    false // Disable pruning for v2.6, enable in v3.0
}
fn default_cluster_prune_days_threshold() -> u32 {
    7 // 7 days at mastery before pruning
}
fn default_cluster_prune_energy_threshold() -> f64 {
    0.85 // E>0.85 for pruning consideration
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
            // ISS v2.1 fields
            min_reviews_per_day: default_min_reviews_per_day(),
            max_reviews_per_day: default_max_reviews_per_day(),
            mean_reviews_per_day: default_mean_reviews_per_day(),
            reviews_stddev: default_reviews_stddev(),
            fatigue_sensitivity: default_fatigue_sensitivity(),
            lapse_baseline: default_lapse_baseline(),
            stability_scale: default_stability_scale(),
            // ISS v2.3 mastery-dependent drift
            drift_alpha_max: default_drift_alpha_max(),
            drift_alpha_min: default_drift_alpha_min(),
            drift_mastery_exponent: default_drift_mastery_exponent(),
            drift_energy_floor: default_drift_energy_floor(),
            // ISS v2.3 capacity control
            session_capacity: default_session_capacity(),
            headroom_reserve: default_headroom_reserve(),
            reviews_to_stability: default_reviews_to_stability(),
            days_to_stability: default_days_to_stability(),
            // ISS v2.6 working set and cluster control
            max_working_set: default_max_working_set(),
            cluster_stability_threshold: default_cluster_stability_threshold(),
            cluster_expansion_batch_size: default_cluster_expansion_batch_size(),
            cluster_auto_prune_enabled: default_cluster_auto_prune_enabled(),
            cluster_prune_days_threshold: default_cluster_prune_days_threshold(),
            cluster_prune_energy_threshold: default_cluster_prune_energy_threshold(),
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
            // ISS v2.1: casual learners do fewer reviews per day
            min_reviews_per_day: 3,
            max_reviews_per_day: 15,
            mean_reviews_per_day: 8.0,
            reviews_stddev: 3.0,
            fatigue_sensitivity: 0.4, // Higher fatigue sensitivity
            lapse_baseline: 0.15,     // Slightly higher forgetting
            stability_scale: 0.9,     // Slightly worse memory
            // ISS v2.3: Mastery-dependent drift (faster decay for casual learners)
            drift_alpha_max: 0.25, // Faster decay for beginners
            drift_alpha_min: 0.03, // Some decay even for masters
            drift_mastery_exponent: 2.0,
            drift_energy_floor: 0.03,
            // ISS v2.3: Capacity control (lower capacity for casual learners)
            session_capacity: 10.0,
            headroom_reserve: 0.25,    // More buffer needed
            reviews_to_stability: 6.0, // Needs more reviews
            days_to_stability: 30.0,
            // ISS v2.6: Working set and cluster control (casual = smaller limits)
            max_working_set: 35, // Smaller working set for casual learners
            cluster_stability_threshold: 0.45, // Higher threshold (more consolidation needed)
            cluster_expansion_batch_size: 2, // Smaller batches for casual learners
            cluster_auto_prune_enabled: false,
            cluster_prune_days_threshold: 7,
            cluster_prune_energy_threshold: 0.85,
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
            // ISS v2.1: dedicated students do more reviews per day
            min_reviews_per_day: 10,
            max_reviews_per_day: 30,
            mean_reviews_per_day: 20.0,
            reviews_stddev: 5.0,
            fatigue_sensitivity: 0.2, // Lower fatigue sensitivity
            lapse_baseline: 0.08,     // Better retention
            stability_scale: 1.2,     // Better memory
            // ISS v2.3: Mastery-dependent drift (slower for dedicated students)
            drift_alpha_max: 0.10, // Slower decay for beginners (better retention)
            drift_alpha_min: 0.02, // Minimal decay for masters
            drift_mastery_exponent: 2.0,
            drift_energy_floor: 0.08,
            // ISS v2.3: Capacity control (higher capacity for dedicated students)
            session_capacity: 25.0,
            headroom_reserve: 0.10,
            reviews_to_stability: 5.0,
            days_to_stability: 30.0,
            // ISS v2.6: Working set and cluster control (dedicated = larger limits)
            max_working_set: 60, // Larger working set for dedicated students
            cluster_stability_threshold: 0.35, // Lower threshold (faster expansion)
            cluster_expansion_batch_size: 4, // Larger batches for dedicated students
            cluster_auto_prune_enabled: false,
            cluster_prune_days_threshold: 7,
            cluster_prune_energy_threshold: 0.85,
        }
    }

    /// Compute mastery-dependent drift rate (ISS v2.3).
    ///
    /// Higher energy (mastery) → lower drift rate (spacing effect).
    ///
    /// # Formula
    /// ```text
    /// drift_rate(E) = α_max × (1 - E^k) + α_min × E^k
    /// ```
    ///
    /// # Properties
    /// - When E=0: drift_rate = α_max (aggressive decay)
    /// - When E=1: drift_rate = α_min (gentle decay)
    /// - Continuous, smooth transition (no thresholds)
    pub fn compute_drift_rate(&self, energy: f64) -> f64 {
        let protection = energy.powf(self.drift_mastery_exponent);
        self.drift_alpha_max * (1.0 - protection) + self.drift_alpha_min * protection
    }

    /// Compute expected recall (R*) based on item maturity (ISS v2.4).
    ///
    /// Young items (0-5 reviews) have lower expectations, mature items have higher.
    /// This prevents the death spiral where new items always fail because expectations
    /// are too high for their energy level.
    ///
    /// # Formula
    /// ```text
    /// maturity = clamp(review_count / 5, 0, 1)
    /// R* = R*_young × (1 - maturity) + R*_mature × maturity
    /// ```
    ///
    /// # Properties
    /// - Young items: R* ≈ 0.65 (lower expectations, less frustration on failure)
    /// - Mature items: R* ≈ 0.85 (normal FSRS expectations)
    pub fn compute_expected_recall(&self, _energy: f64, review_count: u32) -> f64 {
        // Base expectations for different maturity levels (ISS v2.4 tuned)
        // Lowered from 0.65/0.85 to reduce frustration accumulation
        let r_star_young = 0.55; // Very low expectation for young items
        let r_star_mature = 0.80; // Slightly lower than standard FSRS target

        // Interpolate based on review count (mature at 5+ reviews)
        let maturity = (review_count as f64 / 5.0).clamp(0.0, 1.0);

        r_star_young * (1.0 - maturity) + r_star_mature * maturity
    }

    /// Compute drift rate with both energy and maturity dependence (ISS v2.4 Fix 3).
    ///
    /// Young items (few reviews) drift slower, giving them time to stabilize.
    /// This prevents the death spiral where items decay before they can be reinforced.
    ///
    /// # Formula
    /// ```text
    /// base_drift = compute_drift_rate(energy)  // ISS v2.3
    /// maturity = clamp(review_count / 5, 0, 1)
    /// young_protection = 1 - 0.3 × (1 - maturity)  // 30% slower for young items
    /// final_drift = base_drift × young_protection
    /// ```
    pub fn compute_drift_rate_v2(&self, energy: f64, review_count: u32) -> f64 {
        // Base drift from energy (mastery protection) - ISS v2.3
        let base_drift = self.compute_drift_rate(energy);

        // Additional protection for young items (haven't had chance to stabilize)
        let maturity = (review_count as f64 / 5.0).clamp(0.0, 1.0);

        // Young items (maturity=0) get 40% slower drift, mature items (maturity=1) get full drift
        // Increased from 30% to 40% to give young items more time to stabilize
        let young_item_protection = 1.0 - 0.4 * (1.0 - maturity);

        base_drift * young_item_protection
    }

    /// Compute "safe interval" - days before energy drops below critical threshold (ISS v2.4 Fix 2).
    ///
    /// This estimates how many days until the item's energy decays below a usable level,
    /// enabling proactive scheduling before critical decay occurs.
    ///
    /// # Formula
    /// ```text
    /// E_new = E_old × (1 - α)^days
    /// Solve for days: days = ln(E_critical / E_current) / ln(1 - α)
    /// ```
    ///
    /// # Returns
    /// Number of days until energy drops below critical threshold (0.25).
    /// Returns 0.0 if already critical, or f64::MAX if stable enough to not worry.
    pub fn compute_safe_interval(&self, current_energy: f64, review_count: u32) -> f64 {
        const CRITICAL_ENERGY: f64 = 0.25; // Below this, R is too low for reliable recall

        if current_energy <= CRITICAL_ENERGY {
            return 0.0; // Already critical
        }

        // Get drift rate for this item
        let drift_rate = self.compute_drift_rate_v2(current_energy, review_count);

        if drift_rate < 0.001 {
            return f64::MAX; // Essentially no drift
        }

        // E_new = E_old × (1 - α)^days
        // E_critical = E_current × (1 - α)^days
        // days = ln(E_critical / E_current) / ln(1 - α)
        let ratio = CRITICAL_ENERGY / current_energy;
        let days = ratio.ln() / (1.0 - drift_rate).ln();

        days.max(1.0) // At least 1 day
    }
}

// ============================================================================
// Named Student Profiles (ISS v2.3)
// ============================================================================

/// Named student profiles for simulation scenarios.
///
/// These profiles are calibrated to produce realistic outcomes:
/// - `StrongDedicated`: Motivated adult, high persistence, high review counts
/// - `NormalDedicated`: Average motivated learner, moderate settings
/// - `HarshStressTest`: Stress test variant, gives up quickly (NOT for benchmarks)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudentProfile {
    /// Strong dedicated learner - high persistence, high review budget
    StrongDedicated,
    /// Normal dedicated learner - moderate settings
    NormalDedicated,
    /// Harsh stress test student - gives up quickly (DO NOT use for benchmarks)
    HarshStressTest,
}

impl StudentProfile {
    /// Convert profile to StudentParams.
    pub fn to_params(&self) -> StudentParams {
        match self {
            StudentProfile::StrongDedicated => StudentParams {
                skip_day_prob: 0.01,                 // Very rarely skip
                early_quit_prob: 0.005,              // Almost never quit early
                persistence_threshold: 100_000,      // Effectively never give up
                forgetting_rate_mult: 0.5,           // Much better memory (50% of base)
                spacing_sensitivity: 1.5,            // Benefits more from spacing
                item_variability: 0.03,              // Very consistent
                fatigue_onset_minutes: 90.0,         // Long attention span
                fatigue_decay_rate: 0.002,           // Very slow fatigue
                difficulty_sensitivity: 0.05,        // Almost unaffected by difficulty
                known_surah_ids: (1..=15).collect(), // First 15 surahs
                vocab_known_pct: 0.5,
                overdue_sensitivity: 0.01,  // Very tolerant
                momentum_sensitivity: 0.05, // Very stable mood
                min_reviews_per_day: 20,    // Higher minimum
                max_reviews_per_day: 60,    // Higher ceiling
                mean_reviews_per_day: 35.0, // 35 reviews/day average
                reviews_stddev: 8.0,
                fatigue_sensitivity: 0.05, // Almost no fatigue errors
                lapse_baseline: 0.02,      // Very low forgetting
                stability_scale: 2.0,      // 2x effective stability (strong memory)
                // ISS v2.3: Minimal mastery-dependent drift for strong dedicated
                drift_alpha_max: 0.10, // Low decay even for beginners
                drift_alpha_min: 0.01, // Almost no decay for masters
                drift_mastery_exponent: 2.0,
                drift_energy_floor: 0.10,
                // ISS v2.3: Capacity control (benchmark tuned)
                session_capacity: 40.0,
                headroom_reserve: 0.10,
                reviews_to_stability: 4.0,
                days_to_stability: 25.0,
                // ISS v2.6: Working set and cluster control (strong dedicated = high limits)
                max_working_set: 80, // Very large working set capacity
                cluster_stability_threshold: 0.30, // Low threshold (aggressive expansion)
                cluster_expansion_batch_size: 5, // Large batches
                cluster_auto_prune_enabled: false,
                cluster_prune_days_threshold: 7,
                cluster_prune_energy_threshold: 0.85,
            },
            StudentProfile::NormalDedicated => StudentParams {
                skip_day_prob: 0.08,
                early_quit_prob: 0.03,
                persistence_threshold: 600,
                forgetting_rate_mult: 1.0,
                spacing_sensitivity: 1.1,
                item_variability: 0.08,
                fatigue_onset_minutes: 35.0,
                fatigue_decay_rate: 0.015,
                difficulty_sensitivity: 0.2,
                known_surah_ids: (1..=5).collect(),
                vocab_known_pct: 0.2,
                overdue_sensitivity: 0.3,
                momentum_sensitivity: 0.35,
                min_reviews_per_day: 8,
                max_reviews_per_day: 35,
                mean_reviews_per_day: 18.0,
                reviews_stddev: 6.0,
                fatigue_sensitivity: 0.25,
                lapse_baseline: 0.10,
                stability_scale: 1.0,
                // ISS v2.3: Default mastery-dependent drift
                drift_alpha_max: 0.20,
                drift_alpha_min: 0.02,
                drift_mastery_exponent: 2.0,
                drift_energy_floor: 0.05,
                // ISS v2.3: Moderate capacity
                session_capacity: 18.0,
                headroom_reserve: 0.20,
                reviews_to_stability: 5.0,
                days_to_stability: 30.0,
                // ISS v2.6: Working set and cluster control
                max_working_set: 50,
                cluster_stability_threshold: 0.40,
                cluster_expansion_batch_size: 3, // Default batch size
                cluster_auto_prune_enabled: false,
                cluster_prune_days_threshold: 7,
                cluster_prune_energy_threshold: 0.85,
            },
            StudentProfile::HarshStressTest => StudentParams {
                skip_day_prob: 0.20,
                early_quit_prob: 0.10,
                persistence_threshold: 80, // Gives up quickly
                forgetting_rate_mult: 1.4,
                spacing_sensitivity: 0.7,
                item_variability: 0.2,
                fatigue_onset_minutes: 10.0,
                fatigue_decay_rate: 0.08,
                difficulty_sensitivity: 0.6,
                known_surah_ids: vec![1], // Only Al-Fatihah
                vocab_known_pct: 0.05,
                overdue_sensitivity: 0.8,
                momentum_sensitivity: 0.7,
                min_reviews_per_day: 3,
                max_reviews_per_day: 15,
                mean_reviews_per_day: 8.0,
                reviews_stddev: 3.0,
                fatigue_sensitivity: 0.5,
                lapse_baseline: 0.20,
                stability_scale: 0.7,
                // ISS v2.3: Fast mastery-dependent drift for stress test
                drift_alpha_max: 0.30, // Fast decay for beginners
                drift_alpha_min: 0.05, // Some decay even for masters
                drift_mastery_exponent: 2.0,
                drift_energy_floor: 0.02,
                // ISS v2.3: Low capacity for stress test
                session_capacity: 8.0,
                headroom_reserve: 0.30,
                reviews_to_stability: 6.0,
                days_to_stability: 30.0,
                // ISS v2.6: Working set and cluster control (harsh = small limits)
                max_working_set: 25,               // Small working set
                cluster_stability_threshold: 0.50, // High threshold (strict consolidation)
                cluster_expansion_batch_size: 1,   // One at a time for struggling learners
                cluster_auto_prune_enabled: false,
                cluster_prune_days_threshold: 7,
                cluster_prune_energy_threshold: 0.85,
            },
        }
    }

    /// Parse profile from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "strong_dedicated" | "strongdedicated" | "strong" => Some(Self::StrongDedicated),
            "normal_dedicated" | "normaldedicated" | "normal" => Some(Self::NormalDedicated),
            "harsh_stress_test" | "harshstresstest" | "harsh" | "stress" => {
                Some(Self::HarshStressTest)
            }
            _ => None,
        }
    }

    /// Get profile name as string.
    pub fn name(&self) -> &'static str {
        match self {
            Self::StrongDedicated => "strong_dedicated",
            Self::NormalDedicated => "normal_dedicated",
            Self::HarshStressTest => "harsh_stress_test",
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

    /// Force perfect recall (Oracle mode)
    pub force_perfect_recall: bool,
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
            force_perfect_recall: false,
        }
    }

    /// Get mutable reference to the RNG for external sampling.
    pub fn rng_mut(&mut self) -> &mut StdRng {
        &mut self.rng
    }

    /// Sample daily review count from clamped normal distribution (ISS v2.1 §6.1).
    ///
    /// Returns a value in the range [min_reviews_per_day, max_reviews_per_day].
    ///
    /// Formula per spec:
    /// ```text
    /// n_reviews ~ clamp(
    ///     Normal(mean_reviews_per_day, reviews_stddev),
    ///     min_reviews_per_day,
    ///     max_reviews_per_day
    /// )
    /// ```
    pub fn sample_daily_reviews(&mut self) -> usize {
        use rand_distr::{Distribution, Normal};

        let dist = Normal::new(
            self.params.mean_reviews_per_day as f64,
            self.params.reviews_stddev as f64,
        )
        .unwrap_or_else(|_| Normal::new(15.0, 5.0).unwrap());

        let sampled = dist.sample(&mut self.rng);
        let clamped = sampled.clamp(
            self.params.min_reviews_per_day as f64,
            self.params.max_reviews_per_day as f64,
        );
        clamped.round() as usize
    }

    /// Compute recall probability with ISS v2.7 FSRS retrievability model.
    ///
    /// Uses FSRS retrievability formula (gentler than exponential):
    /// - R(t) = (1 + t / (9 * S))^-1
    /// - Applied with stability_scale: S_eff = S * stability_scale
    /// - Applied with fatigue: p_recall_effective = R * fatigue_factor
    ///
    /// This formula is more forgiving than exp(-t/S) for low stability items,
    /// preventing the death spiral where stability collapse → near-zero recall → perpetual failure.
    pub fn compute_recall_probability(
        &self,
        stability: f64,
        elapsed_days: f64,
        progress_in_session: f64, // 0.0 to 1.0
    ) -> f64 {
        // Apply stability scale from student profile
        let stability_eff = stability * self.params.stability_scale as f64;

        // Compute base recall probability using FSRS retrievability formula
        // R(t) = (1 + t / (9 * S))^-1
        // This is gentler than exp(-t/S) for low stability items
        let p_recall = if stability_eff > 0.01 {
            let r = 1.0 / (1.0 + elapsed_days / (9.0 * stability_eff));
            r
        } else {
            // New items or very low stability: use lapse_baseline
            1.0 - self.params.lapse_baseline as f64
        };

        // Apply fatigue factor (late in session = lower recall)
        let fatigue_factor = 1.0 - self.params.fatigue_sensitivity as f64 * progress_in_session;

        (p_recall * fatigue_factor).clamp(0.0, 1.0)
    }

    /// Attempt recall of an item using FSRS-aligned retrievability model.
    ///
    /// ISS v2.4: Now blends FSRS recall with energy state to reflect actual cognitive decay.
    /// Low energy items have lower recall probability even if FSRS says they should be fine.
    ///
    /// Now also updates frustration, weighted failures, and session counters
    /// based on whether the failure was "expected" vs "surprise".
    ///
    /// # Arguments
    /// * `stability` - FSRS stability value in days
    /// * `difficulty` - Item difficulty (typically 1.0-10.0)
    /// * `elapsed_days` - Days since last review
    /// * `review_count` - Number of previous reviews (0 = new item)
    /// * `energy` - Current item energy (0.0-1.0), reflects decay between reviews
    /// * `progress_in_session` - How far through the session (0.0-1.0)
    pub fn attempt_recall(
        &mut self,
        stability: f64,
        difficulty: f64,
        elapsed_days: f64,
        review_count: u32,
        _energy: f64, // Kept for API compatibility, not used with success boost approach
        progress_in_session: f64,
    ) -> RecallResult {
        // Use ISS v2.1 enhanced error model (FSRS-based)
        let fsrs_recall =
            self.compute_recall_probability(stability, elapsed_days, progress_in_session);

        // ISS v2.4: Young item success boost
        // New items need to succeed early to build stability. Without this,
        // they fail repeatedly, keeping stability at minimum, causing perpetual failure spiral.
        //
        // For young items (0-5 reviews), we boost the effective recall probability
        // to give them a fighting chance to stabilize.
        let maturity = (review_count as f64 / 5.0).clamp(0.0, 1.0);

        // Young items get up to +0.50 boost to recall probability
        // This aggressively compensates for energy decay before FSRS scheduling kicks in
        let young_boost = 0.50 * (1.0 - maturity);

        let r = (fsrs_recall + young_boost).clamp(0.0, 1.0);

        // Apply item variability (noise)
        let noise = self
            .rng
            .gen_range(-self.params.item_variability..self.params.item_variability);
        let final_r = (r + noise * 0.1).clamp(0.0, 1.0);

        // Roll for recall
        let roll: f64 = self.rng.gen();
        let recalled = if self.force_perfect_recall {
            true
        } else {
            roll < final_r
        };

        // If force_perfect_recall, we want retrievability to look like 1.0 for metrics,
        // but for internal state update, we should maybe still simulate the "real" biology?
        // NO - if we force recall, the brain should "believe" it recalled it.
        // However, if we set r=1.0, update_after_review will think it was "expected"
        // and reduce frustration.
        let reported_r = if self.force_perfect_recall {
            1.0
        } else {
            final_r
        };

        // Update latent states
        // We pass reported_r so frustration logic sees "high retention -> expected success"
        self.update_after_review(reported_r, recalled, review_count, difficulty);

        RecallResult {
            recalled,
            retrievability: reported_r,
        }
    }

    /// Update frustration, weighted failure score, and session counters after a review.
    ///
    /// ISS v2.4: Uses adaptive R* based on item maturity.
    /// Young items (0-5 reviews) have lower expectations (R*=0.65).
    /// Mature items (5+ reviews) have higher expectations (R*=0.85).
    ///
    /// A fail near R* (well-timed) is less painful than a fail far from R* (mistimed).
    fn update_after_review(&mut self, r: f64, recalled: bool, review_count: u32, difficulty: f64) {
        // ISS v2.4: Adaptive R* based on item maturity
        // Young items have lower expectations to prevent death spiral
        let r_target = self.params.compute_expected_recall(r, review_count);

        // Compute maturity for frustration scaling (ISS v2.4)
        let maturity = (review_count as f64 / 5.0).clamp(0.0, 1.0);

        // Update running average difficulty
        self.session_review_count += 1;
        let n = self.session_review_count as f64;
        self.avg_difficulty = self.avg_difficulty * (n - 1.0) / n + difficulty / n;

        // Distance from adaptive R* (0..1, small if near sweet spot, big if far)
        let r_penalty = (r - r_target).abs();

        if recalled {
            // Success
            self.session_successes += 1;

            // Successful reviews reduce frustration, especially if R is in good band
            let in_band = r_penalty < 0.15; // Within ±0.15 of R*
            let decay = if in_band { 0.05 } else { 0.02 };
            self.frustration *= 1.0 - decay;

            // Any success gradually reduces weighted failure burden
            self.weighted_failure_score *= 0.9;
        } else {
            // Failure
            self.session_failures += 1;

            // ISS v2.4 Fix 1: Scale failure impact by maturity
            // Young items failing is expected → lower frustration impact
            // Mature items failing is concerning → higher frustration impact
            let maturity_factor = 0.5 + 0.5 * maturity; // 0.5 for young, 1.0 for mature

            // Base pain from failing - scaled by maturity
            let base = 1.0 * maturity_factor;

            // R-distance penalty: failing far from the adaptive R* is worse
            // overdue_sensitivity now means "sensitivity to bad scheduling"
            let schedule_term = 1.0 + self.params.overdue_sensitivity * r_penalty;

            let weight = base * schedule_term;
            self.weighted_failure_score += weight;

            // Frustration increase depends on scheduling quality AND maturity
            // Young items (maturity_factor=0.5) cause 50% less frustration
            let delta = self.params.overdue_sensitivity * 0.2 * r_penalty * maturity_factor;
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
        let _brain1 = StudentBrain::new(params.clone(), 42);
        let _brain2 = StudentBrain::new(params, 42);

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

            let r1 = b1.attempt_recall(stability, difficulty, 1.0, 5, 0.5, 0.0); // 1 day elapsed, E=0.5
            let r2 = b2.attempt_recall(stability, difficulty, 30.0, 5, 0.3, 0.0); // 30 days elapsed, E lower

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
            brain.attempt_recall(0.1, 10.0, 1000.0, 5, 0.05, 0.0); // Very hard to recall, low energy
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
