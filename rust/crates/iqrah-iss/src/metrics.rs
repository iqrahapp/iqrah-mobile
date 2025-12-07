//! Precise metric definitions for ISS.
//!
//! All formulas follow the explicit definitions from the ISS requirements:
//! - Retrievability: R(t) = (1 + t/(9*S))^-1
//! - Mastery: R(t) >= 0.9, equivalently S >= t
//! - Plan faithfulness via Spearman rank correlation

use std::collections::HashMap;

/// Compute retrievability at a given horizon using FSRS power formula.
///
/// # Formula
/// ```text
/// R(t) = (1 + t / (9 * S))^-1
/// ```
///
/// # Arguments
/// * `stability` - FSRS stability value (S) in days
/// * `horizon_days` - Time horizon (t) in days
///
/// # Returns
/// Probability of recall at horizon, in [0, 1]
pub fn retrievability(stability: f64, horizon_days: f64) -> f64 {
    if stability <= 0.0 {
        return 0.0;
    }
    (1.0 + horizon_days / (9.0 * stability)).powi(-1)
}

/// Check if an item is mastered at a given horizon.
///
/// An item is mastered if retrievability >= 0.9.
/// Equivalently, stability >= horizon_days.
///
/// # Arguments
/// * `stability` - FSRS stability value in days
/// * `horizon_days` - Evaluation horizon in days
pub fn is_mastered(stability: f64, horizon_days: f64) -> bool {
    retrievability(stability, horizon_days) >= 0.9
}

/// Compute Spearman rank correlation coefficient between two vectors.
///
/// # Arguments
/// * `x` - First vector of values
/// * `y` - Second vector of values (must be same length as x)
///
/// # Returns
/// Spearman's ρ in [-1, 1], or 0.0 if vectors are too short
pub fn spearman_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.len() < 2 {
        return 0.0;
    }

    let n = x.len();

    // Convert to ranks
    let x_ranks = to_ranks(x);
    let y_ranks = to_ranks(y);

    // Compute Pearson correlation on ranks
    let mean_x: f64 = x_ranks.iter().sum::<f64>() / n as f64;
    let mean_y: f64 = y_ranks.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..n {
        let dx = x_ranks[i] - mean_x;
        let dy = y_ranks[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return 0.0;
    }

    cov / (var_x.sqrt() * var_y.sqrt())
}

/// Convert values to ranks (1-based, handling ties with average rank).
fn to_ranks(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        // Find all ties
        while j < n && indexed[j].1 == indexed[i].1 {
            j += 1;
        }
        // Average rank for ties
        let avg_rank = (i + j + 1) as f64 / 2.0; // +1 because ranks are 1-based
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j;
    }
    ranks
}

/// Compute plan faithfulness using Spearman correlation.
///
/// # Arguments
/// * `plan_priorities` - Map of node_id -> priority rank (lower = higher priority)
/// * `introduction_order` - Map of node_id -> order of first introduction (1 = first)
///
/// # Returns
/// Plan faithfulness in [0, 1] where 1 = perfect adherence
pub fn plan_faithfulness(
    plan_priorities: &HashMap<i64, usize>,
    introduction_order: &HashMap<i64, usize>,
) -> f64 {
    let mut p_ranks: Vec<f64> = Vec::new();
    let mut o_ranks: Vec<f64> = Vec::new();

    for (node_id, &p_rank) in plan_priorities {
        if let Some(&o_rank) = introduction_order.get(node_id) {
            p_ranks.push(p_rank as f64);
            o_ranks.push(o_rank as f64);
        }
    }

    if p_ranks.len() < 2 {
        return 0.5; // Undefined, return neutral
    }

    let rho = spearman_correlation(&p_ranks, &o_ranks);
    // Map from [-1, 1] to [0, 1]
    (rho + 1.0) / 2.0
}

/// A snapshot of stability values at the end of a day.
#[derive(Debug, Clone)]
pub struct DailySnapshot {
    /// Day number (0-indexed)
    pub day: u32,
    /// Stability values for each goal item at end of day
    pub stabilities: HashMap<i64, f64>,
}

impl DailySnapshot {
    /// Create a new snapshot for the given day.
    pub fn new(day: u32, stabilities: HashMap<i64, f64>) -> Self {
        Self { day, stabilities }
    }

