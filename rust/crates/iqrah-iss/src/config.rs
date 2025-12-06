//! Simulation configuration and scenario definitions.
//!
//! Supports loading scenarios from YAML files for reproducible experiments.

use crate::baselines::SchedulerVariant;
use crate::brain::StudentParams;
use anyhow::Result;
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
}

fn default_expected_rpm() -> f64 {
    0.1 // 0.1 items mastered per minute = 6 items/hour
}

fn default_mastery_target() -> f64 {
    0.8 // 80% of items mastered
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

    /// Student cognitive parameters
    pub student_params: StudentParams,

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
            session_size: 5,
            enable_bandit: false,
            student_count: 1,
            scheduler: SchedulerVariant::IqrahDefault,
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
            session_size: 10,
            enable_bandit: true,
            student_count: 1,
            scheduler: SchedulerVariant::IqrahDefault,
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
            session_size: 5,
            enable_bandit: true,
            student_count: 1,
            scheduler: SchedulerVariant::IqrahDefault,
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
            session_size: 20,
            enable_bandit: true,
            student_count: 1,
            scheduler: SchedulerVariant::IqrahDefault,
        }
    }

    /// Create a copy with a different scheduler variant.
    pub fn with_scheduler(&self, scheduler: SchedulerVariant) -> Self {
        let mut clone = self.clone();
        clone.scheduler = scheduler;
        clone.name = format!("{}_{}", self.name, scheduler.name());
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
}
