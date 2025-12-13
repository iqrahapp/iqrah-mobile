//! Simulation configuration and scenario definitions.
//!
//! Supports loading scenarios from YAML files for reproducible experiments.

use crate::axis::AxisConfig;
use crate::baselines::SchedulerVariant;
use crate::brain::{StudentParams, StudentParamsSelector, StudentProfile};
use anyhow::Result;
use iqrah_core::scheduler_v2::SessionMixConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Default scheduler seed offset (1 million apart from student seeds).
fn default_scheduler_seed_offset() -> u64 {
    1_000_000
}

/// Main simulation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// List of scenarios to run
    pub scenarios: Vec<Scenario>,

    /// Base RNG seed for reproducibility
    pub base_seed: u64,

    /// Offset added to base_seed for scheduler RNG (default: 1_000_000)
    #[serde(default = "default_scheduler_seed_offset")]
    pub scheduler_seed_offset: u64,

    /// Expected retention per minute (for score normalization)
    #[serde(default = "default_expected_rpm")]
    pub expected_rpm: f64,

    /// Target mastery fraction for days_to_mastery metric
    #[serde(default = "default_mastery_target")]
    pub mastery_target: f64,

    /// Collect detailed debug statistics (default: false)
    #[serde(default)]
    pub debug_stats: bool,

    /// Enable event logging for diagnostics (default: false)
    #[serde(default)]
    pub event_log_enabled: bool,

    /// Window in days for "almost due" items to be included in candidate pool (default: 2)
    /// Set to 0 to disable almost-due inclusion (original behavior)
    #[serde(default = "default_almost_due_window")]
    pub almost_due_window_days: u32,

    /// Retrievability threshold for mastery (default: 0.9)
    /// Lower to 0.7 for more realistic coverage expectations.
    #[serde(default = "default_mastery_r_threshold")]
    pub mastery_r_threshold: f64,

    /// M1.2: Debug trace configuration (gate diagnostics)
    #[serde(default)]
    pub debug_trace: DebugTraceConfig,
}

fn default_expected_rpm() -> f64 {
    0.1 // 0.1 items mastered per minute = 6 items/hour
}

fn default_mastery_target() -> f64 {
    0.8 // 80% of items mastered
}

fn default_almost_due_window() -> u32 {
    2 // Include items due within 2 days as candidates (legacy, now all items included)
}

fn default_mastery_r_threshold() -> f64 {
    0.9 // R >= 0.9 required for mastery (strict)
}

// ============================================================================
// ISS v2.9: Debug Trace Configuration
// ============================================================================

/// Configuration for diagnostic trace output (ISS v2.9).
/// When enabled, emits detailed CSV and markdown summaries for gate analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DebugTraceConfig {
    /// Whether to enable trace output (default: false)
    #[serde(default)]
    pub enabled: bool,

    /// Output directory for trace files (default: "./trace_output")
    #[serde(default = "default_trace_out_dir")]
    pub out_dir: String,
}

fn default_trace_out_dir() -> String {
    "./trace_output".to_string()
}

/// Reason why cluster gate blocked introduction (ISS v2.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GateReason {
    /// No gate blocked - introduction allowed
    #[default]
    None,
    /// Cluster energy below stability threshold
    ClusterWeak,
    /// Working set at capacity
    WorkingSetFull,
    /// Session capacity exceeded
    CapacityExceeded,
    /// Computed intro rate too low
    RateTooLow,
}

impl std::fmt::Display for GateReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateReason::None => write!(f, "allowed"),
            GateReason::ClusterWeak => write!(f, "cluster_weak"),
            GateReason::WorkingSetFull => write!(f, "working_set_full"),
            GateReason::CapacityExceeded => write!(f, "capacity_exceeded"),
            GateReason::RateTooLow => write!(f, "rate_too_low"),
        }
    }
}

