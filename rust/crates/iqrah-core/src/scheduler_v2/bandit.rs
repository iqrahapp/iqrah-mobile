/// Thompson Sampling bandit optimizer for hyper-personalized scheduling
///
/// Uses a multi-armed bandit with Beta priors to select user profile weights,
/// adapting to each user's learning style over time.
use crate::scheduler_v2::profiles::{profile_weights, ProfileName};
use crate::scheduler_v2::UserProfile;
use rand::Rng;
use rand_distr::{Beta, Distribution};

/// Default safe profile for blending (to maintain UX stability).
pub const DEFAULT_SAFE_PROFILE: ProfileName = ProfileName::Balanced;

/// Blending ratio: 80% chosen profile, 20% safe profile.
pub const BLEND_RATIO: f32 = 0.8;

// ============================================================================
// BANDIT STATE
// ============================================================================

/// State for a single bandit arm (profile).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BanditArmState {
    /// Profile name for this arm
    pub profile_name: ProfileName,

    /// Beta distribution alpha parameter (successes)
    pub successes: f32,

    /// Beta distribution beta parameter (failures)
    pub failures: f32,
}

impl BanditArmState {
    /// Creates a new arm with uninformed prior (1.0, 1.0).
    pub fn new(profile_name: ProfileName) -> Self {
        Self {
            profile_name,
            successes: 1.0,
            failures: 1.0,
        }
    }

    /// Creates an arm with specified parameters.
    pub fn with_params(profile_name: ProfileName, successes: f32, failures: f32) -> Self {
        Self {
            profile_name,
            successes,
            failures,
        }
    }

    /// Samples from this arm's Beta distribution.
    pub fn sample<R: Rng>(&self, rng: &mut R) -> f32 {
        let beta = Beta::new(self.successes as f64, self.failures as f64)
            .expect("Beta distribution parameters must be positive");
        beta.sample(rng) as f32
    }

    /// Updates this arm based on a reward.
    pub fn update(&mut self, reward: f32) {
        let reward = reward.clamp(0.0, 1.0);
        self.successes += reward;
        self.failures += 1.0 - reward;
    }
}

// ============================================================================
// BANDIT OPTIMIZER
// ============================================================================

/// Thompson Sampling bandit optimizer for profile selection.
///
/// Generic over RNG for testability.
pub struct BanditOptimizer<R: Rng> {
    rng: R,
}

impl<R: Rng> BanditOptimizer<R> {
    /// Creates a new bandit optimizer with the given RNG.
    pub fn new(rng: R) -> Self {
        Self { rng }
    }

    /// Chooses the best arm (profile) using Thompson Sampling.
    ///
    /// # Arguments
    /// * `arms` - Current state of all arms
    ///
    /// # Returns
    /// * The selected profile name
    pub fn choose_arm(&mut self, arms: &[BanditArmState]) -> ProfileName {
        if arms.is_empty() {
            return DEFAULT_SAFE_PROFILE;
        }

        // Sample from each arm and pick the highest
        let mut best_profile = arms[0].profile_name;
        let mut best_sample = arms[0].sample(&mut self.rng);

        for arm in &arms[1..] {
            let sample = arm.sample(&mut self.rng);
            if sample > best_sample {
                best_sample = sample;
                best_profile = arm.profile_name;
            }
        }

        best_profile
    }

    /// Initializes arms for a new user/goal_group with uninformed priors.
    pub fn initialize_arms() -> Vec<BanditArmState> {
        ProfileName::all()
            .iter()
            .map(|&name| BanditArmState::new(name))
            .collect()
    }
}

// ============================================================================
// PROFILE BLENDING
// ============================================================================