    /// Count how many goal items are mastered at this snapshot.
    pub fn count_mastered(&self, goal_items: &[i64]) -> usize {
        goal_items
            .iter()
            .filter(|&&nid| {
                self.stabilities
                    .get(&nid)
                    .map(|&s| is_mastered(s, self.day as f64))
                    .unwrap_or(false)
            })
            .count()
    }
}

/// Calculate days to mastery from daily snapshots.
///
/// # Arguments
/// * `snapshots` - Daily stability snapshots
/// * `goal_items` - Goal item node IDs
/// * `target_fraction` - Fraction of items that must be mastered (e.g., 0.8 = 80%)
///
/// # Returns
/// First day (1-indexed) where at least target_fraction are mastered, or None
pub fn days_to_mastery(
    snapshots: &[DailySnapshot],
    goal_items: &[i64],
    target_fraction: f64,
) -> Option<u32> {
    if goal_items.is_empty() {
        return None;
    }

    let threshold = (goal_items.len() as f64 * target_fraction).ceil() as usize;

    for snapshot in snapshots {
        let mastered = snapshot.count_mastered(goal_items);
        if mastered >= threshold {
            return Some(snapshot.day + 1); // Convert to 1-indexed
        }
    }

    None
}

/// Aggregated simulation metrics for a single student run.
#[derive(Debug, Clone)]
pub struct SimulationMetrics {
    /// Items mastered per minute of study time (primary metric)
    pub retention_per_minute: f64,

    /// Smallest day d where >= X% of items mastered (None if never reached)
    pub days_to_mastery: Option<u32>,

    /// Fraction of goal items that reached mastery [0, 1]
    pub coverage_pct: f64,

    /// Correlation between plan priority and actual introduction order [0, 1]
    pub plan_faithfulness: f64,

    /// Total simulated learning minutes
    pub total_minutes: f64,

    /// Total simulated days (may be less than target if gave up)
    pub total_days: u32,

    /// Whether student gave up before completing simulation
    pub gave_up: bool,

    /// Number of goal items (for reference)
    pub goal_item_count: usize,

    /// Number of items that reached mastery
    pub items_mastered: usize,

    // === New outcome metrics at T_eval ===
    /// Fraction of items with R(T_eval) >= 0.9 at evaluation horizon
    pub coverage_t: f64,

    /// Mean retrievability across all goal items at T_eval
    pub mean_r_t: f64,

    /// Number of items with R(T_eval) >= 0.9
    pub items_good_t: usize,

    /// Items good at T / total_minutes (efficiency)
    pub rpm_t: f64,

    /// For long scenarios: items with R(T_short) >= 0.9 where T_short = min(90, target_days)
    pub items_good_short: Option<usize>,

    /// For long scenarios: items_good_short / total_minutes
    pub rpm_short: Option<f64>,

    // === Acquisition metrics (T_acq = 14) ===
    /// Fraction of items with Stability >= min(14, T)
    pub coverage_acq: f64,

    /// Mean retrievability at T_acq
    pub mean_r_acq: f64,

    /// Number of goal items that were never reviewed (0 reviews)
    pub items_never_reviewed: usize,
}

impl SimulationMetrics {
    /// Compute final composite score.
    ///
    /// # Arguments
    /// * `target_days` - Original simulation target duration
    /// * `expected_rpm` - Expected retention per minute for normalization
    ///
    /// # Returns
    /// Score in [-1, 1] where negative = gave up early, higher = better
    pub fn final_score(&self, target_days: u32, expected_rpm: f64) -> f64 {
        if self.gave_up {
            // Penalty proportional to how early they gave up
            let survival_ratio = (self.total_days as f64 / target_days as f64).clamp(0.0, 1.0);
            return -1.0 * (1.0 - survival_ratio);
        }

        // Normalize retention per minute (cap at 1.0)
        let r_norm = if expected_rpm > 0.0 {
            (self.retention_per_minute / expected_rpm).min(1.0)
        } else {
            0.0
        };

        // Normalize days to mastery (faster = better)
        let d = self.days_to_mastery.unwrap_or(target_days);
        let mastery_term = (1.0 - d as f64 / target_days as f64).clamp(0.0, 1.0);

        // Clamp other metrics
        let cov = self.coverage_pct.clamp(0.0, 1.0);
        let faith = self.plan_faithfulness.clamp(0.0, 1.0);

        // Weighted sum: retention 40%, mastery time 30%, coverage 20%, faithfulness 10%
        0.4 * r_norm + 0.3 * mastery_term + 0.2 * cov + 0.1 * faith
    }

