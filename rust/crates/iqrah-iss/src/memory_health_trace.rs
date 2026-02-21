//! M3 Memory Health Trace - Daily aggregates to explain retention gaps.
//!
//! Outputs a separate CSV file (memory_health_trace.csv) from gate trace to track:
//! - Energy/stability distributions (mean, p10)
//! - Retrievability at today horizon
//! - Review pressure (reviews per active item)
//! - Backlog severity (due age percentiles)

use crate::config::DebugTraceConfig;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Daily memory health aggregates.
#[derive(Debug, Clone, Default)]
pub struct MemoryHealthRow {
    pub day: u32,

    // Energy distribution
    pub mean_energy: f64,
    pub p10_energy: f64,

    // Stability distribution (FSRS)
    pub mean_stability: f64,
    pub p10_stability: f64,

    // Retrievability at day end (R(S, 0) - same-day horizon)
    pub mean_retrievability_today: f64,

    // Recall probability for items actually reviewed today
    pub mean_p_recall_reviewed_today: f64,

    // Review pressure: today's reviews / total_active
    pub mean_reviews_per_active_item_today: f64,

    // Backlog severity: p50/p90 days since last review for due items
    pub p50_due_age_days: f64,
    pub p90_due_age_days: f64,

    // Counts
    pub total_active: usize,
    pub items_reviewed_today: usize,
    pub items_due_today: usize,

    // M2.8: At-risk backlog metrics (retrievability-aware)
    /// Items with R_today < 0.80 (strict threshold)
    pub at_risk_count: usize,
    /// at_risk_count / total_active (strict)
    pub at_risk_ratio: f64,
    /// 10th percentile of R_today among active items (tail health)
    pub p10_r_today: f64,
    /// p90 due_age only among at-risk (R<0.80) items
    pub p90_due_age_at_risk: f64,
    // M2.9: Weaker at-risk threshold (R < 0.90)
    /// Items with R_today < 0.90
    pub at_risk_count_0_9: usize,
    /// at_risk_count_0_9 / total_active
    pub at_risk_ratio_0_9: f64,
}

/// Collector for memory health trace data.
pub struct MemoryHealthTraceCollector {
    enabled: bool,
    out_dir: PathBuf,
    scenario_name: String,
    variant_name: String,
    rows: Vec<MemoryHealthRow>,
}

impl MemoryHealthTraceCollector {
    /// Create a new collector (respects config.enabled).
    pub fn new(config: &DebugTraceConfig, scenario_name: &str, variant_name: &str) -> Self {
        Self {
            enabled: config.enabled,
            out_dir: PathBuf::from(&config.out_dir),
            scenario_name: scenario_name.to_string(),
            variant_name: variant_name.to_string(),
            rows: Vec::new(),
        }
    }

    /// Add a row (only if enabled).
    pub fn add_row(&mut self, row: MemoryHealthRow) {
        if self.enabled {
            self.rows.push(row);
        }
    }

    /// Get rows for testing.
    pub fn rows(&self) -> &[MemoryHealthRow] {
        &self.rows
    }

    /// Write output files. Returns paths written.
    pub fn write_output(&self) -> Result<Vec<PathBuf>> {
        if !self.enabled || self.rows.is_empty() {
            return Ok(Vec::new());
        }

        std::fs::create_dir_all(&self.out_dir)?;

        let csv_filename = format!(
            "{}_{}_memory_health_trace.csv",
            self.scenario_name, self.variant_name
        );
        let csv_path = self.out_dir.join(&csv_filename);
        self.write_csv(&csv_path)?;

        Ok(vec![csv_path])
    }

    /// Write CSV file.
    fn write_csv(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;

        // Header (M2.8: added at-risk metrics)
        writeln!(
            file,
            "day,mean_energy,p10_energy,mean_stability,p10_stability,mean_retrievability_today,mean_p_recall_reviewed_today,mean_reviews_per_active_item_today,p50_due_age_days,p90_due_age_days,total_active,items_reviewed_today,items_due_today,at_risk_count,at_risk_ratio,p10_R_today,p90_due_age_at_risk,at_risk_count_0_9,at_risk_ratio_0_9"
        )?;

        // Data rows
        for row in &self.rows {
            writeln!(
                file,
                "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.1},{:.1},{},{},{},{},{:.4},{:.4},{:.1},{},{:.4}",
                row.day,
                row.mean_energy,
                row.p10_energy,
                row.mean_stability,
                row.p10_stability,
                row.mean_retrievability_today,
                row.mean_p_recall_reviewed_today,
                row.mean_reviews_per_active_item_today,
                row.p50_due_age_days,
                row.p90_due_age_days,
                row.total_active,
                row.items_reviewed_today,
                row.items_due_today,
                row.at_risk_count,
                row.at_risk_ratio,
                row.p10_r_today,
                row.p90_due_age_at_risk,
                row.at_risk_count_0_9,
                row.at_risk_ratio_0_9,
            )?;
        }

        Ok(())
    }
}

