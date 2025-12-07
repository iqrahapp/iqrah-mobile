/// Scheduler v2.0 - Advanced Prerequisite-Aware Scheduler
///
/// This module implements a sophisticated scheduling system with:
/// - Prerequisite Mastery Gate: Ensures prerequisites are mastered before scheduling dependent concepts
/// - Multi-factor Priority Scoring: Combines urgency, readiness, foundation, and influence
/// - Session Composition: Intelligent difficulty mixing (60% easy, 30% medium, 10% hard)
/// - Bandit Optimization: Thompson Sampling for hyper-personalized user profiles
/// - Session Modes: Revision (review only) and MixedLearning (new + review)
///
/// # Architecture
///
/// The scheduler follows a three-stage pipeline:
///
/// 1. **Data Retrieval**: Fetch candidate nodes, prerequisites, and user state from repositories
/// 2. **Filtering & Scoring**: Apply prerequisite gate, calculate readiness and priority scores
/// 3. **Composition**: Select nodes based on difficulty/mastery distribution and session size
///
/// # Example
///
/// ```rust,ignore
/// use iqrah_core::scheduler_v2::*;
///
/// let profile = UserProfile::balanced();
/// let session = generate_session(
///     "user1",
///     "memorization:surah-1",
///     &profile,
///     20, // session size
///     chrono::Utc::now().timestamp_millis(),
///     SessionMode::MixedLearning,
/// ).await?;
/// ```
pub mod bandit;
pub mod profiles;
pub mod scoring;
pub mod session_generator;
pub mod types;

pub use bandit::{
    blend_profile, BanditArmState, BanditOptimizer, BLEND_RATIO, DEFAULT_SAFE_PROFILE,
};
pub use profiles::{calculate_session_reward, profile_weights, ProfileName, SessionResult};
pub use scoring::{
    calculate_days_overdue, calculate_priority_score, calculate_readiness,
    count_unsatisfied_parents,
};
pub use session_generator::{generate_session, SessionMode};
pub use types::{
    CandidateNode, InMemNode, MemoryBasics, ParentEnergyMap, SessionMixConfig, UserProfile,
    MASTERY_THRESHOLD,
};