/// Blends a chosen profile with the safe default for UX stability.
///
/// Uses 80/20 ratio: 80% chosen profile, 20% safe profile.
///
/// # Arguments
/// * `chosen` - The profile selected by the bandit
///
/// # Returns
/// * Blended user profile
pub fn blend_profile(chosen: ProfileName) -> UserProfile {
    let chosen_weights = profile_weights(chosen);
    let safe_weights = profile_weights(DEFAULT_SAFE_PROFILE);
    chosen_weights.blend(&safe_weights, BLEND_RATIO)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler_v2::profiles::{calculate_session_reward, SessionResult};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_bandit_arm_state_new() {
        let arm = BanditArmState::new(ProfileName::Balanced);
        assert_eq!(arm.profile_name, ProfileName::Balanced);
        assert_eq!(arm.successes, 1.0);
        assert_eq!(arm.failures, 1.0);
    }

    #[test]
    fn test_bandit_arm_state_update() {
        let mut arm = BanditArmState::new(ProfileName::Balanced);

        // Update with reward 0.8
        arm.update(0.8);
        assert!((arm.successes - 1.8).abs() < 0.001);
        assert!((arm.failures - 1.2).abs() < 0.001);

        // Update with reward 0.3
        arm.update(0.3);
        assert!((arm.successes - 2.1).abs() < 0.001);
        assert!((arm.failures - 1.9).abs() < 0.001);
    }

    #[test]
    fn test_bandit_arm_sample() {
        let arm = BanditArmState::new(ProfileName::Balanced);
        let mut rng = StdRng::seed_from_u64(42);

        // Just test that sampling doesn't panic and returns valid value
        let sample = arm.sample(&mut rng);
        assert!((0.0..=1.0).contains(&sample));
    }

    #[test]
    fn test_bandit_optimizer_choose_arm() {
        let arms = vec![
            BanditArmState::with_params(ProfileName::Balanced, 10.0, 5.0),
            BanditArmState::with_params(ProfileName::FoundationHeavy, 5.0, 10.0),
            BanditArmState::with_params(ProfileName::UrgencyHeavy, 2.0, 2.0),
        ];

        let rng = StdRng::seed_from_u64(42);
        let mut bandit = BanditOptimizer::new(rng);

        let chosen = bandit.choose_arm(&arms);
        // Should choose one of the profiles (exact choice depends on RNG)
        assert!(ProfileName::all().contains(&chosen));
    }

    #[test]
    fn test_bandit_optimizer_empty_arms() {
        let rng = StdRng::seed_from_u64(42);
        let mut bandit = BanditOptimizer::new(rng);

        let chosen = bandit.choose_arm(&[]);
        assert_eq!(chosen, DEFAULT_SAFE_PROFILE);
    }

    #[test]
    fn test_initialize_arms() {
        let arms = BanditOptimizer::<StdRng>::initialize_arms();
        assert_eq!(arms.len(), ProfileName::all().len());

        for arm in arms {
            assert_eq!(arm.successes, 1.0);
            assert_eq!(arm.failures, 1.0);
        }
    }

    #[test]
    fn test_blend_profile() {
        let blended = blend_profile(ProfileName::FoundationHeavy);
        let chosen = profile_weights(ProfileName::FoundationHeavy);
        let safe = profile_weights(DEFAULT_SAFE_PROFILE);

        // Should be 80% chosen, 20% safe
        let expected_urgency = 0.8 * chosen.w_urgency + 0.2 * safe.w_urgency;
        assert!((blended.w_urgency - expected_urgency).abs() < 0.001);

        let expected_foundation = 0.8 * chosen.w_foundation + 0.2 * safe.w_foundation;
        assert!((blended.w_foundation - expected_foundation).abs() < 0.001);
    }

    #[test]
    fn test_integration_bandit_workflow() {
        // Simulate a full workflow
        let mut arms = BanditOptimizer::<StdRng>::initialize_arms();
        let rng = StdRng::seed_from_u64(42);
        let mut bandit = BanditOptimizer::new(rng);

        // Choose an arm
        let chosen = bandit.choose_arm(&arms);

        // Simulate session result
        let result = SessionResult {
            correct: 8,
            total: 10,
            completed: 10,
            presented: 10,
        };

        // Calculate reward
        let reward = calculate_session_reward(&result);

        // Update the chosen arm
        let arm_idx = arms.iter().position(|a| a.profile_name == chosen).unwrap();
        arms[arm_idx].update(reward);

        // Verify update happened
        assert!(arms[arm_idx].successes > 1.0);
        assert!(arms[arm_idx].failures >= 1.0);
    }
}
