use crate::domain::models::{Hint, WordVisibility};

/// Maps a word's energy and its neighbors' energy to a visibility state.
///
/// # Parameters
/// - `energy`: The word's current mastery level (0.0 to 1.0)
/// - `text`: The word's text content (for extracting first/last characters)
/// - `prev_word_energy`: The energy of the preceding word (if it exists)
/// - `next_word_energy`: The energy of the succeeding word (if it exists)
///
/// # Logic
/// - If energy < 0.15: Word is fully visible (still learning)
/// - If energy >= 0.85: Word is hidden (nearly mastered)
/// - If 0.15 <= energy < 0.85: Word is obscured with a hint
///   - Hint type depends on neighbor energy (are they strong anchors?)
///   - Coverage increases smoothly with energy (using power curve)
pub fn map_energy_to_visibility(
    energy: f64,
    text: &str,
    prev_word_energy: Option<f64>,
    next_word_energy: Option<f64>,
) -> WordVisibility {
    // Threshold constants
    const VISIBLE_THRESHOLD: f64 = 0.15;
    const HIDDEN_THRESHOLD: f64 = 0.85;
    const ANCHOR_THRESHOLD: f64 = 0.3;

    // Fully visible for low energy (still learning)
    if energy < VISIBLE_THRESHOLD {
        return WordVisibility::Visible;
    }

    // Fully hidden for high energy (nearly mastered)
    if energy >= HIDDEN_THRESHOLD {
        return WordVisibility::Hidden;
    }

    // Obscured for intermediate energy levels
    // Calculate coverage using a power curve for smooth progression
    let normalized_energy = (energy - VISIBLE_THRESHOLD) / (HIDDEN_THRESHOLD - VISIBLE_THRESHOLD);
    let coverage = normalized_energy.powf(1.5).clamp(0.0, 1.0);

    // Extract first and last characters
    let first_char = text.chars().next().unwrap_or('_');
    let last_char = text.chars().last().unwrap_or('_');

    // Determine if neighbors are strong anchors
    let prev_is_anchor = prev_word_energy.unwrap_or(1.0) >= ANCHOR_THRESHOLD;
    let next_is_anchor = next_word_energy.unwrap_or(1.0) >= ANCHOR_THRESHOLD;

    // Context-aware hint selection
    let hint = match (prev_is_anchor, next_is_anchor) {
        (true, false) => Hint::First { char: first_char },  // Previous word is anchor
        (false, true) => Hint::Last { char: last_char },     // Next word is anchor
        (true, true) => Hint::Both { first: first_char, last: last_char }, // Both are anchors
        (false, false) => Hint::First { char: first_char },  // Default fallback
    };

    WordVisibility::Obscured { hint, coverage }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fully_visible_for_low_energy() {
        let visibility = map_energy_to_visibility(0.1, "test", None, None);
        assert_eq!(visibility, WordVisibility::Visible);

        let visibility = map_energy_to_visibility(0.0, "test", Some(0.5), Some(0.5));
        assert_eq!(visibility, WordVisibility::Visible);
    }

    #[test]
    fn test_fully_hidden_for_high_energy() {
        let visibility = map_energy_to_visibility(0.85, "test", None, None);
        assert_eq!(visibility, WordVisibility::Hidden);

        let visibility = map_energy_to_visibility(0.95, "test", Some(0.5), Some(0.5));
        assert_eq!(visibility, WordVisibility::Hidden);

        let visibility = map_energy_to_visibility(1.0, "test", Some(0.0), Some(0.0));
        assert_eq!(visibility, WordVisibility::Hidden);
    }

    #[test]
    fn test_hint_first_when_prev_is_anchor() {
        // Previous word is strong (0.5), next is weak (0.1)
        let visibility = map_energy_to_visibility(0.5, "test", Some(0.5), Some(0.1));

        match visibility {
            WordVisibility::Obscured { hint, coverage: _ } => {
                assert_eq!(hint, Hint::First { char: 't' });
            }
            _ => panic!("Expected Obscured variant"),
        }
    }

    #[test]
    fn test_hint_last_when_next_is_anchor() {
        // Previous word is weak (0.1), next is strong (0.5)
        let visibility = map_energy_to_visibility(0.5, "test", Some(0.1), Some(0.5));

        match visibility {
            WordVisibility::Obscured { hint, coverage: _ } => {
                assert_eq!(hint, Hint::Last { char: 't' });
            }
            _ => panic!("Expected Obscured variant"),
        }
    }

    #[test]
    fn test_hint_both_when_both_are_anchors() {
        // Both neighbors are strong
        let visibility = map_energy_to_visibility(0.5, "test", Some(0.4), Some(0.6));

        match visibility {
            WordVisibility::Obscured { hint, coverage: _ } => {
                assert_eq!(hint, Hint::Both { first: 't', last: 't' });
            }
            _ => panic!("Expected Obscured variant"),
        }
    }

    #[test]
    fn test_hint_first_when_neither_are_anchors() {
        // Both neighbors are weak (default fallback)
        let visibility = map_energy_to_visibility(0.5, "test", Some(0.1), Some(0.2));

        match visibility {
            WordVisibility::Obscured { hint, coverage: _ } => {
                assert_eq!(hint, Hint::First { char: 't' });
            }
            _ => panic!("Expected Obscured variant"),
        }
    }

    #[test]
    fn test_hint_first_when_no_neighbors() {
        // No neighbors (None values default to 1.0, which is > 0.3, so both are anchors)
        let visibility = map_energy_to_visibility(0.5, "test", None, None);

        match visibility {
            WordVisibility::Obscured { hint, coverage: _ } => {
                // With None values defaulting to 1.0, both are anchors, so we get Both
                assert_eq!(hint, Hint::Both { first: 't', last: 't' });
            }
            _ => panic!("Expected Obscured variant"),
        }
    }

    #[test]
    fn test_coverage_increases_with_energy() {
        // Test coverage at different energy levels
        let vis_low = map_energy_to_visibility(0.2, "test", Some(0.5), Some(0.5));
        let vis_mid = map_energy_to_visibility(0.5, "test", Some(0.5), Some(0.5));
        let vis_high = map_energy_to_visibility(0.8, "test", Some(0.5), Some(0.5));

        let coverage_low = match vis_low {
            WordVisibility::Obscured { hint: _, coverage } => coverage,
            _ => panic!("Expected Obscured"),
        };

        let coverage_mid = match vis_mid {
            WordVisibility::Obscured { hint: _, coverage } => coverage,
            _ => panic!("Expected Obscured"),
        };

        let coverage_high = match vis_high {
            WordVisibility::Obscured { hint: _, coverage } => coverage,
            _ => panic!("Expected Obscured"),
        };

        assert!(coverage_low < coverage_mid);
        assert!(coverage_mid < coverage_high);
        assert!(coverage_low >= 0.0 && coverage_low <= 1.0);
        assert!(coverage_high >= 0.0 && coverage_high <= 1.0);
    }

    #[test]
    fn test_coverage_power_curve() {
        // Test that coverage follows the power curve formula
        let energy = 0.5;
        let visibility = map_energy_to_visibility(energy, "test", Some(0.5), Some(0.5));

        let expected_coverage = ((energy - 0.15) / (0.85 - 0.15)).powf(1.5);

        match visibility {
            WordVisibility::Obscured { hint: _, coverage } => {
                assert!((coverage - expected_coverage).abs() < 0.001);
            }
            _ => panic!("Expected Obscured"),
        }
    }

    #[test]
    fn test_edge_cases_at_thresholds() {
        // Test behavior exactly at threshold boundaries
        let vis_at_visible_threshold = map_energy_to_visibility(0.15, "test", Some(0.5), Some(0.5));
        match vis_at_visible_threshold {
            WordVisibility::Obscured { hint: _, coverage } => {
                assert_eq!(coverage, 0.0); // Should be minimum coverage
            }
            _ => panic!("Expected Obscured at visible threshold"),
        }

        let vis_just_below_hidden = map_energy_to_visibility(0.849, "test", Some(0.5), Some(0.5));
        match vis_just_below_hidden {
            WordVisibility::Obscured { hint: _, coverage: _ } => {
                // Should still be obscured
            }
            _ => panic!("Expected Obscured just below hidden threshold"),
        }
    }

    #[test]
    fn test_arabic_text() {
        // Test with Arabic characters (without diacritics for char literal compatibility)
        let visibility = map_energy_to_visibility(0.5, "بسم", Some(0.5), Some(0.5));

        match visibility {
            WordVisibility::Obscured { hint, coverage: _ } => {
                assert_eq!(hint, Hint::Both { first: 'ب', last: 'م' });
            }
            _ => panic!("Expected Obscured"),
        }
    }

    #[test]
    fn test_single_character_word() {
        let visibility = map_energy_to_visibility(0.5, "a", Some(0.5), Some(0.5));

        match visibility {
            WordVisibility::Obscured { hint, coverage: _ } => {
                assert_eq!(hint, Hint::Both { first: 'a', last: 'a' });
            }
            _ => panic!("Expected Obscured"),
        }
    }
}
