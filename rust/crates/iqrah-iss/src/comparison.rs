//! Comparison mode for running multiple scheduler variants.
//!
//! This module provides functionality to run the same scenario with different
//! scheduler variants and aggregate metrics for comparison.

use crate::baselines::SchedulerVariant;
use crate::config::{Scenario, SimulationConfig};
use crate::debug_stats::{RunDebugReport, VariantDebugReport};
use crate::evaluation::{evaluate, EvalResult};
use crate::metrics::SimulationMetrics;
use crate::simulator::Simulator;

use anyhow::Result;
use chrono::Utc;
use iqrah_core::ports::ContentRepository;
use serde::Serialize;
use std::sync::Arc;
use tracing::{info, warn};

/// Aggregated metrics for a group of students.
#[derive(Debug, Clone, Serialize)]
pub struct AggregatedMetrics {
    /// Mean final score
    pub final_score_mean: f64,
    /// Standard deviation of final score
    pub final_score_std: f64,
    /// Median final score
    pub final_score_median: f64,

    /// Mean retention per minute
    pub retention_per_minute_mean: f64,
    /// Standard deviation of retention per minute
    pub retention_per_minute_std: f64,

    /// Mean coverage percentage
    pub coverage_pct_mean: f64,
    /// Standard deviation of coverage
    pub coverage_pct_std: f64,

    /// Mean plan faithfulness
    pub plan_faithfulness_mean: f64,

    /// Mean days to mastery (only for students who reached it)
    pub days_to_mastery_mean: Option<f64>,
    /// Count of students who reached mastery
    pub days_to_mastery_count: usize,

    /// Fraction of students who gave up
    pub gave_up_fraction: f64,

    /// Total students simulated
    pub total_students: usize,

    // === New outcome metrics ===
    pub coverage_t_mean: f64,
    pub mean_r_t_mean: f64,
    pub rpm_t_mean: f64,
    pub rpm_short_mean: Option<f64>,
    pub coverage_acq_mean: f64,
    pub mean_r_acq_mean: f64,

    /// Mean number of items never reviewed across students
    pub items_never_reviewed_mean: f64,
}

impl AggregatedMetrics {
    /// Compute aggregated metrics from a list of simulation results.
    pub fn compute(metrics: &[SimulationMetrics], target_days: u32, expected_rpm: f64) -> Self {
        let n = metrics.len();
        if n == 0 {
            return Self::empty();
        }

        let n_f64 = n as f64;

        // Compute final scores
        let final_scores: Vec<f64> = metrics
            .iter()
            .map(|m| m.final_score(target_days, expected_rpm))
            .collect();

        let final_score_mean = final_scores.iter().sum::<f64>() / n_f64;
        let final_score_std = std_dev(&final_scores, final_score_mean);
        let final_score_median = median(&final_scores);

        // Retention per minute
        let rpms: Vec<f64> = metrics.iter().map(|m| m.retention_per_minute).collect();
        let retention_per_minute_mean = rpms.iter().sum::<f64>() / n_f64;
        let retention_per_minute_std = std_dev(&rpms, retention_per_minute_mean);

        // Coverage
        let coverages: Vec<f64> = metrics.iter().map(|m| m.coverage_pct).collect();
        let coverage_pct_mean = coverages.iter().sum::<f64>() / n_f64;
        let coverage_pct_std = std_dev(&coverages, coverage_pct_mean);

        // Plan faithfulness
        let faithfulness: Vec<f64> = metrics.iter().map(|m| m.plan_faithfulness).collect();
        let plan_faithfulness_mean = faithfulness.iter().sum::<f64>() / n_f64;

        // Days to mastery (only for those who reached it)
        let days_to_mastery: Vec<u32> = metrics.iter().filter_map(|m| m.days_to_mastery).collect();
        let days_to_mastery_count = days_to_mastery.len();
        let days_to_mastery_mean = if days_to_mastery_count > 0 {
            Some(days_to_mastery.iter().sum::<u32>() as f64 / days_to_mastery_count as f64)
        } else {
            None
        };

        // Gave up fraction
        let gave_up_count = metrics.iter().filter(|m| m.gave_up).count();
        let gave_up_fraction = gave_up_count as f64 / n_f64;

        // New outcome metrics
        let coverage_t_sum: f64 = metrics.iter().map(|m| m.coverage_t).sum();
        let coverage_t_mean = coverage_t_sum / n_f64;

        let mean_r_t_sum: f64 = metrics.iter().map(|m| m.mean_r_t).sum();
        let mean_r_t_mean = mean_r_t_sum / n_f64;

        let rpm_t_sum: f64 = metrics.iter().map(|m| m.rpm_t).sum();
        let rpm_t_mean = rpm_t_sum / n_f64;

        let rpm_short_count = metrics.iter().filter(|m| m.rpm_short.is_some()).count();
        let rpm_short_mean = if rpm_short_count > 0 {
            let sum: f64 = metrics.iter().filter_map(|m| m.rpm_short).sum();
            Some(sum / rpm_short_count as f64)
        } else {
            None
        };

        let coverage_acq_sum: f64 = metrics.iter().map(|m| m.coverage_acq).sum();
        let coverage_acq_mean = coverage_acq_sum / n_f64;

        let mean_r_acq_sum: f64 = metrics.iter().map(|m| m.mean_r_acq).sum();
        let mean_r_acq_mean = mean_r_acq_sum / n_f64;

        let items_never_reviewed_sum: f64 =
            metrics.iter().map(|m| m.items_never_reviewed as f64).sum();
        let items_never_reviewed_mean = items_never_reviewed_sum / n_f64;

        Self {
            final_score_mean,
            final_score_std,
            final_score_median,
            retention_per_minute_mean,
            retention_per_minute_std,
            coverage_pct_mean,
            coverage_pct_std,
            plan_faithfulness_mean,
            days_to_mastery_mean,
            days_to_mastery_count,
            gave_up_fraction,
            total_students: n,
            coverage_t_mean,
            mean_r_t_mean,
            rpm_t_mean,
            rpm_short_mean,
            coverage_acq_mean,
            mean_r_acq_mean,
            items_never_reviewed_mean,
        }
    }