/// A single row of gate trace data for CSV output.
///
/// M2.0 SEMANTICS CLARIFICATION:
/// - `capacity_budget`: Soft budget for capacity ratio computation (NOT a hard cap)
/// - `actual_reviews`: Items actually reviewed (can exceed capacity_budget by design)
/// - `budget_delta`: capacity_budget - actual_reviews (can be negative)
/// - `max_new_gate_param`: max_working_set param value (only gates NEW introductions)
/// - `total_active`: Items with review_count > 0 (can exceed max_new_gate_param)
///
/// M2.1 TRACE CORRECTNESS:
/// - `introduced_today`: Delta of introduced_total vs previous day (CORRECT metric)
/// - `introduced_total`: Total items introduced so far (authoritative)
/// - `single_review_items`: Items with review_count == 1 (old misdefinition, kept for reference)
/// - `new_items_limit_today`: Actual cap on new items computed this day
/// - Session budget columns: session_size, due_budget, intro_budget, due_selected, new_selected
///
/// M2.2 BUDGET ENFORCEMENT:
/// - `due_candidates_available`: Due/overdue items available for selection
/// - `new_candidates_available`: Unintroduced items available for selection
/// - `intro_cap`: min(intro_budget, new_items_limit_today) - actual cap on new selection
/// - `spill_to_due`: Unused intro slots that went to due items
/// - `spill_to_new`: Unused due slots that went to new items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateTraceRow {
    pub day: u32,
    /// Items scheduled for review (same as actual_reviews in current impl)
    pub due_reviews: usize,
    /// Actual items reviewed this day (NOT capped by capacity_budget)
    pub actual_reviews: usize,
    /// Soft capacity budget for utilization ratio (NOT a hard limit)
    pub capacity_budget: usize,
    /// capacity_budget - actual_reviews (can be negative by design)
    pub budget_delta: i32,

    // M2.1: Corrected introduction metrics
    /// Delta of introduced_total vs previous day (what was ACTUALLY introduced today)
    pub introduced_today: usize,
    /// Total items introduced so far (= introduction_order.len())
    pub introduced_total: usize,
    /// Items with review_count == 1 (kept for backwards compat, but NOT "introduced today")
    pub single_review_items: usize,
    /// Cap on new items computed for this day (from gate logic)
    pub new_items_limit_today: usize,

    /// Items with review_count > 0 (can exceed max_new_gate_param)
    pub total_active: usize,
    /// max_working_set param (only gates NEW introductions, not total active)
    pub max_new_gate_param: usize,
    pub cluster_energy: f64,
    pub gate_blocked: bool,
    pub gate_reason: GateReason,
    pub threshold: f64,
    pub working_set_factor: f64,
    pub capacity_used: f64,

    // M2.1: Session composition budgets
    /// Total session size from sample_daily_reviews()
    pub session_size: usize,
    /// Budget allocated to due items (session_size - intro_budget)
    pub due_budget: usize,
    /// Budget allocated to new items (clamped by intro_min_per_day, max_new_items_per_day)
    pub intro_budget: usize,
    /// Actual due items selected
    pub due_selected: usize,
    /// Actual new items selected
    pub new_selected: usize,

    // M2.2: Budget enforcement verification
    /// Due/overdue items available for selection
    pub due_candidates_available: usize,
    /// Unintroduced items available for selection
    pub new_candidates_available: usize,
    /// Effective intro cap = min(intro_budget, new_items_limit_today)
    pub intro_cap: usize,
    /// Unused intro slots that went to due items
    pub spill_to_due: usize,
    /// Unused due slots that went to new items
    pub spill_to_new: usize,

    // M2.3: Candidate funnel diagnostics
    /// Total goal items
    pub goal_total: usize,
    /// Unintroduced items = goal_total - introduced_total
    pub unintroduced_total: usize,
    /// Unintroduced items that passed get_candidates (always == unintroduced for ISS)
    pub new_from_get_candidates: usize,
    /// Unintroduced items that passed cluster filter (review_count == 0 check)
    pub new_pass_cluster_filter: usize,
    /// Final new candidates in session selection partition
    pub new_candidates_in_session: usize,

    // M2.4: Introduction policy explicit clamp stages
    /// Gate expand mode (true = allow expansion, false = consolidate)
    pub gate_expand_mode: bool,
    /// Threshold low boundary = threshold - hysteresis
    pub threshold_low: f64,
    /// Threshold high boundary = threshold + hysteresis
    pub threshold_high: f64,
    /// Stage 0: Base batch size (raw allowance)
    pub allowance_raw: usize,
    /// Stage 1: After capacity throttle
    pub allowance_after_capacity: usize,
    /// Stage 2: After working-set clamp (HARD)
    pub allowance_after_workingset: usize,
    /// Stage 3: After gate clamp
    pub allowance_after_gate: usize,
    /// Stage 4: Final after floor
    pub allowance_final: usize,

    // M1: intro policy params (for debugging)
    pub intro_min_per_day: usize,
    pub intro_bootstrap_until_active: usize,
    // M2.5: working-set ratio-of-goal trace
    pub max_working_set_effective: usize,
    // M2.6: Backlog-aware working set + floor trace
    pub max_ws_budget: Option<usize>,
    pub target_reviews_per_active: Option<f64>,
    pub intro_floor_effective: usize,
    pub p90_due_age_days_trace: f64,
    pub max_p90_due_age_days: Option<f64>,
    pub backlog_severe: bool,
    // M2.7: Overdue fairness diagnostics
    pub overdue_candidates_count: usize,
    pub overdue_selected_count: usize,
    pub max_due_age_selected: f64,
}

