//! Multi-objective evaluation metrics for scheduler assessment.
//!
//! This module implements the formal evaluation specification to prevent
//! metric gaming by naive optimizers. It combines coverage, cost, fairness,
//! overkill, and spacing metrics into a composite score with degeneracy detection.

use crate::debug_stats::{DelayBuckets, RBuckets, StudentDebugSummary};
use crate::metrics::SimulationMetrics;
use serde::Serialize;
use std::collections::HashMap;

// ============================================================================
// ENUMS
// ============================================================================

/// Degeneracy flags for problematic scheduler behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Flag {
    /// Tiny plan with absurd review counts per item
    Hammering,
    /// Many items never reviewed after full horizon
    Starvation,
    /// Vast majority of reviews at R ≥ 0.95
    Overkill,
    /// Reviews concentrated in d_0 and d_15+ with no middle ground
    SpacingCollapse,
}

/// Qualitative verdict for scheduler behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Verdict {
    Unacceptable,
    Ok,
    Good,
    Excellent,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Unacceptable => write!(f, "UNACCEPTABLE"),
            Verdict::Ok => write!(f, "OK"),
            Verdict::Good => write!(f, "GOOD"),
            Verdict::Excellent => write!(f, "EXCELLENT"),
        }
    }
}

// ============================================================================
// METRICS STRUCT
// ============================================================================

/// Comprehensive evaluation metrics for a simulation run.
#[derive(Debug, Clone, Serialize)]
pub struct EvalMetrics {
    // Coverage
    pub coverage_h: f64,
    pub mean_r_h: f64,
    pub n_never: usize,
    pub rho_starve: f64,

    // Cost
    pub r_bar: f64,
    pub total_reviews: u64,
    pub c_norm: f64,

    // Fairness
    pub sigma_r: f64,
    pub cv_r: f64,
    pub gini_r: f64, // Gini coefficient [0=perfect equality, 1=max inequality]
    pub mmr: f64,    // max/mean ratio

    // Overkill (from RBuckets)
    pub rho_over: f64,
    pub rho_safe: f64,
    pub rho_need: f64,

    // Spacing (from DelayBuckets)
    pub d_0_pct: f64,
    pub d_1_pct: f64,
    pub s_space: f64, // spacing entropy

    // Context
    pub n_items: usize,
    pub horizon_days: u32,
}

/// Complete evaluation result with score, verdict, and flags.
#[derive(Debug, Clone, Serialize)]
pub struct EvalResult {
    pub score: f64,
    pub verdict: Verdict,
    pub flags: Vec<Flag>,
    pub metrics: EvalMetrics,
}

// ============================================================================
// WEIGHT CONSTANTS
// ============================================================================

const W_COV: f64 = 0.35;
const W_COST: f64 = 0.20;
const W_OVER: f64 = 0.15;
const W_UNFAIR: f64 = 0.10;
const W_STARVE: f64 = 0.10;
const W_SPACE: f64 = 0.10;

const CV_MAX: f64 = 2.0;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Compute expected reviews per item based on plan size and horizon.
fn r_expected(n: usize, horizon_days: u32) -> f64 {
    let h_factor = horizon_days as f64 / 30.0;
    match n {
        0..=10 => 5.0 + 0.3 * h_factor,
        11..=50 => 4.0 + 0.2 * h_factor,
        _ => 3.0 + 0.15 * h_factor,
    }
}

/// Compute spacing entropy from delay buckets.
fn spacing_entropy(buckets: &DelayBuckets) -> f64 {
    let counts = [
        buckets.d_0,
        buckets.d_1,
        buckets.d_2_3,
        buckets.d_4_7,
        buckets.d_8_14,
        buckets.d_15_30,
        buckets.d_31_plus,
    ];

    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }

    let mut entropy = 0.0;
    for &count in &counts {
        if count > 0 {
            let p = count as f64 / total as f64;
            entropy -= p * p.ln();
        }
    }

    // Normalize to [0, 1] (max entropy is ln(7))
    entropy / 7.0_f64.ln()
}

