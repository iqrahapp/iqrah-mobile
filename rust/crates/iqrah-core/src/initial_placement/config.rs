//! Configuration for initial placement mapping.

use serde::{Deserialize, Serialize};

/// Configuration for initial placement mapping formulas.
///
/// All coefficients are tunable to adjust how self-reported knowledge
/// translates to FSRS memory states and energies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialPlacementConfig {
    // ==========================================================================
    // Verse Memorization Mapping
    // ==========================================================================
    /// Base stability (in days) for a fully-known verse
    ///
    /// Formula: stability = base * (pct ^ power) + min
    /// Default: 30.0 days
    pub verse_base_stability_days: f64,

    /// Power exponent for stability mapping
    ///
    /// Higher values = more aggressive curve (low pct → low stability)
    /// Default: 2.0
    pub verse_stability_power: f64,

    /// Minimum stability for any initialized verse
    ///
    /// Default: 1.0 days
    pub verse_min_stability_days: f64,

    /// Base review count for fully-known verse
    ///
    /// Scales linearly with memorization_pct
    /// Default: 5
    pub verse_base_review_count: u32,

    /// Threshold for treating verse as "fully known"
    ///
    /// pct >= threshold means 100% of verses in that portion get initialized
    /// Default: 0.9
    pub verse_known_threshold: f64,

    /// Base difficulty for initialized verses
    ///
    /// Lower = easier (range 1.0-10.0 in FSRS)
    /// Default: 3.0
    pub verse_base_difficulty: f64,

    // ==========================================================================
    // Vocabulary/Meaning Mapping
    // ==========================================================================
    /// Base stability (in days) for known vocabulary
    ///
    /// Vocabulary is typically less stable than memorization
    /// Default: 14.0 days
    pub vocab_base_stability_days: f64,

    /// Power exponent for vocab stability mapping
    ///
    /// Default: 1.5
    pub vocab_stability_power: f64,

    /// Base review count for known vocabulary
    ///
    /// Default: 3
    pub vocab_base_review_count: u32,

    /// Base difficulty for vocabulary nodes
    ///
    /// Default: 4.0
    pub vocab_base_difficulty: f64,

    // ==========================================================================
    // Energy Initialization
    // ==========================================================================
    /// Energy value for known verses (0.0-1.0)
    ///
    /// Default: 0.8
    pub verse_known_energy: f64,

    /// Energy value for partially-known verses (0.0-1.0)
    ///
    /// Default: 0.4
    pub verse_partial_energy: f64,

    /// Energy value for known vocabulary (0.0-1.0)
    ///
    /// Default: 0.7
    pub vocab_known_energy: f64,

    // ==========================================================================
    // Global Modifiers
    // ==========================================================================
    /// Difficulty reduction factor for reading fluency
    ///
    /// Applied as: difficulty *= (1.0 - factor * fluency)
    /// Default: 0.2
    pub fluency_difficulty_reduction: f64,

    /// Partial knowledge threshold
    ///
    /// Verses with memorization_pct > threshold but < known_threshold
    /// get partial initialization
    /// Default: 0.3
    pub partial_threshold: f64,
}

impl Default for InitialPlacementConfig {
    fn default() -> Self {
        Self {
            // Verse memorization
            verse_base_stability_days: 30.0,
            verse_stability_power: 2.0,
            verse_min_stability_days: 1.0,
            verse_base_review_count: 5,
            verse_known_threshold: 0.9,
            verse_base_difficulty: 3.0,

            // Vocabulary
            vocab_base_stability_days: 14.0,
            vocab_stability_power: 1.5,
            vocab_base_review_count: 3,
            vocab_base_difficulty: 4.0,

            // Energy
            verse_known_energy: 0.8,
            verse_partial_energy: 0.4,
            vocab_known_energy: 0.7,

            // Global
            fluency_difficulty_reduction: 0.2,
            partial_threshold: 0.3,
        }
    }
}

impl InitialPlacementConfig {
    /// Calculate verse stability from memorization percentage.
    ///
    /// Formula: base * (pct ^ power) + min
    pub fn verse_stability(&self, memorization_pct: f64) -> f64 {
        let pct = memorization_pct.clamp(0.0, 1.0);
        self.verse_base_stability_days * pct.powf(self.verse_stability_power)
            + self.verse_min_stability_days
    }

    /// Calculate verse review count from memorization percentage.
    pub fn verse_review_count(&self, memorization_pct: f64) -> u32 {
        let pct = memorization_pct.clamp(0.0, 1.0);
        ((self.verse_base_review_count as f64) * pct).round() as u32
    }

    /// Calculate verse difficulty adjusted for reading fluency.
    pub fn verse_difficulty(&self, reading_fluency: f64) -> f64 {
        let fluency = reading_fluency.clamp(0.0, 1.0);
        self.verse_base_difficulty * (1.0 - self.fluency_difficulty_reduction * fluency)
    }

    /// Calculate vocab stability from understanding percentage.
    pub fn vocab_stability(&self, understanding_pct: f64) -> f64 {
        let pct = understanding_pct.clamp(0.0, 1.0);
        self.vocab_base_stability_days * pct.powf(self.vocab_stability_power)
            + self.verse_min_stability_days
    }

    /// Calculate vocab review count from understanding percentage.
    pub fn vocab_review_count(&self, understanding_pct: f64) -> u32 {
        let pct = understanding_pct.clamp(0.0, 1.0);
        ((self.vocab_base_review_count as f64) * pct).round() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verse_stability_mapping() {
        let config = InitialPlacementConfig::default();

        // Full memorization
        let s100 = config.verse_stability(1.0);
        assert!((s100 - 31.0).abs() < 0.01); // 30 * 1.0^2 + 1 = 31

        // Half memorization
        let s50 = config.verse_stability(0.5);
        assert!((s50 - 8.5).abs() < 0.01); // 30 * 0.25 + 1 = 8.5

        // No memorization
        let s0 = config.verse_stability(0.0);
        assert!((s0 - 1.0).abs() < 0.01); // 30 * 0 + 1 = 1
    }

    #[test]
    fn test_verse_difficulty_with_fluency() {
        let config = InitialPlacementConfig::default();

        // No fluency
        let d0 = config.verse_difficulty(0.0);
        assert!((d0 - 3.0).abs() < 0.01);

        // Full fluency
        let d100 = config.verse_difficulty(1.0);
        assert!((d100 - 2.4).abs() < 0.01); // 3.0 * (1 - 0.2 * 1.0) = 2.4
    }
}
