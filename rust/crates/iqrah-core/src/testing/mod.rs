//! Test utilities and fixtures for iqrah-core tests.
//!
//! This module provides:
//! - Re-exports of mockall-generated mocks
//! - Reusable test fixtures (verses, chapters, words, etc.)
//! - Helper functions for common mock setups

pub mod fixtures;

// Re-export the mockall-generated mocks
pub use crate::ports::content_repository::MockContentRepository;
pub use crate::ports::user_repository::MockUserRepository;