    /// Create empty metrics.
    pub fn empty() -> Self {
        Self {
            final_score_mean: 0.0,
            final_score_std: 0.0,
            final_score_median: 0.0,
            retention_per_minute_mean: 0.0,
            retention_per_minute_std: 0.0,
            coverage_pct_mean: 0.0,
            coverage_pct_std: 0.0,
            plan_faithfulness_mean: 0.0,
            days_to_mastery_mean: None,
            days_to_mastery_count: 0,
            gave_up_fraction: 0.0,
            total_students: 0,
            coverage_t_mean: 0.0,
            mean_r_t_mean: 0.0,
            rpm_t_mean: 0.0,
            rpm_short_mean: None,
            coverage_acq_mean: 0.0,
            mean_r_acq_mean: 0.0,
            items_never_reviewed_mean: 0.0,
        }
    }
}

/// Result of running one scheduler variant.
#[derive(Debug, Clone, Serialize)]
pub struct VariantResult {
    /// Variant name
    pub variant: String,
    /// Number of students simulated
    pub students: usize,
    /// Aggregated metrics
    pub metrics: AggregatedMetrics,
    /// Individual student metrics (optional, for detailed analysis)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub individual_metrics: Vec<MetricsSummary>,

    // v0.5 additions
    /// Timeline / learning curve data (per-day aggregates)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub timeline: Vec<crate::stats::TimelinePoint>,
    /// Difficulty bucket metrics (easy/medium/hard)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub difficulty_buckets: Vec<crate::stats::DifficultyBucketMetrics>,
    /// Confidence interval stats for key metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_stats: Option<ConfidenceIntervalStats>,
    /// Multi-objective evaluation result (v2.1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation: Option<EvalResult>,
}

/// Confidence interval statistics for key metrics (v0.5).
#[derive(Debug, Clone, Serialize)]
pub struct ConfidenceIntervalStats {
    pub final_score: crate::stats::MetricStats,
    pub retention_per_minute: crate::stats::MetricStats,
    pub coverage_pct: crate::stats::MetricStats,
    pub plan_faithfulness: crate::stats::MetricStats,
}

/// Summary of individual student metrics for JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSummary {
    pub student_index: usize,
    pub final_score: f64,
    pub retention_per_minute: f64,
    pub coverage_pct: f64,
    pub days_to_mastery: Option<u32>,
    pub gave_up: bool,
}

/// Full comparison results.
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonResults {
    /// Scenario name
    pub scenario: String,
    /// Goal ID
    pub goal_id: String,
    /// Target days
    pub target_days: u32,
    /// Base seed used
    pub base_seed: u64,
    /// Results for each variant
    pub variants: Vec<VariantResult>,
    /// Statistical significance results between variant pairs (v0.5)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub significance_results: Vec<crate::stats::SignificanceResult>,
}

