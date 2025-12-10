//! Axis semantics for ISS v2.1 presets.
//!
//! This module implements the axis configuration per spec §3:
//! - Knowledge units (WORD_INSTANCE, PHRASE_INSTANCE, AYAH_INSTANCE) may have multiple axes
//! - Each axis has its own energy and FSRS state
//! - ISS presets declare which axes to schedule and how to measure coverage
//!
//! **Invariant A1:** Every preset must explicitly declare AxisMode and coverage semantics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Axis kind representing different types of knowledge associated with a unit.
///
/// Each axis has its own learning curve and FSRS state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisKind {
    /// Acoustic/articulation memory (primary for Quran memorization)
    Memorization,
    /// Meaning/translation understanding
    Translation,
    /// Root recognition or morphological pattern
    Root,
    /// Tajwid (articulation rules)
    Tajwid,
}

impl AxisKind {
    /// Get all available axis kinds.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Memorization,
            Self::Translation,
            Self::Root,
            Self::Tajwid,
        ]
    }

    /// Get display name for this axis.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Memorization => "memorization",
            Self::Translation => "translation",
            Self::Root => "root",
            Self::Tajwid => "tajwid",
        }
    }
}

impl Default for AxisKind {
    fn default() -> Self {
        Self::Memorization
    }
}

/// Axis mode for preset configuration (per spec §3.3).
///
/// Determines how many axes are scheduled per knowledge unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AxisMode {
    /// Only schedule a single axis per unit (default for official benchmarks).
    ///
    /// Per **Invariant P1**, the official `juz_amma_dedicated` benchmark
    /// must use SingleAxis with Memorization.
    SingleAxis {
        /// The axis to schedule (e.g., Memorization)
        axis: AxisKind,
    },

    /// Schedule multiple axes separately (for experimental presets only).
    ///
    /// Each axis-item is treated as a separate schedulable entity.
    MultiAxis {
        /// List of axes to schedule
        axes: Vec<AxisKind>,
        /// Optional per-axis weights for evaluation or cost accounting
        #[serde(default)]
        axis_weights: HashMap<AxisKind, f32>,
    },
}

impl Default for AxisMode {
    fn default() -> Self {
        Self::SingleAxis {
            axis: AxisKind::Memorization,
        }
    }
}

impl AxisMode {
    /// Create a single-axis mode with the given axis.
    pub fn single(axis: AxisKind) -> Self {
        Self::SingleAxis { axis }
    }

    /// Create a multi-axis mode with equal weights.
    pub fn multi(axes: Vec<AxisKind>) -> Self {
        Self::MultiAxis {
            axes,
            axis_weights: HashMap::new(),
        }
    }

    /// Get the number of axis-items generated per knowledge unit.
    pub fn items_per_unit(&self) -> usize {
        match self {
            Self::SingleAxis { .. } => 1,
            Self::MultiAxis { axes, .. } => axes.len(),
        }
    }

    /// Check if this mode schedules the given axis.
    pub fn schedules_axis(&self, axis: AxisKind) -> bool {
        match self {
            Self::SingleAxis { axis: a } => *a == axis,
            Self::MultiAxis { axes, .. } => axes.contains(&axis),
        }
    }
}

/// Coverage mode for preset (per spec §3.3 Invariant A1).
///
/// Determines how coverage metrics are computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AxisCoverageMode {
    /// Coverage measured per knowledge unit.
    ///
    /// A unit is "covered" if its primary axis (or any axis in MultiAxis mode)
    /// reaches the acquisition/mastery threshold.
    #[default]
    PerUnit,

    /// Coverage measured per axis-item.
    ///
    /// Each axis-item must individually reach thresholds for coverage.
    PerAxis,
}

impl AxisCoverageMode {
    /// Get display name for this coverage mode.
    pub fn name(&self) -> &'static str {
        match self {
            Self::PerUnit => "per_unit",
            Self::PerAxis => "per_axis",
        }
    }
}

