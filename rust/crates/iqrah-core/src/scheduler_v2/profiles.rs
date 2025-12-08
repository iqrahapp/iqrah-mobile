/// User profile presets for scheduler v2.1
///
/// Defines different weighting strategies for scheduling, used by the bandit optimizer
/// to hyper-personalize the learning experience.
use crate::scheduler_v2::UserProfile;

// ============================================================================
// PROFILE NAME ENUM
// ============================================================================

/// Named profile presets with different weight preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileName {
    /// Balanced: Equal weights for all factors (1.0, 1.0, 1.0, 1.0)
    Balanced,

    /// Foundation-heavy: Prioritizes foundational concepts (PageRank on forward graph)
    FoundationHeavy,

    /// Influence-heavy: Prioritizes influential concepts (PageRank on reversed graph)
    InfluenceHeavy,

    /// Urgency-heavy: Prioritizes overdue items
    UrgencyHeavy,

    /// Readiness-focused: Prioritizes nodes where prerequisites are well-mastered
    ReadinessFocused,
}

impl ProfileName {
    /// Returns all profile names for iteration.
    pub fn all() -> &'static [ProfileName] {
        &[
            ProfileName::Balanced,
            ProfileName::FoundationHeavy,
            ProfileName::InfluenceHeavy,
            ProfileName::UrgencyHeavy,
            ProfileName::ReadinessFocused,
        ]
    }

    /// Converts profile name to string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProfileName::Balanced => "Balanced",
            ProfileName::FoundationHeavy => "FoundationHeavy",
            ProfileName::InfluenceHeavy => "InfluenceHeavy",
            ProfileName::UrgencyHeavy => "UrgencyHeavy",
            ProfileName::ReadinessFocused => "ReadinessFocused",
        }
    }

    /// Parses profile name from string.
    ///
    /// Note: Named `parse_str` instead of `from_str` to avoid confusion with `FromStr` trait.
    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "Balanced" => Some(ProfileName::Balanced),
            "FoundationHeavy" => Some(ProfileName::FoundationHeavy),
            "InfluenceHeavy" => Some(ProfileName::InfluenceHeavy),
            "UrgencyHeavy" => Some(ProfileName::UrgencyHeavy),
            "ReadinessFocused" => Some(ProfileName::ReadinessFocused),
            _ => None,
        }
    }
}

// ============================================================================
// PROFILE WEIGHTS MAPPING
// ============================================================================

/// Maps a profile name to its weight configuration.
pub fn profile_weights(name: ProfileName) -> UserProfile {
    match name {
        ProfileName::Balanced => UserProfile {
            w_urgency: 1.0,
            w_readiness: 1.0,
            w_foundation: 1.0,
            w_influence: 1.0,
            w_fairness: 0.3,
        },
        ProfileName::FoundationHeavy => UserProfile {
            w_urgency: 0.8,
            w_readiness: 1.0,
            w_foundation: 1.5,
            w_influence: 0.8,
            w_fairness: 0.2,
        },
        ProfileName::InfluenceHeavy => UserProfile {
            w_urgency: 0.8,
            w_readiness: 1.0,
            w_foundation: 0.8,
            w_influence: 1.5,
            w_fairness: 0.2,
        },
        ProfileName::UrgencyHeavy => UserProfile {
            w_urgency: 1.5,
            w_readiness: 0.8,
            w_foundation: 1.0,
            w_influence: 1.0,
            w_fairness: 0.15,
        },
        ProfileName::ReadinessFocused => UserProfile {
            w_urgency: 0.8,
            w_readiness: 1.5,
            w_foundation: 1.0,
            w_influence: 1.0,
            w_fairness: 0.25,
        },
    }
}

// ============================================================================
// SESSION RESULT & REWARD CALCULATION
// ============================================================================

/// Summary of a completed session for bandit feedback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionResult {
    /// Number of items answered correctly
    pub correct: u32,

    /// Total number of items in session
    pub total: u32,

    /// Number of items the user actually completed (vs skipped/abandoned)
    pub completed: u32,

    /// Number of items presented to the user
    pub presented: u32,
}

/// Calculates reward for a session based on accuracy and completion rate.
///
/// Formula: `reward = 0.6 * accuracy + 0.4 * completion_rate`
///
/// # Arguments
/// * `result` - The session result
///
/// # Returns
/// * Reward value in [0.0, 1.0]
pub fn calculate_session_reward(result: &SessionResult) -> f32 {
    let accuracy = if result.total > 0 {
        result.correct as f32 / result.total as f32
    } else {
        0.0
    };

    let completion_rate = if result.presented > 0 {
        result.completed as f32 / result.presented as f32
    } else {
        0.0
    };

    ((0.6 * accuracy) + (0.4 * completion_rate)).clamp(0.0, 1.0)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_name_all() {
        let profiles = ProfileName::all();
        assert_eq!(profiles.len(), 5);
        assert!(profiles.contains(&ProfileName::Balanced));
        assert!(profiles.contains(&ProfileName::FoundationHeavy));
    }

    #[test]
    fn test_profile_name_str_roundtrip() {
        for profile in ProfileName::all() {
            let s = profile.as_str();
            let parsed = ProfileName::parse_str(s);
            assert_eq!(parsed, Some(*profile));
        }
    }

    #[test]
    fn test_profile_weights_balanced() {
        let weights = profile_weights(ProfileName::Balanced);
        assert_eq!(weights.w_urgency, 1.0);
        assert_eq!(weights.w_readiness, 1.0);
        assert_eq!(weights.w_foundation, 1.0);
        assert_eq!(weights.w_influence, 1.0);
    }

    #[test]
    fn test_profile_weights_foundation_heavy() {
        let weights = profile_weights(ProfileName::FoundationHeavy);
        assert_eq!(weights.w_foundation, 1.5);
        assert!(weights.w_foundation > weights.w_influence);
    }

    #[test]
    fn test_calculate_session_reward_perfect() {
        let result = SessionResult {
            correct: 10,
            total: 10,
            completed: 10,
            presented: 10,
        };
        let reward = calculate_session_reward(&result);
        assert_eq!(reward, 1.0);
    }

    #[test]
    fn test_calculate_session_reward_half_accuracy() {
        let result = SessionResult {
            correct: 5,
            total: 10,
            completed: 10,
            presented: 10,
        };
        let reward = calculate_session_reward(&result);
        // accuracy = 0.5, completion = 1.0
        // reward = 0.6 * 0.5 + 0.4 * 1.0 = 0.3 + 0.4 = 0.7
        assert!((reward - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_calculate_session_reward_incomplete() {
        let result = SessionResult {
            correct: 5,
            total: 10,
            completed: 7,
            presented: 10,
        };
        let reward = calculate_session_reward(&result);
        // accuracy = 0.5, completion = 0.7
        // reward = 0.6 * 0.5 + 0.4 * 0.7 = 0.3 + 0.28 = 0.58
        assert!((reward - 0.58).abs() < 0.001);
    }
}
