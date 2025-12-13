//! Helper functions for exercise evaluation.
//!
//! Provides wrappers around ContentRepository methods and utility functions
//! for sampling nodes, computing sequential paths, and other exercise needs.

use anyhow::{anyhow, Result};
use iqrah_core::domain::MemoryState;
use iqrah_core::ports::ContentRepository;
use rand::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use super::SamplingStrategy;

// ============================================================================
// Sequential Path Helpers
// ============================================================================

/// Get word node IDs for an ayah in sequential order.
///
/// Uses ContentRepository to fetch words for a verse key.
pub async fn get_word_sequence_for_ayah(
    content_repo: &Arc<dyn ContentRepository>,
    verse_key: &str,
) -> Result<Vec<i64>> {
    use iqrah_core::domain::node_id as nid;

    let words = content_repo.get_words_for_verse(verse_key).await?;

    Ok(words.into_iter().map(|w| nid::encode_word(w.id)).collect())
}

/// Get sequential word path from start to end verse.
///
/// Returns all word node IDs in reading order from start_verse to end_verse.
/// Both verses are inclusive.
///
/// # Example
/// ```ignore
/// let path = get_sequential_path(content_repo, 1, 1, 7).await?;
/// // Returns words from 1:1 through 1:7
/// ```
pub async fn get_sequential_path(
    content_repo: &Arc<dyn ContentRepository>,
    chapter: i32,
    start_verse: i32,
    end_verse: i32,
) -> Result<Vec<i64>> {
    use iqrah_core::domain::node_id as nid;

    let mut all_words = Vec::new();

    // Get all verses for the chapter
    let verses = content_repo.get_verses_for_chapter(chapter).await?;

    // Filter to range and collect words
    for verse in verses {
        if verse.verse_number >= start_verse && verse.verse_number <= end_verse {
            let words = content_repo.get_words_for_verse(&verse.key).await?;
            for word in words {
                all_words.push(nid::encode_word(word.id));
            }
        }
    }

    Ok(all_words)
}

/// Get all word node IDs for a page (in reading order).
///
/// Queries verses by page number, then fetches words for each verse.
pub async fn get_words_on_page(
    content_repo: &Arc<dyn ContentRepository>,
    page_number: u16,
) -> Result<Vec<i64>> {
    use iqrah_core::domain::node_id as nid;

    let mut all_words = Vec::new();

    // Get all chapters (could optimize with page-based query)
    let chapters = content_repo.get_chapters().await?;

    for chapter in chapters {
        let verses = content_repo.get_verses_for_chapter(chapter.number).await?;

        for verse in verses {
            if verse.page == page_number as i32 {
                let words = content_repo.get_words_for_verse(&verse.key).await?;
                for word in words {
                    all_words.push(nid::encode_word(word.id));
                }
            }
        }
    }

    Ok(all_words)
}

// ============================================================================
// Ayah Sampling Helpers
// ============================================================================

/// Filter goal items to only verse (ayah) node IDs.
///
/// Uses node ID encoding to identify verse nodes.
pub fn get_ayahs_from_goal(goal_items: &[i64]) -> Vec<i64> {
    use iqrah_core::domain::node_id as nid;

    goal_items
        .iter()
        .filter(|&&id| nid::decode_verse(id).is_some())
        .copied()
        .collect()
}

/// Sample a random ayah from goal items.
pub fn sample_random_ayah(goal_items: &[i64], rng: &mut impl Rng) -> Result<i64> {
    let ayahs = get_ayahs_from_goal(goal_items);

    if ayahs.is_empty() {
        return Err(anyhow!("No ayahs found in goal items"));
    }

    let idx = rng.gen_range(0..ayahs.len());
    Ok(ayahs[idx])
}

