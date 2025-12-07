//! Initial Placement Service for user onboarding.
//!
//! This module provides functionality to initialize a user's knowledge state
//! based on their intake questionnaire answers. It maps self-reported
//! memorization levels to FSRS memory states and KG node energies.
//!
//! # Architecture
//!
//! - [`IntakeAnswers`] - User's questionnaire responses
//! - [`InitialPlacementConfig`] - Configurable mapping parameters
//! - [`InitialPlacementService`] - Main service for applying initial placement
//! - [`InitialPlacementSummary`] - Results of the placement operation

mod config;
mod service;
mod summary;
mod types;

pub use config::InitialPlacementConfig;
pub use service::InitialPlacementService;
pub use summary::{InitialPlacementSummary, SurahPlacementResult};
pub use types::{ArabicLevel, IntakeAnswers, SurahSelfReport};

#[cfg(test)]
mod tests;
