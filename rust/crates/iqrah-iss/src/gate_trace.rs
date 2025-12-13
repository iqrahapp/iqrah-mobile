//! Gate trace instrumentation for diagnostic output (ISS v2.9).
//!
//! Collects per-day gate decision data and outputs CSV and markdown summaries.

use crate::config::{DebugTraceConfig, GateReason, GateTraceRow};
use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Collector for gate trace rows during simulation.
#[derive(Debug, Default)]
pub struct GateTraceCollector {
    rows: Vec<GateTraceRow>,
    enabled: bool,
    out_dir: String,
    scenario_name: String,
    variant_name: String,
}

impl GateTraceCollector {
    /// Create a new trace collector from config.
    pub fn new(config: &DebugTraceConfig, scenario_name: &str, variant_name: &str) -> Self {
        Self {
            rows: Vec::new(),
            enabled: config.enabled,
            out_dir: config.out_dir.clone(),
            scenario_name: scenario_name.to_string(),
            variant_name: variant_name.to_string(),
        }
    }

    /// Create a disabled collector (no-op).
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Check if trace is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add a trace row.
    pub fn add_row(&mut self, row: GateTraceRow) {
        if self.enabled {
            self.rows.push(row);
        }
    }

    /// Get collected rows (for testing).
    pub fn rows(&self) -> &[GateTraceRow] {
        &self.rows
    }

    /// Write CSV and markdown output files.
    /// Returns paths to the created files, or Vec::empty if disabled.
    pub fn write_output(&self) -> Result<Vec<String>> {
        if !self.enabled || self.rows.is_empty() {
            return Ok(Vec::new());
        }

        // Create output directory if needed
        fs::create_dir_all(&self.out_dir)?;

        let base_name = format!(
            "{}_{}",
            self.scenario_name.replace(' ', "_").to_lowercase(),
            self.variant_name.replace(' ', "_").to_lowercase()
        );

        let csv_path = Path::new(&self.out_dir).join(format!("{}_gate_trace.csv", base_name));
        let md_path = Path::new(&self.out_dir).join(format!("{}_gate_summary.md", base_name));

        self.write_csv(&csv_path)?;
        self.write_markdown(&md_path)?;

        Ok(vec![
            csv_path.to_string_lossy().to_string(),
            md_path.to_string_lossy().to_string(),
        ])
    }