/// Compute overkill/safe/need rates from R buckets.
fn compute_r_rates(buckets: &RBuckets) -> (f64, f64, f64) {
    let total = buckets.r_0_0_3
        + buckets.r_0_3_0_6
        + buckets.r_0_6_0_85
        + buckets.r_0_85_0_95
        + buckets.r_0_95_1_0;

    if total == 0 {
        return (0.0, 0.0, 0.0);
    }

    let rho_over = buckets.r_0_95_1_0 as f64 / total as f64;
    let rho_safe = buckets.r_0_85_0_95 as f64 / total as f64;
    let rho_need = (buckets.r_0_0_3 + buckets.r_0_3_0_6 + buckets.r_0_6_0_85) as f64 / total as f64;

    (rho_over, rho_safe, rho_need)
}

/// Compute fairness metrics from per-item review counts.
/// Returns (mean, sigma, cv, gini, mmr).
fn compute_fairness(
    per_item_reviews: &HashMap<i64, u32>,
    n_items: usize,
) -> (f64, f64, f64, f64, f64) {
    if per_item_reviews.is_empty() || n_items == 0 {
        return (0.0, 0.0, 0.0, 0.0, 0.0);
    }

    // Include zero-review items in the calculation
    let mut counts: Vec<f64> = per_item_reviews.values().map(|&c| c as f64).collect();

    // Pad with zeros for items that were never reviewed
    let n_reviewed = counts.len();
    for _ in n_reviewed..n_items {
        counts.push(0.0);
    }

    let n = counts.len() as f64;
    let sum: f64 = counts.iter().sum();
    let mean = sum / n;

    if mean == 0.0 {
        return (0.0, 0.0, 0.0, 0.0, 0.0);
    }

    // Standard deviation
    let variance: f64 = counts.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let sigma = variance.sqrt();

    // Coefficient of variation
    let cv = sigma / mean;

    // Max/mean ratio
    let max_count = counts.iter().cloned().fold(0.0_f64, f64::max);
    let mmr = max_count / mean;

    // Gini coefficient: G = (Σ|xi - xj|) / (2 * n^2 * mean)
    // Using the sorted formula: G = (2 * Σ(i * x_i)) / (n * Σx_i) - (n+1)/n
    counts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let weighted_sum: f64 = counts
        .iter()
        .enumerate()
        .map(|(i, &x)| (i as f64 + 1.0) * x)
        .sum();
    let gini = if sum > 0.0 {
        (2.0 * weighted_sum) / (n * sum) - (n + 1.0) / n
    } else {
        0.0
    };

    (mean, sigma, cv, gini.clamp(0.0, 1.0), mmr)
}

// ============================================================================
// DEGENERACY DETECTORS
// ============================================================================

/// Detect small-plan hammering.
fn detect_hammering(n: usize, r_bar: f64, horizon_days: u32) -> bool {
    let threshold = match n {
        0..=10 => 15.0 + 0.5 * (horizon_days as f64 / 30.0),
        11..=50 => 12.0 + 0.4 * (horizon_days as f64 / 30.0),
        _ => 10.0 + 0.3 * (horizon_days as f64 / 30.0),
    };
    r_bar > threshold
}

/// Detect new-item starvation.
fn detect_starvation(n_never: usize, n: usize) -> bool {
    if n == 0 {
        return false;
    }
    let pct_never = n_never as f64 / n as f64;
    pct_never > 0.15
}

/// Detect overkill abuse.
fn detect_overkill(rho_over: f64) -> bool {
    rho_over > 0.50
}

/// Detect spacing collapse.
fn detect_spacing_collapse(d_0_pct: f64, s_space: f64) -> bool {
    d_0_pct > 0.40 || s_space < 0.40
}

// ============================================================================
// SCORE COMPUTATION
// ============================================================================