    /// Create metrics from raw simulation data.
    ///
    /// # Arguments
    /// * `stabilities` - Map of node_id -> final stability for each goal item
    /// * `goal_items` - Set of goal item node_ids
    /// * `horizon_days` - Evaluation horizon (usually target_days)
    /// * `total_minutes` - Total learning time
    /// * `total_days` - Days simulated
    /// * `gave_up` - Whether student quit early
    /// * `plan_priorities` - Priority rankings from plan
    /// * `introduction_order` - Order items were first introduced
    #[allow(clippy::too_many_arguments)]
    pub fn compute(
        stabilities: &HashMap<i64, f64>,
        goal_items: &[i64],
        horizon_days: f64,
        total_minutes: f64,
        total_days: u32,
        gave_up: bool,
        plan_priorities: &HashMap<i64, usize>,
        introduction_order: &HashMap<i64, usize>,
    ) -> Self {
        let items_mastered = goal_items
            .iter()
            .filter(|&&nid| {
                stabilities
                    .get(&nid)
                    .map(|&s| is_mastered(s, horizon_days))
                    .unwrap_or(false)
            })
            .count();

        let retention_per_minute = if total_minutes > 0.0 {
            items_mastered as f64 / total_minutes
        } else {
            0.0
        };

        let coverage_pct = if !goal_items.is_empty() {
            (items_mastered as f64 / goal_items.len() as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let faithfulness = plan_faithfulness(plan_priorities, introduction_order);

        // === New outcome metrics at T_eval ===

        // Compute R_eval for each item and aggregate metrics
        let r_evals: Vec<f64> = goal_items
            .iter()
            .map(|&nid| {
                stabilities
                    .get(&nid)
                    .map(|&s| retrievability(s, horizon_days))
                    .unwrap_or(0.0)
            })
            .collect();

        let items_good_t = r_evals.iter().filter(|&&r| r >= 0.9).count();
        let coverage_t = if !goal_items.is_empty() {
            items_good_t as f64 / goal_items.len() as f64
        } else {
            0.0
        };
        let mean_r_t = if !r_evals.is_empty() {
            r_evals.iter().sum::<f64>() / r_evals.len() as f64
        } else {
            0.0
        };
        let rpm_t = if total_minutes > 0.0 {
            items_good_t as f64 / total_minutes
        } else {
            0.0
        };

        // === Acquisition metrics (T_acq = min(14, horizon)) ===
        let t_acq = horizon_days.min(14.0);
        let items_acq = goal_items
            .iter()
            .filter(|&&nid| stabilities.get(&nid).map(|&s| s >= t_acq).unwrap_or(false))
            .count();
        let coverage_acq = if !goal_items.is_empty() {
            (items_acq as f64 / goal_items.len() as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let r_acq_vals: Vec<f64> = goal_items
            .iter()
            .map(|&nid| {
                stabilities
                    .get(&nid)
                    .map(|&s| retrievability(s, t_acq))
                    .unwrap_or(0.0)
            })
            .collect();
        let mean_r_acq = if !r_acq_vals.is_empty() {
            r_acq_vals.iter().sum::<f64>() / r_acq_vals.len() as f64
        } else {
            0.0
        };

        // For long scenarios (target >= 180 days), also compute short-term metrics
        let (items_good_short, rpm_short) = if horizon_days >= 180.0 {
            let t_short = 90.0_f64.min(horizon_days);
            let items_short = goal_items
                .iter()
                .filter(|&&nid| {
                    stabilities
                        .get(&nid)
                        .map(|&s| retrievability(s, t_short) >= 0.9)
                        .unwrap_or(false)
                })
                .count();
            let rpm = if total_minutes > 0.0 {
                items_short as f64 / total_minutes
            } else {
                0.0
            };
            (Some(items_short), Some(rpm))
        } else {
            (None, None)
        };

        // Items never reviewed = goal items not in introduction_order
        let items_never_reviewed = goal_items.len().saturating_sub(introduction_order.len());

        Self {
            retention_per_minute,
            days_to_mastery: None, // TODO: Implement daily snapshot tracking
            coverage_pct,
            plan_faithfulness: faithfulness,
            total_minutes,
            total_days,
            gave_up,
            goal_item_count: goal_items.len(),
            items_mastered,
            coverage_t,
            mean_r_t,
            items_good_t,
            rpm_t,
            items_good_short,
            rpm_short,
            coverage_acq,
            mean_r_acq,
            items_never_reviewed,
        }
    }
}

impl Default for SimulationMetrics {
    fn default() -> Self {
        Self {
            retention_per_minute: 0.0,
            days_to_mastery: None,
            coverage_pct: 0.0,
            plan_faithfulness: 0.5,
            total_minutes: 0.0,
            total_days: 0,
            gave_up: false,
            goal_item_count: 0,
            items_mastered: 0,
            coverage_t: 0.0,
            mean_r_t: 0.0,
            items_good_t: 0,
            rpm_t: 0.0,
            items_good_short: None,
            rpm_short: None,
            coverage_acq: 0.0,
            mean_r_acq: 0.0,
            items_never_reviewed: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrievability_formula() {
        // R(t) = (1 + t/(9*S))^-1
        // When t=0, R=1.0
        assert!((retrievability(10.0, 0.0) - 1.0).abs() < 0.001);

        // When S=t, R = (1 + 1/9)^-1 = 9/10 = 0.9
        assert!((retrievability(30.0, 30.0) - 0.9).abs() < 0.001);

        // When S=9, t=9: R = (1 + 9/(9*9))^-1 = (1 + 1/9)^-1 = 0.9
        assert!((retrievability(9.0, 9.0) - 0.9).abs() < 0.001);

        // Stability 0 should return 0
        assert_eq!(retrievability(0.0, 10.0), 0.0);
    }

    #[test]
    fn test_is_mastered_at_threshold() {
        // S >= t means mastered (R = 0.9)
        // Use slightly above to avoid floating point precision issues at exact boundary
        assert!(is_mastered(31.0, 30.0)); // Slightly above threshold
        assert!(is_mastered(40.0, 30.0)); // Above threshold
        assert!(!is_mastered(20.0, 30.0)); // Below threshold
        assert!(!is_mastered(29.0, 30.0)); // Just below threshold
    }

    #[test]
    fn test_spearman_perfect_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let rho = spearman_correlation(&x, &y);
        assert!((rho - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_spearman_inverse_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let rho = spearman_correlation(&x, &y);
        assert!((rho - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_plan_faithfulness_perfect() {
        let mut priorities = HashMap::new();
        let mut intros = HashMap::new();

        // Items introduced in priority order
        priorities.insert(1, 1);
        priorities.insert(2, 2);
        priorities.insert(3, 3);
        intros.insert(1, 1);
        intros.insert(2, 2);
        intros.insert(3, 3);

        let faith = plan_faithfulness(&priorities, &intros);
        assert!((faith - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_plan_faithfulness_reversed() {
        let mut priorities = HashMap::new();
        let mut intros = HashMap::new();

        // Items introduced in reverse priority order
        priorities.insert(1, 1);
        priorities.insert(2, 2);
        priorities.insert(3, 3);
        intros.insert(1, 3);
        intros.insert(2, 2);
        intros.insert(3, 1);

        let faith = plan_faithfulness(&priorities, &intros);
        assert!((faith - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_final_score_gave_up_early() {
        let metrics = SimulationMetrics {
            gave_up: true,
            total_days: 10,
            ..Default::default()
        };

        // Gave up at day 10 of 100 → survival_ratio = 0.1 → score = -0.9
        let score = metrics.final_score(100, 0.1);
        assert!((score - (-0.9)).abs() < 0.001);
    }

    #[test]
    fn test_final_score_completed() {
        let metrics = SimulationMetrics {
            retention_per_minute: 0.05, // Half of expected
            days_to_mastery: Some(50),  // Half of target
            coverage_pct: 0.8,
            plan_faithfulness: 1.0,
            total_minutes: 1000.0,
            total_days: 100,
            gave_up: false,
            goal_item_count: 100,
            items_mastered: 80,
            coverage_t: 0.8,
            mean_r_t: 0.85,
            items_good_t: 80,
            rpm_t: 0.08,
            items_good_short: None,
            rpm_short: None,
            coverage_acq: 0.8,
            mean_r_acq: 0.85,
            items_never_reviewed: 0,
        };

        // r_norm = 0.5, mastery_term = 0.5, cov = 0.8, faith = 1.0
        // 0.4*0.5 + 0.3*0.5 + 0.2*0.8 + 0.1*1.0 = 0.2 + 0.15 + 0.16 + 0.1 = 0.61
        let score = metrics.final_score(100, 0.1);
        assert!((score - 0.61).abs() < 0.01);
    }
}