    fn write_csv(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;

        // Header (M2.4: added introduction policy allowance columns, M2.5: added max_working_set_effective, M2.6: backlog-aware, M2.7: overdue fairness)
        writeln!(
            file,
            "day,due_reviews,actual_reviews,capacity_budget,budget_delta,introduced_today,introduced_total,single_review_items,new_items_limit_today,total_active,max_new_gate_param,cluster_energy,threshold,working_set_factor,capacity_used,session_size,due_budget,intro_budget,due_selected,new_selected,due_candidates_available,new_candidates_available,intro_cap,spill_to_due,spill_to_new,goal_total,unintroduced_total,new_from_get_candidates,new_pass_cluster_filter,new_candidates_in_session,gate_expand_mode,threshold_low,threshold_high,allowance_raw,allowance_after_capacity,allowance_after_workingset,allowance_after_gate,allowance_final,intro_min_per_day,intro_bootstrap_until_active,max_working_set_effective,max_ws_budget,target_reviews_per_active,intro_floor_effective,p90_due_age_days,max_p90_due_age_days,backlog_severe,overdue_candidates_count,overdue_selected_count,max_due_age_selected,gate_blocked,gate_reason"
        )?;

        // Data rows
        for row in &self.rows {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{},{},{},{:.4},{:.4},{:.4},{:.4},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.4},{:.4},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.1},{},\"{}\"",
                row.day,
                row.due_reviews,
                row.actual_reviews,
                row.capacity_budget,
                row.budget_delta,
                row.introduced_today,
                row.introduced_total,
                row.single_review_items,
                row.new_items_limit_today,
                row.total_active,
                row.max_new_gate_param,
                row.cluster_energy,
                row.threshold,
                row.working_set_factor,
                row.capacity_used,
                row.session_size,
                row.due_budget,
                row.intro_budget,
                row.due_selected,
                row.new_selected,
                row.due_candidates_available,
                row.new_candidates_available,
                row.intro_cap,
                row.spill_to_due,
                row.spill_to_new,
                row.goal_total,
                row.unintroduced_total,
                row.new_from_get_candidates,
                row.new_pass_cluster_filter,
                row.new_candidates_in_session,
                row.gate_expand_mode,
                row.threshold_low,
                row.threshold_high,
                row.allowance_raw,
                row.allowance_after_capacity,
                row.allowance_after_workingset,
                row.allowance_after_gate,
                row.allowance_final,
                row.intro_min_per_day,
                row.intro_bootstrap_until_active,
                row.max_working_set_effective,
                row.max_ws_budget.map_or("".to_string(), |v| v.to_string()),
                row.target_reviews_per_active.map_or("".to_string(), |v| format!("{:.4}", v)),
                row.intro_floor_effective,
                row.p90_due_age_days_trace,
                row.max_p90_due_age_days.map_or("".to_string(), |v| format!("{:.1}", v)),
                row.backlog_severe,
                row.overdue_candidates_count,
                row.overdue_selected_count,
                row.max_due_age_selected,
                row.gate_blocked,
                row.gate_reason
            )?;
        }

        Ok(())
    }

    fn write_markdown(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;

        let total_days = self.rows.len();
        let days_blocked = self.rows.iter().filter(|r| r.gate_blocked).count();
        let total_introduced: usize = self.rows.last().map(|r| r.introduced_total).unwrap_or(0);
        let avg_new_per_day = if total_days > 0 {
            self.rows.iter().map(|r| r.introduced_today).sum::<usize>() as f64 / total_days as f64
        } else {
            0.0
        };

        writeln!(
            file,
            "# Gate Diagnostics: {} - {}",
            self.scenario_name, self.variant_name
        )?;
        writeln!(file)?;
        writeln!(file, "## Overview")?;
        writeln!(file, "- Days simulated: {}", total_days)?;
        writeln!(
            file,
            "- Days gate blocked: {} ({:.1}%)",
            days_blocked,
            days_blocked as f64 / total_days.max(1) as f64 * 100.0
        )?;
        writeln!(file, "- Total items introduced: {}", total_introduced)?;
        writeln!(file, "- Average introductions/day: {:.2}", avg_new_per_day)?;
        writeln!(file)?;

        // Gate reason histogram
        let mut reason_counts = std::collections::HashMap::new();
        for row in &self.rows {
            *reason_counts.entry(row.gate_reason).or_insert(0usize) += 1;
        }

        writeln!(file, "## Gate Reason Distribution")?;
        writeln!(file, "| Reason | Count | Percent |")?;
        writeln!(file, "|--------|-------|---------|")?;
        for reason in [
            GateReason::None,
            GateReason::ClusterWeak,
            GateReason::WorkingSetFull,
            GateReason::CapacityExceeded,
            GateReason::RateTooLow,
        ] {
            let count = reason_counts.get(&reason).copied().unwrap_or(0);
            let pct = count as f64 / total_days.max(1) as f64 * 100.0;
            writeln!(file, "| {} | {} | {:.1}% |", reason, count, pct)?;
        }
        writeln!(file)?;

        // Signal statistics
        let energies: Vec<f64> = self.rows.iter().map(|r| r.cluster_energy).collect();
        let wsf: Vec<f64> = self.rows.iter().map(|r| r.working_set_factor).collect();
        let cap_used: Vec<f64> = self.rows.iter().map(|r| r.capacity_used).collect();

        writeln!(file, "## Signal Statistics")?;
        writeln!(file, "| Signal | Min | Mean | Max |")?;
        writeln!(file, "|--------|-----|------|-----|")?;
        writeln!(
            file,
            "| cluster_energy | {:.3} | {:.3} | {:.3} |",
            self.min(&energies),
            self.mean(&energies),
            self.max(&energies)
        )?;
        writeln!(
            file,
            "| working_set_factor | {:.3} | {:.3} | {:.3} |",
            self.min(&wsf),
            self.mean(&wsf),
            self.max(&wsf)
        )?;
        writeln!(
            file,
            "| capacity_used | {:.3} | {:.3} | {:.3} |",
            self.min(&cap_used),
            self.mean(&cap_used),
            self.max(&cap_used)
        )?;
        writeln!(file)?;

        // Day snapshots at key intervals
        writeln!(file, "## Day Snapshots")?;
        let snapshot_days = [1, 10, 30, 60, 90, 120, 150, 180];
        for &day in &snapshot_days {
            if let Some(row) = self.rows.iter().find(|r| r.day == day) {
                writeln!(file, "### Day {}", day)?;
                writeln!(file, "- due_reviews: {}", row.due_reviews)?;
                writeln!(file, "- actual_reviews: {}", row.actual_reviews)?;
                writeln!(file, "- introduced_today: {}", row.introduced_today)?;
                writeln!(file, "- introduced_total: {}", row.introduced_total)?;
                writeln!(file, "- total_active: {}", row.total_active)?;
                writeln!(file, "- cluster_energy: {:.3}", row.cluster_energy)?;
                writeln!(file, "- threshold: {:.3}", row.threshold)?;
                writeln!(file, "- capacity_used: {:.3}", row.capacity_used)?;
                writeln!(file, "- gate_reason: {}", row.gate_reason)?;
                writeln!(file, "- session_size: {}", row.session_size)?;
                writeln!(file, "- intro_budget: {}", row.intro_budget)?;
                writeln!(
                    file,
                    "- new_items_limit_today: {}",
                    row.new_items_limit_today
                )?;
                writeln!(file)?;
            }
        }

        Ok(())
    }

    fn min(&self, values: &[f64]) -> f64 {
        values.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    fn max(&self, values: &[f64]) -> f64 {
        values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    fn mean(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_collector_writes_nothing() {
        let config = DebugTraceConfig::default(); // enabled: false
        let collector = GateTraceCollector::new(&config, "test", "variant");

        // Add a row (should be ignored)
        let row = GateTraceRow {
            day: 1,
            due_reviews: 10,
            actual_reviews: 10,
            capacity_budget: 40,
            budget_delta: 30,
            introduced_today: 5,
            introduced_total: 5,
            single_review_items: 5,
            new_items_limit_today: 10,
            total_active: 5,
            max_new_gate_param: 60,
            cluster_energy: 0.5,
            gate_blocked: false,
            gate_reason: GateReason::None,
            threshold: 0.15,
            working_set_factor: 1.0,
            capacity_used: 0.2,
            session_size: 15,
            due_budget: 10,
            intro_budget: 5,
            due_selected: 10,
            new_selected: 5,
            due_candidates_available: 50,
            new_candidates_available: 100,
            intro_cap: 5,
            spill_to_due: 0,
            spill_to_new: 0,
            goal_total: 564,
            unintroduced_total: 559,
            new_from_get_candidates: 559,
            new_pass_cluster_filter: 100,
            new_candidates_in_session: 100,
            // M2.4: Introduction policy allowance stages
            gate_expand_mode: true,
            threshold_low: 0.14,
            threshold_high: 0.16,
            allowance_raw: 5,
            allowance_after_capacity: 5,
            allowance_after_workingset: 5,
            allowance_after_gate: 5,
            allowance_final: 5,
            intro_min_per_day: 3,
            intro_bootstrap_until_active: 100,
            max_working_set_effective: 50,
            // M2.6: Backlog-aware working set + floor
            max_ws_budget: Some(200),
            target_reviews_per_active: Some(0.08),
            intro_floor_effective: 3,
            p90_due_age_days_trace: 30.0,
            max_p90_due_age_days: Some(45.0),
            backlog_severe: false,
            // M2.7: Overdue fairness
            overdue_candidates_count: 100,
            overdue_selected_count: 15,
            max_due_age_selected: 25.0,
        };
        let mut collector = collector;
        collector.add_row(row);

        assert!(collector.rows().is_empty());
        assert!(collector.write_output().unwrap().is_empty());
    }

    #[test]
    fn test_enabled_collector_collects_rows() {
        let config = DebugTraceConfig {
            enabled: true,
            out_dir: "/tmp/test_trace".to_string(),
        };
        let mut collector = GateTraceCollector::new(&config, "test", "variant");

        let row = GateTraceRow {
            day: 1,
            due_reviews: 10,
            actual_reviews: 10,
            capacity_budget: 40,
            budget_delta: 30,
            introduced_today: 5,
            introduced_total: 5,
            single_review_items: 5,
            new_items_limit_today: 10,
            total_active: 5,
            max_new_gate_param: 60,
            cluster_energy: 0.5,
            gate_blocked: false,
            gate_reason: GateReason::None,
            threshold: 0.15,
            working_set_factor: 1.0,
            capacity_used: 0.2,
            session_size: 15,
            due_budget: 10,
            intro_budget: 5,
            due_selected: 10,
            new_selected: 5,
            due_candidates_available: 50,
            new_candidates_available: 100,
            intro_cap: 5,
            spill_to_due: 0,
            spill_to_new: 0,
            goal_total: 564,
            unintroduced_total: 559,
            new_from_get_candidates: 559,
            new_pass_cluster_filter: 100,
            new_candidates_in_session: 100,
            // M2.4: Introduction policy allowance stages
            gate_expand_mode: true,
            threshold_low: 0.14,
            threshold_high: 0.16,
            allowance_raw: 5,
            allowance_after_capacity: 5,
            allowance_after_workingset: 5,
            allowance_after_gate: 5,
            allowance_final: 5,
            intro_min_per_day: 3,
            intro_bootstrap_until_active: 100,
            max_working_set_effective: 50,
            // M2.6: Backlog-aware working set + floor
            max_ws_budget: Some(200),
            target_reviews_per_active: Some(0.08),
            intro_floor_effective: 3,
            p90_due_age_days_trace: 30.0,
            max_p90_due_age_days: Some(45.0),
            backlog_severe: false,
            // M2.7: Overdue fairness
            overdue_candidates_count: 100,
            overdue_selected_count: 15,
            max_due_age_selected: 25.0,
        };
        collector.add_row(row.clone());
        collector.add_row(GateTraceRow { day: 2, ..row });

        assert_eq!(collector.rows().len(), 2);
    }
}