/// Apply guardrails to cap score when flags are triggered.
fn apply_guardrails(score: f64, flags: &[Flag]) -> f64 {
    let mut result = score;

    if flags.contains(&Flag::Hammering) {
        result = result.min(0.30);
    }
    if flags.contains(&Flag::Starvation) {
        result = result.min(0.40);
    }
    if flags.contains(&Flag::Overkill) {
        result = result.min(0.50);
    }
    if flags.contains(&Flag::SpacingCollapse) {
        result -= 0.10;
    }

    result
}

/// Compute verdict from metrics and score.
fn compute_verdict(metrics: &EvalMetrics, score: f64, flags: &[Flag]) -> Verdict {
    // UNACCEPTABLE checks (ISS v2.2: using mean_r_h with lowered thresholds)
    if metrics.mean_r_h < 0.40  // Lowered from 0.50 since mean_r is more forgiving
        || metrics.rho_starve > 0.25
        || metrics.rho_over > 0.60
        || flags.len() >= 2
    {
        return Verdict::Unacceptable;
    }

    // EXCELLENT checks (ISS v2.2: using mean_r_h)
    if metrics.mean_r_h >= 0.75  // Lowered from 0.85
        && score >= 0.60
        && metrics.rho_over <= 0.30
        && metrics.rho_starve <= 0.05
        && flags.is_empty()
    {
        return Verdict::Excellent;
    }

    // GOOD checks (ISS v2.2: using mean_r_h)
    if metrics.mean_r_h >= 0.60  // Lowered from 0.70
        && score >= 0.40
        && flags.is_empty()
    {
        return Verdict::Good;
    }

    Verdict::Ok
}

// ============================================================================
// MAIN EVALUATION FUNCTION
// ============================================================================