/// Sample multiple ayahs according to strategy.
pub fn sample_ayahs(
    goal_items: &[i64],
    sample_size: usize,
    strategy: SamplingStrategy,
    memory_states: &HashMap<i64, MemoryState>,
    rng: &mut impl Rng,
) -> Result<Vec<i64>> {
    let mut ayahs = get_ayahs_from_goal(goal_items);

    if ayahs.is_empty() {
        return Err(anyhow!("No ayahs found in goal items"));
    }

    match strategy {
        SamplingStrategy::Random => {
            ayahs.shuffle(rng);
            Ok(ayahs.into_iter().take(sample_size).collect())
        }

        SamplingStrategy::FullRange => {
            // Take first N ayahs (in goal order)
            Ok(ayahs.into_iter().take(sample_size).collect())
        }

        SamplingStrategy::Urgency => {
            // Sort by energy ascending (lowest energy = most urgent)
            ayahs.sort_by(|a, b| {
                let e_a = memory_states.get(a).map(|s| s.energy).unwrap_or(0.0);
                let e_b = memory_states.get(b).map(|s| s.energy).unwrap_or(0.0);
                e_a.partial_cmp(&e_b).unwrap_or(std::cmp::Ordering::Equal)
            });
            Ok(ayahs.into_iter().take(sample_size).collect())
        }

        SamplingStrategy::Coverage => {
            // Sort by review count ascending (least reviewed first)
            ayahs.sort_by(|a, b| {
                let r_a = memory_states.get(a).map(|s| s.review_count).unwrap_or(0);
                let r_b = memory_states.get(b).map(|s| s.review_count).unwrap_or(0);
                r_a.cmp(&r_b)
            });
            Ok(ayahs.into_iter().take(sample_size).collect())
        }

        SamplingStrategy::Frequency => {
            // For frequency, just use full range (frequency filtering is vocabulary-specific)
            Ok(ayahs.into_iter().take(sample_size).collect())
        }
    }
}

// ============================================================================
// Node ID Utilities
// ============================================================================

/// Parse verse key from a verse node ID.
///
/// Returns (chapter, verse) tuple.
pub fn parse_verse_key(node_id: i64) -> Option<(i32, i32)> {
    use iqrah_core::domain::node_id as nid;
    nid::decode_verse(node_id).map(|(ch, v)| (ch as i32, v as i32))
}

/// Get verse key string from node ID.
pub fn verse_key_string(node_id: i64) -> Option<String> {
    parse_verse_key(node_id).map(|(ch, v)| format!("{}:{}", ch, v))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use iqrah_core::domain::node_id as nid;

    #[test]
    fn test_get_ayahs_from_goal() {
        // Create mix of verse and word node IDs
        let verse_1_1 = nid::encode_verse(1, 1);
        let verse_1_2 = nid::encode_verse(1, 2);
        let word_1 = nid::encode_word(1);
        let word_2 = nid::encode_word(2);

        let goal_items = vec![verse_1_1, word_1, verse_1_2, word_2];
        let ayahs = get_ayahs_from_goal(&goal_items);

        assert_eq!(ayahs.len(), 2);
        assert!(ayahs.contains(&verse_1_1));
        assert!(ayahs.contains(&verse_1_2));
    }

    #[test]
    fn test_sample_random_ayah() {
        let verse_1_1 = nid::encode_verse(1, 1);
        let verse_1_2 = nid::encode_verse(1, 2);
        let goal_items = vec![verse_1_1, verse_1_2];

        let mut rng = rand::thread_rng();
        let result = sample_random_ayah(&goal_items, &mut rng);

        assert!(result.is_ok());
        let ayah = result.unwrap();
        assert!(ayah == verse_1_1 || ayah == verse_1_2);
    }

    #[test]
    fn test_sample_random_ayah_empty() {
        let goal_items: Vec<i64> = vec![];
        let mut rng = rand::thread_rng();
        let result = sample_random_ayah(&goal_items, &mut rng);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_verse_key() {
        let verse_id = nid::encode_verse(1, 5);
        let parsed = parse_verse_key(verse_id);

        assert_eq!(parsed, Some((1, 5)));
    }

    #[test]
    fn test_verse_key_string() {
        let verse_id = nid::encode_verse(2, 255);
        let key = verse_key_string(verse_id);

        assert_eq!(key, Some("2:255".to_string()));
    }
}
