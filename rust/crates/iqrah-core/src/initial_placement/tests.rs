//! Unit tests for initial placement module.

use rand::Rng;

use crate::initial_placement::config::InitialPlacementConfig;
use crate::initial_placement::service::make_rng_for;

#[cfg(test)]
mod determinism_tests {
    use super::*;

    #[test]
    fn test_rng_deterministic_same_inputs() {
        let rng1 = make_rng_for("user123", 1, 42);
        let rng2 = make_rng_for("user123", 1, 42);

        // Same seed should produce same sequence
        let mut rng1 = rng1;
        let mut rng2 = rng2;

        for _ in 0..10 {
            assert_eq!(rng1.gen::<u64>(), rng2.gen::<u64>());
        }
    }

    #[test]
    fn test_rng_different_with_different_user() {
        let mut rng1 = make_rng_for("user123", 1, 42);
        let mut rng2 = make_rng_for("user456", 1, 42);

        // Different user should produce different sequence
        let different = (0..10).any(|_| rng1.gen::<u64>() != rng2.gen::<u64>());
        assert!(different);
    }

    #[test]
    fn test_rng_different_with_different_chapter() {
        let mut rng1 = make_rng_for("user123", 1, 42);
        let mut rng2 = make_rng_for("user123", 2, 42);

        let different = (0..10).any(|_| rng1.gen::<u64>() != rng2.gen::<u64>());
        assert!(different);
    }

    #[test]
    fn test_rng_different_with_different_seed() {
        let mut rng1 = make_rng_for("user123", 1, 42);
        let mut rng2 = make_rng_for("user123", 1, 123);

        let different = (0..10).any(|_| rng1.gen::<u64>() != rng2.gen::<u64>());
        assert!(different);
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_stability_monotonic() {
        let config = InitialPlacementConfig::default();

        let s0 = config.verse_stability(0.0);
        let s25 = config.verse_stability(0.25);
        let s50 = config.verse_stability(0.50);
        let s75 = config.verse_stability(0.75);
        let s100 = config.verse_stability(1.0);

        assert!(s0 < s25);
        assert!(s25 < s50);
        assert!(s50 < s75);
        assert!(s75 < s100);
    }

    #[test]
    fn test_vocab_stability_monotonic() {
        let config = InitialPlacementConfig::default();

        let s0 = config.vocab_stability(0.0);
        let s50 = config.vocab_stability(0.50);
        let s100 = config.vocab_stability(1.0);

        assert!(s0 < s50);
        assert!(s50 < s100);
    }

    #[test]
    fn test_difficulty_decreases_with_fluency() {
        let config = InitialPlacementConfig::default();

        let d0 = config.verse_difficulty(0.0);
        let d50 = config.verse_difficulty(0.5);
        let d100 = config.verse_difficulty(1.0);

        assert!(d100 < d50);
        assert!(d50 < d0);
    }
}