/// Diagnostic output from compute_sustainable_intro_rate (ISS v2.9 M1.2).
/// Filled in by the intro decision function to avoid duplicating gating logic.
#[derive(Debug, Clone, Default)]
pub struct GateDiagnostics {
    /// Cluster average energy (weighted)
    pub cluster_energy: f64,
    /// Stability threshold used for gating
    pub threshold: f64,
    /// Working set factor (1.0 when plenty of room, 0.0 at limit)
    pub working_set_factor: f64,
    /// Session capacity utilization ratio (maintenance_burden / session_capacity)
    pub capacity_used: f64,
    /// Whether intro was blocked by any gate
    pub gate_blocked: bool,
    /// Primary reason for blocking (if blocked)
    pub gate_reason: GateReason,
    /// Number of active items (review_count > 0)
    pub active_count: usize,
    /// Maximum working set size from params
    pub max_working_set: usize,
}

impl SimulationConfig {
    /// Load configuration from a YAML file.
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to a YAML file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = serde_yaml::to_string(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Get the student seed for a given student index.
    pub fn student_seed(&self, student_index: usize) -> u64 {
        self.base_seed + student_index as u64
    }

    /// Get the scheduler seed.
    pub fn scheduler_seed(&self) -> u64 {
        self.base_seed + self.scheduler_seed_offset
    }

    /// Create a minimal config for testing.
    pub fn minimal_test() -> Self {
        Self {
            scenarios: vec![Scenario::minimal_test()],
            base_seed: 42,
            scheduler_seed_offset: 1_000_000,
            expected_rpm: 0.1,
            mastery_target: 0.8,
            debug_stats: false,
            event_log_enabled: false,
            almost_due_window_days: 2,
            mastery_r_threshold: 0.9,
            debug_trace: DebugTraceConfig::default(),
        }
    }
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            scenarios: vec![Scenario::default()],
            base_seed: 42,
            scheduler_seed_offset: 1_000_000,
            expected_rpm: 0.1,
            mastery_target: 0.8,
            debug_stats: false,
            event_log_enabled: false,
            almost_due_window_days: 2,
            mastery_r_threshold: 0.9,
            debug_trace: DebugTraceConfig::default(),
        }
    }
}