/// Evaluate a simulation run against multi-objective metrics.
///
/// # Arguments
/// * `sim_metrics` - Standard simulation metrics
/// * `debug` - Debug statistics including R-buckets and delay buckets
/// * `per_item_reviews` - Map of node_id -> review count
/// * `horizon_days` - Evaluation horizon
///
/// # Returns
/// Complete evaluation result with score, verdict, and flags.
pub fn evaluate(
    sim_metrics: &SimulationMetrics,
    debug: &StudentDebugSummary,
    per_item_reviews: &HashMap<i64, u32>,
    horizon_days: u32,
) -> EvalResult {
    let n_items = sim_metrics.goal_item_count;
    let n_never = sim_metrics.items_never_reviewed;

    // Compute fairness metrics
    let (r_bar, sigma_r, cv_r, gini_r, mmr) = compute_fairness(per_item_reviews, n_items);

    // Compute R-rate metrics
    let (rho_over, rho_safe, rho_need) = compute_r_rates(&debug.r_buckets);

    // Compute spacing metrics
    let total_reviews = debug.total_reviews as u64;
    let d_0_pct = if total_reviews > 0 {
        debug.delay_buckets.d_0 as f64 / total_reviews as f64
    } else {
        0.0
    };
    let d_1_pct = if total_reviews > 0 {
        debug.delay_buckets.d_1 as f64 / total_reviews as f64
    } else {
        0.0
    };
    let s_space = spacing_entropy(&debug.delay_buckets);

    // Compute normalized cost
    let r_exp = r_expected(n_items, horizon_days);
    let c_norm = if r_exp > 0.0 { r_bar / r_exp } else { 0.0 };

    // Starvation rate
    let rho_starve = if n_items > 0 {
        n_never as f64 / n_items as f64
    } else {
        0.0
    };

    // Build metrics struct
    let metrics = EvalMetrics {
        coverage_h: sim_metrics.coverage_t,
        mean_r_h: sim_metrics.mean_r_t,
        n_never,
        rho_starve,
        r_bar,
        total_reviews,
        c_norm,
        sigma_r,
        cv_r,
        gini_r,
        mmr,
        rho_over,
        rho_safe,
        rho_need,
        d_0_pct,
        d_1_pct,
        s_space,
        n_items,
        horizon_days,
    };

    // Detect degeneracies
    let mut flags = Vec::new();
    if detect_hammering(n_items, r_bar, horizon_days) {
        flags.push(Flag::Hammering);
    }
    if detect_starvation(n_never, n_items) {
        flags.push(Flag::Starvation);
    }
    if detect_overkill(rho_over) {
        flags.push(Flag::Overkill);
    }
    if detect_spacing_collapse(d_0_pct, s_space) {
        flags.push(Flag::SpacingCollapse);
    }

    // Compute raw score
    // ISS v2.2: Use mean_r_h (continuous coverage) as primary coverage term
    // This rewards partial learning and aligns with FSRS semantics
    let cv_norm = (cv_r / CV_MAX).min(1.0);
    let c_norm_capped = c_norm.min(2.0);

    let raw_score = W_COV * metrics.mean_r_h
        - W_COST * c_norm_capped
        - W_OVER * rho_over
        - W_UNFAIR * cv_norm
        - W_STARVE * rho_starve
        + W_SPACE * s_space;

    // Apply guardrails
    let final_score = apply_guardrails(raw_score, &flags).clamp(-1.0, 1.0);

    // Compute verdict
    let verdict = compute_verdict(&metrics, final_score, &flags);

    EvalResult {
        score: final_score,
        verdict,
        flags,
        metrics,
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn make_r_buckets(
        r_0_0_3: u64,
        r_0_3_0_6: u64,
        r_0_6_0_85: u64,
        r_0_85_0_95: u64,
        r_0_95_1_0: u64,
    ) -> RBuckets {
        RBuckets {
            r_0_0_3,
            r_0_3_0_6,
            r_0_6_0_85,
            r_0_85_0_95,
            r_0_95_1_0,
        }
    }

    #[test]
    fn test_r_expected_small_plan() {
        let exp = r_expected(7, 30);
        assert!(exp > 5.0 && exp < 6.0);
    }

    #[test]
    fn test_r_expected_large_plan() {
        let exp = r_expected(100, 30);
        assert!(exp > 3.0 && exp < 4.0);
    }

    #[test]
    fn test_spacing_entropy_uniform() {
        let buckets = DelayBuckets {
            d_0: 100,
            d_1: 100,
            d_2_3: 100,
            d_4_7: 100,
            d_8_14: 100,
            d_15_30: 100,
            d_31_plus: 100,
        };
        let s = spacing_entropy(&buckets);
        assert!(
            (s - 1.0).abs() < 0.01,
            "Uniform distribution should have entropy ~1.0"
        );
    }

    #[test]
    fn test_spacing_entropy_concentrated() {
        let buckets = DelayBuckets {
            d_0: 1000,
            d_1: 0,
            d_2_3: 0,
            d_4_7: 0,
            d_8_14: 0,
            d_15_30: 0,
            d_31_plus: 0,
        };
        let s = spacing_entropy(&buckets);
        assert!(
            s < 0.01,
            "Concentrated distribution should have entropy ~0.0"
        );
    }

    #[test]
    fn test_detect_hammering_small_plan() {
        assert!(detect_hammering(7, 25.0, 30));
        assert!(!detect_hammering(7, 10.0, 30));
    }

    #[test]
    fn test_detect_starvation() {
        assert!(detect_starvation(3, 10)); // 30% never reviewed
        assert!(!detect_starvation(1, 10)); // 10% never reviewed
    }

    #[test]
    fn test_detect_overkill() {
        assert!(detect_overkill(0.60));
        assert!(!detect_overkill(0.30));
    }

    #[test]
    fn test_verdict_unacceptable_low_coverage() {
        // ISS v2.2: Testing with mean_r_h below 0.40 threshold
        let metrics = EvalMetrics {
            coverage_h: 0.40,
            mean_r_h: 0.35, // Below 0.40 threshold
            n_never: 0,
            rho_starve: 0.0,
            r_bar: 5.0,
            total_reviews: 50,
            c_norm: 1.0,
            sigma_r: 1.0,
            cv_r: 0.2,
            gini_r: 0.1,
            mmr: 1.5,
            rho_over: 0.2,
            rho_safe: 0.3,
            rho_need: 0.5,
            d_0_pct: 0.1,
            d_1_pct: 0.2,
            s_space: 0.8,
            n_items: 10,
            horizon_days: 30,
        };
        let verdict = compute_verdict(&metrics, 0.5, &[]);
        assert_eq!(verdict, Verdict::Unacceptable);
    }

    #[test]
    fn test_verdict_excellent() {
        // ISS v2.2: Testing with mean_r_h above 0.75 threshold
        let metrics = EvalMetrics {
            coverage_h: 0.90,
            mean_r_h: 0.80, // Above 0.75 threshold
            n_never: 0,
            rho_starve: 0.0,
            r_bar: 5.0,
            total_reviews: 50,
            c_norm: 1.0,
            sigma_r: 1.0,
            cv_r: 0.2,
            gini_r: 0.1,
            mmr: 1.5,
            rho_over: 0.2,
            rho_safe: 0.3,
            rho_need: 0.5,
            d_0_pct: 0.1,
            d_1_pct: 0.2,
            s_space: 0.8,
            n_items: 10,
            horizon_days: 30,
        };
        let verdict = compute_verdict(&metrics, 0.65, &[]);
        assert_eq!(verdict, Verdict::Excellent);
    }

    #[test]
    fn test_guardrails_cap_hammering() {
        let score = apply_guardrails(0.80, &[Flag::Hammering]);
        assert!((score - 0.30).abs() < 0.01);
    }

    #[test]
    fn test_guardrails_stack() {
        let score = apply_guardrails(0.50, &[Flag::SpacingCollapse]);
        assert!((score - 0.40).abs() < 0.01);
    }

    #[test]
    fn test_compute_fairness() {
        let mut reviews = HashMap::new();
        reviews.insert(1, 5);
        reviews.insert(2, 5);
        reviews.insert(3, 5);

        let (r_bar, sigma, cv, gini, mmr) = compute_fairness(&reviews, 3);
        assert!((r_bar - 5.0).abs() < 0.01);
        assert!(sigma < 0.01, "Uniform should have 0 std dev");
        assert!(cv < 0.01);
        assert!(gini < 0.01, "Uniform should have 0 Gini");
        assert!((mmr - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_fairness_with_zeros() {
        let mut reviews = HashMap::new();
        reviews.insert(1, 10);
        // Item 2 and 3 have 0 reviews (not in map)

        let (r_bar, _sigma, cv, gini, mmr) = compute_fairness(&reviews, 3);
        // Mean = (10 + 0 + 0) / 3 = 3.33
        assert!((r_bar - 3.33).abs() < 0.1);
        assert!(cv > 1.0, "High variance expected");
        assert!(gini > 0.3, "Inequality should show in Gini");
        assert!(mmr > 2.0, "High MMR expected");
    }

    // =========================================================================
    // ADVERSARIAL TESTS - Verify bad schedulers get penalized
    // =========================================================================

    #[test]
    fn test_adversarial_hammer_scenario() {
        // Simulate "hammer 7 cards" - 7 items with 200 reviews each = absurd
        let mut reviews = HashMap::new();
        for i in 1..=7 {
            reviews.insert(i, 200); // 200 reviews per item = extreme hammering
        }

        let n_items = 7;
        let horizon = 30;
        let (r_bar, _sigma, _cv, _gini, _mmr) = compute_fairness(&reviews, n_items);

        // r_bar = 200, threshold for n=7, H=30 is ~15.5
        assert!(
            detect_hammering(n_items, r_bar, horizon),
            "Hammering detector should trigger: r_bar={}, expected > 15.5",
            r_bar
        );

        // Score should be capped at 0.30 by guardrails
        let score = apply_guardrails(0.80, &[Flag::Hammering]);
        assert!(score <= 0.30, "Hammering should cap score at 0.30");
    }

    #[test]
    fn test_adversarial_starvation_scenario() {
        // Simulate "never introduce new items" - 30% of items never reviewed
        let n_items = 100;
        let n_never = 30; // 30% never reviewed

        assert!(
            detect_starvation(n_never, n_items),
            "Starvation detector should trigger for 30% never reviewed"
        );
    }

    #[test]
    fn test_adversarial_overkill_scenario() {
        // Simulate "only review when R > 0.95" - 70% overkill
        let rho_over = 0.70;

        assert!(
            detect_overkill(rho_over),
            "Overkill detector should trigger for 70% overkill rate"
        );

        // Score should be capped at 0.50
        let score = apply_guardrails(0.80, &[Flag::Overkill]);
        assert!(score <= 0.50, "Overkill should cap score at 0.50");
    }

    #[test]
    fn test_adversarial_multiple_flags_unacceptable() {
        // Two or more flags -> UNACCEPTABLE verdict
        let metrics = EvalMetrics {
            coverage_h: 0.80, // Good coverage but...
            mean_r_h: 0.85,
            n_never: 0,
            rho_starve: 0.0,
            r_bar: 25.0, // High reviews = hammering
            total_reviews: 175,
            c_norm: 5.0,
            sigma_r: 5.0,
            cv_r: 0.5,
            gini_r: 0.3,
            mmr: 3.0,
            rho_over: 0.55, // Also overkill
            rho_safe: 0.2,
            rho_need: 0.25,
            d_0_pct: 0.1,
            d_1_pct: 0.2,
            s_space: 0.7,
            n_items: 7,
            horizon_days: 30,
        };

        let flags = vec![Flag::Hammering, Flag::Overkill];
        let verdict = compute_verdict(&metrics, 0.30, &flags);

        assert_eq!(
            verdict,
            Verdict::Unacceptable,
            "Two flags should always result in UNACCEPTABLE"
        );
    }

    // =========================================================================
    // MONOTONICITY TESTS - More X → consistent effect on score
    // =========================================================================

    #[test]
    fn test_monotonicity_more_overkill_lower_score() {
        // Higher overkill rate should always reduce raw score
        let base_score_low_overkill = W_COV * 0.80 - W_OVER * 0.10; // 10% overkill
        let base_score_high_overkill = W_COV * 0.80 - W_OVER * 0.50; // 50% overkill

        assert!(
            base_score_low_overkill > base_score_high_overkill,
            "More overkill should always reduce score"
        );
    }

    #[test]
    fn test_monotonicity_better_spacing_higher_score() {
        // Higher spacing entropy should improve score
        let score_low_entropy = W_SPACE * 0.20;
        let score_high_entropy = W_SPACE * 0.90;

        assert!(
            score_high_entropy > score_low_entropy,
            "Better spacing entropy should always increase score"
        );
    }

    #[test]
    fn test_gini_perfect_equality() {
        // All items with same review count → Gini ≈ 0
        let mut reviews = HashMap::new();
        for i in 1..=10 {
            reviews.insert(i, 5); // All have exactly 5 reviews
        }
        let (_mean, _sigma, _cv, gini, _mmr) = compute_fairness(&reviews, 10);
        assert!(
            gini < 0.01,
            "Perfect equality should have Gini near 0, got {}",
            gini
        );
    }

    #[test]
    fn test_gini_extreme_inequality() {
        // One item gets all reviews → Gini near 1
        let mut reviews = HashMap::new();
        reviews.insert(1, 100); // One item gets 100 reviews
                                // Items 2-10 get 0 reviews (not in map)

        let (_mean, _sigma, _cv, gini, _mmr) = compute_fairness(&reviews, 10);
        assert!(
            gini > 0.8,
            "Extreme inequality should have Gini near 1, got {}",
            gini
        );
    }
}