/// Run comparison across multiple scheduler variants.
///
/// # Arguments
/// * `content_repo` - Content repository
/// * `base_scenario` - Base scenario (scheduler field will be overridden)
/// * `variants` - Scheduler variants to compare
/// * `students_per_variant` - Number of students to simulate per variant
/// * `base_seed` - Base RNG seed
/// * `include_individual` - Whether to include individual student metrics
pub async fn run_comparison(
    content_repo: Arc<dyn ContentRepository>,
    base_scenario: &Scenario,
    variants: &[SchedulerVariant],
    students_per_variant: usize,
    base_seed: u64,
    expected_rpm: f64,
    include_individual: bool,
) -> Result<(ComparisonResults, Option<RunDebugReport>)> {
    info!(
        "Running comparison: {} variants × {} students",
        variants.len(),
        students_per_variant
    );

    let mut variant_results = Vec::with_capacity(variants.len());
    let mut all_variant_scores: Vec<(String, Vec<f64>)> = Vec::with_capacity(variants.len());
    let mut variant_reports = Vec::new(); // for debug stats

    for &variant in variants {
        info!("Running variant: {}", variant.name());

        // Create scenario with this variant
        let scenario = base_scenario.with_scheduler(variant);

        // Create config
        let config = SimulationConfig {
            scenarios: vec![scenario.clone()],
            base_seed,
            expected_rpm,
            debug_stats: true, // Force debug stats for comparison to get full reports
            ..Default::default()
        };

        // Create simulator
        let simulator = Simulator::new(Arc::clone(&content_repo), config.clone());

        // Run simulations
        let mut all_metrics = Vec::with_capacity(students_per_variant);
        let mut individual_summaries = Vec::new();
        let mut variant_debug_summaries = Vec::new();

        for i in 0..students_per_variant {
            match simulator.simulate_student(&scenario, i).await {
                Ok((metrics, summary)) => {
                    if include_individual {
                        individual_summaries.push(MetricsSummary {
                            student_index: i,
                            final_score: metrics.final_score(scenario.target_days, expected_rpm),
                            retention_per_minute: metrics.retention_per_minute,
                            coverage_pct: metrics.coverage_pct,
                            days_to_mastery: metrics.days_to_mastery,
                            gave_up: metrics.gave_up,
                        });
                    }
                    all_metrics.push(metrics);
                    if let Some(s) = summary {
                        variant_debug_summaries.push(s);
                    }
                }
                Err(e) => {
                    warn!("Simulation failed for student {}: {}", i, e);
                }
            }
        }

        // Compute debug stats for variant if summaries exist
        if !variant_debug_summaries.is_empty() {
            let report = VariantDebugReport::from_summaries(
                &variant.name().to_string(),
                &variant_debug_summaries,
            );
            variant_reports.push(report);
        }

        // Aggregate metrics
        let aggregated =
            AggregatedMetrics::compute(&all_metrics, scenario.target_days, expected_rpm);

        // Compute CI stats if there are enough students
        let ci_stats = if all_metrics.len() >= 2 {
            let final_scores: Vec<f64> = all_metrics
                .iter()
                .map(|m| m.final_score(scenario.target_days, expected_rpm))
                .collect();
            let rpms: Vec<f64> = all_metrics.iter().map(|m| m.retention_per_minute).collect();
            let coverages: Vec<f64> = all_metrics.iter().map(|m| m.coverage_pct).collect();
            let faithfulness: Vec<f64> = all_metrics.iter().map(|m| m.plan_faithfulness).collect();

            Some(ConfidenceIntervalStats {
                final_score: crate::stats::MetricStats::from_values(&final_scores),
                retention_per_minute: crate::stats::MetricStats::from_values(&rpms),
                coverage_pct: crate::stats::MetricStats::from_values(&coverages),
                plan_faithfulness: crate::stats::MetricStats::from_values(&faithfulness),
            })
        } else {
            None
        };

        // Store raw final scores for significance testing later
        let final_scores_for_sig: Vec<f64> = all_metrics
            .iter()
            .map(|m| m.final_score(scenario.target_days, expected_rpm))
            .collect();
        all_variant_scores.push((variant.name().to_string(), final_scores_for_sig));

        variant_results.push(VariantResult {
            variant: variant.name().to_string(),
            students: all_metrics.len(),
            metrics: aggregated,
            individual_metrics: individual_summaries,
            // v0.5 fields - timeline/difficulty_buckets filled on demand
            timeline: vec![],
            difficulty_buckets: vec![],
            ci_stats,
            evaluation: None, // Computed below after debug stats
        });
    }

    // Create debug report first
    let debug_report = if !variant_reports.is_empty() {
        Some(RunDebugReport {
            scenario_name: base_scenario.name.clone(),
            timestamp: Utc::now(),
            variants: variant_reports,
        })
    } else {
        None
    };

    // Compute evaluation for each variant using debug stats
    for (i, vr) in variant_results.iter_mut().enumerate() {
        if let Some(ref report) = debug_report {
            if let Some(variant_debug) = report.variants.get(i) {
                if let Some(first_student) = variant_debug.students.first() {
                    // Find matching SimulationMetrics from comparison
                    // For simplicity, use mean values (evaluation is already aggregated by construction)
                    let fake_sim_metrics = crate::metrics::SimulationMetrics {
                        retention_per_minute: vr.metrics.retention_per_minute_mean,
                        days_to_mastery: None,
                        coverage_pct: vr.metrics.coverage_pct_mean,
                        plan_faithfulness: vr.metrics.plan_faithfulness_mean,
                        total_minutes: 0.0,
                        total_days: base_scenario.target_days,
                        gave_up: false,
                        goal_item_count: (vr.metrics.items_never_reviewed_mean
                            + vr.metrics.coverage_t_mean * 100.0)
                            as usize,
                        items_mastered: (vr.metrics.coverage_t_mean * 100.0) as usize,
                        coverage_t: vr.metrics.coverage_t_mean,
                        mean_r_t: vr.metrics.mean_r_t_mean,
                        items_good_t: 0,
                        rpm_t: vr.metrics.rpm_t_mean,
                        items_good_short: None,
                        rpm_short: vr.metrics.rpm_short_mean,
                        coverage_acq: vr.metrics.coverage_acq_mean,
                        mean_r_acq: vr.metrics.mean_r_acq_mean,
                        items_never_reviewed: vr.metrics.items_never_reviewed_mean as usize,
                    };
                    let eval = evaluate(
                        &fake_sim_metrics,
                        first_student,
                        &first_student.per_item_reviews,
                        base_scenario.target_days,
                    );
                    vr.evaluation = Some(eval);
                }
            }
        }
    }

    // Calculate significance results
    let mut significance_results = Vec::new();
    if all_variant_scores.len() >= 2 {
        for i in 0..all_variant_scores.len() {
            for j in (i + 1)..all_variant_scores.len() {
                let (name_a, scores_a) = &all_variant_scores[i];
                let (name_b, scores_b) = &all_variant_scores[j];

                let p_value = crate::stats::welchs_t_test(scores_a, scores_b);
                let mean_a = scores_a.iter().sum::<f64>() / scores_a.len().max(1) as f64;
                let mean_b = scores_b.iter().sum::<f64>() / scores_b.len().max(1) as f64;

                significance_results.push(crate::stats::SignificanceResult {
                    variant_a: name_a.clone(),
                    variant_b: name_b.clone(),
                    metric: "final_score".to_string(),
                    p_value,
                    a_greater: mean_a > mean_b,
                });
            }
        }
    }

    Ok((
        ComparisonResults {
            scenario: base_scenario.name.clone(),
            goal_id: base_scenario.goal_id.clone(),
            target_days: base_scenario.target_days,
            base_seed,
            variants: variant_results,
            significance_results,
        },
        debug_report,
    ))
}

