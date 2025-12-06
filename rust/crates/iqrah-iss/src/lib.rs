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

pub mod baselines;
pub mod brain;
pub mod comparison;
pub mod config;
pub mod in_memory_repo;
pub mod metrics;
pub mod simulator;

// Re-exports for convenience
pub use baselines::{
    FixedSrsBaseline, PageOrderBaseline, RandomBaseline, SchedulerVariant, SessionGenerator,
};
pub use brain::{PriorKnowledgeConfig, RecallResult, StudentBrain, StudentParams};
pub use comparison::{run_comparison, AggregatedMetrics, ComparisonResults, VariantResult};
pub use config::{Scenario, SimulationConfig};
pub use in_memory_repo::InMemoryUserRepository;
pub use metrics::{days_to_mastery, is_mastered, retrievability, DailySnapshot, SimulationMetrics};
pub use simulator::Simulator;