/// A single simulation scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Human-readable name for this scenario
    pub name: String,

    /// Goal ID from content.db (e.g., "surah:1" or "juz:30")
    pub goal_id: String,

    /// Number of simulated days
    pub target_days: u32,

    /// Daily time budget in minutes
    pub daily_minutes: f64,

    /// Fixed student cognitive parameters (for backward compatibility)
    /// If student_params_selector is set, this is ignored.
    #[serde(default)]
    pub student_params: StudentParams,

    /// Optional: selector for heterogeneous student populations
    /// If not set, uses student_params for all students.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub student_params_selector: Option<StudentParamsSelector>,

    /// Number of items per session
    pub session_size: usize,

    /// Whether to enable bandit-driven profile selection
    #[serde(default)]
    pub enable_bandit: bool,

    /// Number of students to simulate (for batch runs)
    #[serde(default = "default_student_count")]
    pub student_count: usize,

    /// Scheduler variant to use
    #[serde(default)]
    pub scheduler: SchedulerVariant,

    /// Optional session mix configuration override.
    /// If not set, ISS computes based on plan size and horizon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_mix: Option<SessionMixConfig>,

    /// Axis configuration for this scenario (per ISS v2.1 spec §3).
    /// Determines which axes to schedule and how to measure coverage.
    /// Default: SingleAxis(Memorization) + PerUnit coverage
    #[serde(default)]
    pub axis_config: AxisConfig,

    /// Named student profile to use (ISS v2.3).
    /// Takes precedence over student_params and student_params_selector.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub student_profile: Option<StudentProfile>,

    /// Exercise configurations for v2.8 exercise framework.
    /// Optional: scenarios without this field work as v2.7 (backward compatible).
    #[serde(default)]
    pub exercises: Vec<crate::exercises::ExerciseConfig>,
}

fn default_student_count() -> usize {
    1
}

impl Scenario {
    /// Create a minimal test scenario.
    pub fn minimal_test() -> Self {
        Self {
            name: "minimal_test".to_string(),
            goal_id: "surah:1".to_string(), // Al-Fatiha (7 verses)
            target_days: 5,
            daily_minutes: 10.0,
            student_params: StudentParams::default(),
            student_params_selector: None,
            session_size: 5,
            enable_bandit: false,
            student_count: 1,
            scheduler: SchedulerVariant::IqrahDefault,
            session_mix: None,
            axis_config: AxisConfig::benchmark(),
            student_profile: None,
            exercises: vec![],
        }
    }

    /// Create a 30-day single student scenario.
    pub fn single_student_30_days() -> Self {
        Self {
            name: "single_student_30_days".to_string(),
            goal_id: "surah:1".to_string(),
            target_days: 30,
            daily_minutes: 30.0,
            student_params: StudentParams::default(),
            student_params_selector: None,
            session_size: 10,
            enable_bandit: true,
            student_count: 1,
            scheduler: SchedulerVariant::IqrahDefault,
            session_mix: None,
            axis_config: AxisConfig::benchmark(),
            student_profile: None,
            exercises: vec![],
        }
    }

    /// Create a casual learner scenario.
    pub fn casual_learner() -> Self {
        Self {
            name: "casual_learner".to_string(),
            goal_id: "surah:114".to_string(), // An-Nas (short)
            target_days: 30,
            daily_minutes: 15.0,
            student_params: StudentParams::casual_learner(),
            student_params_selector: None,
            session_size: 5,
            enable_bandit: true,
            student_count: 1,
            scheduler: SchedulerVariant::IqrahDefault,
            session_mix: None,
            axis_config: AxisConfig::benchmark(),
            student_profile: None,
            exercises: vec![],
        }
    }

    /// Create a dedicated student scenario.
    pub fn dedicated_student() -> Self {
        Self {
            name: "dedicated_student".to_string(),
            goal_id: "juz:30".to_string(), // Juz Amma
            target_days: 90,
            daily_minutes: 60.0,
            student_params: StudentParams::dedicated_student(),
            student_params_selector: None,
            session_size: 20,
            enable_bandit: true,
            student_count: 1,
            scheduler: SchedulerVariant::IqrahDefault,
            session_mix: None,
            axis_config: AxisConfig::benchmark(),
            student_profile: None,
            exercises: vec![],
        }
    }

    /// Create a copy with a different scheduler variant.
    pub fn with_scheduler(&self, scheduler: SchedulerVariant) -> Self {
        let mut clone = self.clone();
        clone.scheduler = scheduler;
        clone.name = format!("{}_{}", self.name, scheduler.name());
        clone
    }