/// Compute standard deviation.
fn std_dev(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let variance =
        values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

/// Compute median of a list.
fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregated_metrics_empty() {
        let metrics = AggregatedMetrics::compute(&[], 30, 0.1);
        assert_eq!(metrics.total_students, 0);
    }

    #[test]
    fn test_aggregated_metrics_single() {
        let m = SimulationMetrics {
            retention_per_minute: 0.1,
            days_to_mastery: Some(15),
            coverage_pct: 0.8,
            plan_faithfulness: 0.9,
            total_minutes: 100.0,
            total_days: 30,
            gave_up: false,
            goal_item_count: 10,
            items_mastered: 8,
            coverage_t: 0.8,
            mean_r_t: 0.9,
            items_good_t: 8,
            rpm_t: 0.08,
            items_good_short: None,
            rpm_short: None,
            coverage_acq: 0.8,
            mean_r_acq: 0.9,
            items_never_reviewed: 0,
        };
        let agg = AggregatedMetrics::compute(&[m], 30, 0.1);

        assert_eq!(agg.total_students, 1);
        assert!((agg.coverage_pct_mean - 0.8).abs() < 0.001);
        assert_eq!(agg.days_to_mastery_count, 1);
        assert!((agg.days_to_mastery_mean.unwrap() - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_std_dev() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let std = std_dev(&values, mean);
        // Expected: sqrt(32/7) ≈ 2.138
        assert!((std - 2.138).abs() < 0.01);
    }

    #[test]
    fn test_median() {
        assert!((median(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 3.0).abs() < 0.001);
        assert!((median(&[1.0, 2.0, 3.0, 4.0]) - 2.5).abs() < 0.001);
    }
}
