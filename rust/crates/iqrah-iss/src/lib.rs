//! Iqrah Student Simulations (ISS) - Scheduler Evaluation Framework
//!
//! ISS is a simulation framework that runs virtual students through the real
//! Iqrah scheduling pipeline to evaluate scheduler effectiveness and derive
//! meaningful efficiency metrics.
//!
//! # Architecture
//!
//! ISS **orchestrates** the simulation; `iqrah-core` **decides** what to schedule.
//! This means ISS does NOT reimplement scheduling or FSRS logic.
//!
//! # Key Components
//!
//! - [`StudentBrain`]: Cognitive model for simulating student recall behavior
//! - [`InMemoryUserRepository`]: Fast in-memory storage implementing UserRepository trait
//! - [`SimulationMetrics`]: Precise metric computation (retention, coverage, faithfulness)
//! - [`Simulator`]: Main orchestrator for running simulations
//! - [`SchedulerVariant`]: Enum for selecting scheduler (Iqrah vs baselines)
//! - [`run_comparison`]: Run multiple scheduler variants and aggregate metrics
//!
//! # v0.5 Statistical Analysis
//!
//! - [`MetricStats`]: Aggregated stats with 95% confidence intervals
//! - [`TimelinePoint`]: Learning curve data points
//! - [`SignificanceResult`]: Welch's t-test results between variants
//! - [`DifficultyBucketMetrics`]: Performance breakdown by difficulty

pub mod baselines;
pub mod brain;
pub mod comparison;
pub mod config;
pub mod debug_stats;
pub mod in_memory_repo;
pub mod metrics;
pub mod simulator;
pub mod stats;

// Re-exports for convenience
pub use baselines::{
    FixedSrsBaseline, PageOrderBaseline, RandomBaseline, SchedulerVariant, SessionGenerator,
};
pub use brain::{
    ParamRange, ParamVariation, PriorKnowledgeConfig, RecallResult, StudentBrain, StudentParams,
    StudentParamsSelector,
};
pub use comparison::{run_comparison, AggregatedMetrics, ComparisonResults, VariantResult};
pub use config::{Scenario, SimulationConfig};
pub use in_memory_repo::InMemoryUserRepository;
pub use metrics::{days_to_mastery, is_mastered, retrievability, DailySnapshot, SimulationMetrics};
pub use simulator::Simulator;
pub use stats::{
    compute_difficulty_buckets, welchs_t_test, DifficultyBucket, DifficultyBucketMetrics,
    MetricStats, SignificanceResult, StudentDailyPoint, TimelinePoint,
};