    /// Get student parameters for a specific student index.
    ///
    /// Priority: student_profile > student_params_selector > student_params
    pub fn get_student_params(&self, student_index: usize, base_seed: u64) -> StudentParams {
        // Priority 1: Named profile (ISS v2.3)
        if let Some(profile) = &self.student_profile {
            return profile.to_params();
        }
        // Priority 2: Heterogeneous selector
        if let Some(selector) = &self.student_params_selector {
            return selector.sample_for_student(student_index, base_seed);
        }
        // Priority 3: Fixed params
        self.student_params.clone()
    }

    /// Create scenario with a specific student profile.
    pub fn with_profile(&self, profile: StudentProfile) -> Self {
        let mut clone = self.clone();
        clone.student_profile = Some(profile);
        clone
    }
}

impl Default for Scenario {
    fn default() -> Self {
        Self::single_student_30_days()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_yaml_round_trip() {
        let config = SimulationConfig::minimal_test();

        // Write to temp file
        let mut file = NamedTempFile::new().unwrap();
        let yaml = serde_yaml::to_string(&config).unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        // Read back
        let loaded = SimulationConfig::load(file.path()).unwrap();

        assert_eq!(loaded.base_seed, config.base_seed);
        assert_eq!(loaded.scenarios.len(), 1);
        assert_eq!(loaded.scenarios[0].name, "minimal_test");
    }

    #[test]
    fn test_seed_generation() {
        let config = SimulationConfig {
            base_seed: 100,
            scheduler_seed_offset: 1_000_000,
            ..Default::default()
        };

        assert_eq!(config.student_seed(0), 100);
        assert_eq!(config.student_seed(1), 101);
        assert_eq!(config.student_seed(99), 199);
        assert_eq!(config.scheduler_seed(), 1_000_100);
    }

    #[test]
    fn test_scenario_defaults() {
        let scenario = Scenario::default();
        assert_eq!(scenario.target_days, 30);
        assert_eq!(scenario.session_size, 10);
        assert!(scenario.enable_bandit);
    }

    #[test]
    fn test_dedicated_student_uses_benchmark_axis() {
        let scenario = Scenario::dedicated_student();
        // Should return None (indicating valid)
        assert!(
            scenario.axis_config.validate_for_benchmark().is_none(),
            "Dedicated student scenario must use benchmark axis configuration"
        );
    }
}

/// Compute min_new_per_session based on plan size and horizon.
///
// NOTE: compute_min_new_for_plan() was removed in ISS v2.3
// Replaced by dynamic compute_sustainable_intro_rate() in simulator.rs

/// Compute almost_due_window_days to keep recently-introduced items in candidate pool.
///
/// For large plans, FSRS schedules items 3-30 days out after first review.
/// If almost_due_window is too small (default 2 days), those items disappear
/// from candidates, causing coverage stalls.
///
/// ISS v2.2: Use smooth continuous formula, no hardcoded thresholds.
/// Formula: window = min(horizon, max(7, cycle_days * 4))
///          where cycle_days = session_size / intro_rate
///
/// This naturally scales from ~7 days for tiny plans to full horizon for huge plans.
pub fn compute_almost_due_window(
    total_items: usize,
    horizon_days: u32,
    session_size: usize,
) -> u32 {
    if total_items == 0 || horizon_days == 0 {
        return 30; // Safe default for edge cases
    }

    // REMOVED (ISS v2.2 cleanup): Hardcoded >100 item threshold.
    // Now using smooth continuous formula that scales naturally.

    let effective_days = (horizon_days as f32 * 0.80).max(1.0);
    let intro_rate = total_items as f32 / effective_days;
    let cycle_days = session_size as f32 / intro_rate.max(0.1);

    // Window = cycle * 4 (covers multiple FSRS review cycles)
    // Minimum 7 days, maximum full horizon
    let window = (cycle_days * 4.0).round().max(7.0) as u32;
    window.min(horizon_days)
}