/// Complete axis configuration for a preset.
///
/// Combines AxisMode and AxisCoverageMode per Invariant A1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    /// How axes are scheduled
    pub mode: AxisMode,
    /// How coverage is measured
    pub coverage_mode: AxisCoverageMode,
    /// For SingleAxis mode with PerUnit coverage, which axis determines coverage
    #[serde(default)]
    pub coverage_axis: Option<AxisKind>,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            mode: AxisMode::default(),
            coverage_mode: AxisCoverageMode::default(),
            coverage_axis: Some(AxisKind::Memorization),
        }
    }
}

impl AxisConfig {
    /// Create config for official benchmarks (SingleAxis + PerUnit + Memorization).
    ///
    /// This is the required configuration per **Invariant P1**.
    pub fn benchmark() -> Self {
        Self {
            mode: AxisMode::single(AxisKind::Memorization),
            coverage_mode: AxisCoverageMode::PerUnit,
            coverage_axis: Some(AxisKind::Memorization),
        }
    }

    /// Create config for multi-axis experiments.
    pub fn experimental(axes: Vec<AxisKind>) -> Self {
        Self {
            mode: AxisMode::multi(axes),
            coverage_mode: AxisCoverageMode::PerAxis,
            coverage_axis: None,
        }
    }

    /// Validate that this config is valid for official benchmarks.
    ///
    /// Returns an error message if invalid.
    pub fn validate_for_benchmark(&self) -> Option<&'static str> {
        match &self.mode {
            AxisMode::SingleAxis { axis } => {
                if *axis != AxisKind::Memorization {
                    return Some("Official benchmarks must use Memorization axis");
                }
                if self.coverage_mode != AxisCoverageMode::PerUnit {
                    return Some("Official benchmarks must use PerUnit coverage");
                }
                None
            }
            AxisMode::MultiAxis { .. } => Some("Official benchmarks must use SingleAxis mode"),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_axis_items_per_unit() {
        let mode = AxisMode::single(AxisKind::Memorization);
        assert_eq!(mode.items_per_unit(), 1);
    }

    #[test]
    fn test_multi_axis_items_per_unit() {
        let mode = AxisMode::multi(vec![
            AxisKind::Memorization,
            AxisKind::Translation,
            AxisKind::Root,
        ]);
        assert_eq!(mode.items_per_unit(), 3);
    }

    #[test]
    fn test_schedules_axis() {
        let single = AxisMode::single(AxisKind::Memorization);
        assert!(single.schedules_axis(AxisKind::Memorization));
        assert!(!single.schedules_axis(AxisKind::Translation));

        let multi = AxisMode::multi(vec![AxisKind::Memorization, AxisKind::Translation]);
        assert!(multi.schedules_axis(AxisKind::Memorization));
        assert!(multi.schedules_axis(AxisKind::Translation));
        assert!(!multi.schedules_axis(AxisKind::Root));
    }

    #[test]
    fn test_benchmark_config_valid() {
        let config = AxisConfig::benchmark();
        assert!(config.validate_for_benchmark().is_none());
    }

    #[test]
    fn test_experimental_config_invalid_for_benchmark() {
        let config = AxisConfig::experimental(vec![AxisKind::Memorization, AxisKind::Translation]);
        assert!(config.validate_for_benchmark().is_some());
    }

    #[test]
    fn test_default_is_benchmark() {
        let config = AxisConfig::default();
        assert!(config.validate_for_benchmark().is_none());
    }

    #[test]
    fn test_axis_kind_serialization() {
        let axis = AxisKind::Memorization;
        let json = serde_json::to_string(&axis).unwrap();
        assert_eq!(json, "\"memorization\"");
    }

    #[test]
    fn test_axis_mode_serialization() {
        let mode = AxisMode::single(AxisKind::Translation);
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains("\"type\":\"single_axis\""));
        assert!(json.contains("\"translation\""));
    }
}