// ============================================================================
// Helper functions for computing aggregates
// ============================================================================

/// Compute mean of a slice.
pub fn compute_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Compute percentile (0-100). Returns 0.0 for empty slice.
/// Uses linear interpolation for fractional indices.
pub fn compute_percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Use nearest-rank method for percentile
    let rank = (percentile / 100.0) * (sorted.len() as f64 - 1.0);
    let lower_idx = rank.floor() as usize;
    let upper_idx = rank.ceil() as usize;

    if lower_idx == upper_idx {
        sorted[lower_idx]
    } else {
        let frac = rank - lower_idx as f64;
        sorted[lower_idx] * (1.0 - frac) + sorted[upper_idx] * frac
    }
}

/// Compute p10 (10th percentile).
pub fn compute_p10(values: &[f64]) -> f64 {
    compute_percentile(values, 10.0)
}

/// Compute p50 (median).
pub fn compute_p50(values: &[f64]) -> f64 {
    compute_percentile(values, 50.0)
}

/// Compute p90 (90th percentile).
pub fn compute_p90(values: &[f64]) -> f64 {
    compute_percentile(values, 90.0)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_collector_writes_no_files() {
        let config = DebugTraceConfig::default(); // enabled: false
        let mut collector = MemoryHealthTraceCollector::new(&config, "test", "variant");

        // Add a row (should be ignored)
        let row = MemoryHealthRow {
            day: 1,
            mean_energy: 0.5,
            p10_energy: 0.2,
            mean_stability: 10.0,
            p10_stability: 2.0,
            mean_retrievability_today: 0.85,
            mean_p_recall_reviewed_today: 0.75,
            mean_reviews_per_active_item_today: 1.5,
            p50_due_age_days: 3.0,
            p90_due_age_days: 7.0,
            total_active: 100,
            items_reviewed_today: 20,
            items_due_today: 15,
            // M2.8
            at_risk_count: 10,
            at_risk_ratio: 0.10,
            p10_r_today: 0.65,
            p90_due_age_at_risk: 25.0,
            // M2.9
            at_risk_count_0_9: 15,
            at_risk_ratio_0_9: 0.15,
        };
        collector.add_row(row);

        assert!(collector.rows().is_empty());
        assert!(collector.write_output().unwrap().is_empty());
    }

    #[test]
    fn test_enabled_collector_correct_line_count() {
        let config = DebugTraceConfig {
            enabled: true,
            out_dir: "/tmp/test_memory_trace".to_string(),
        };
        let mut collector = MemoryHealthTraceCollector::new(&config, "test", "variant");

        let row = MemoryHealthRow {
            day: 1,
            mean_energy: 0.5,
            p10_energy: 0.2,
            mean_stability: 10.0,
            p10_stability: 2.0,
            mean_retrievability_today: 0.85,
            mean_p_recall_reviewed_today: 0.75,
            mean_reviews_per_active_item_today: 1.5,
            p50_due_age_days: 3.0,
            p90_due_age_days: 7.0,
            total_active: 100,
            items_reviewed_today: 20,
            items_due_today: 15,
            // M2.8
            at_risk_count: 10,
            at_risk_ratio: 0.10,
            p10_r_today: 0.65,
            p90_due_age_at_risk: 25.0,
            // M2.9
            at_risk_count_0_9: 15,
            at_risk_ratio_0_9: 0.15,
        };
        collector.add_row(row.clone());
        collector.add_row(MemoryHealthRow {
            day: 2,
            ..row.clone()
        });
        collector.add_row(MemoryHealthRow { day: 3, ..row });

        assert_eq!(collector.rows().len(), 3);
    }

    #[test]
    fn test_p10_aggregation_correctness() {
        // p10 of [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]:
        // rank = 0.1 * 9 = 0.9, so p10 = 1 * 0.1 + 2 * 0.9 = 1.9
        let values: Vec<f64> = (1..=10).map(|x| x as f64).collect();
        let p10 = compute_p10(&values);
        assert!((p10 - 1.9).abs() < 0.1, "p10 should be ~1.9, got {}", p10);

        // p50: rank = 0.5 * 9 = 4.5, so p50 = 5 * 0.5 + 6 * 0.5 = 5.5
        let p50 = compute_p50(&values);
        assert!((p50 - 5.5).abs() < 0.1, "p50 should be ~5.5, got {}", p50);

        // p90: rank = 0.9 * 9 = 8.1, so p90 = 9 * 0.9 + 10 * 0.1 = 9.1
        let p90 = compute_p90(&values);
        assert!((p90 - 9.1).abs() < 0.1, "p90 should be ~9.1, got {}", p90);
    }

    #[test]
    fn test_mean_empty_slice() {
        assert_eq!(compute_mean(&[]), 0.0);
    }

    #[test]
    fn test_percentile_empty_slice() {
        assert_eq!(compute_p10(&[]), 0.0);
        assert_eq!(compute_p50(&[]), 0.0);
        assert_eq!(compute_p90(&[]), 0.0);
    }
}
