// exercises/find_mistake.rs
// Exercise 11: Find the Mistake - Verse with one subtle word substitution

use super::types::Exercise;
use crate::domain::node_id::PREFIX_VERSE;
use crate::{ContentRepository, Word};
use anyhow::Result;
use rand::seq::SliceRandom;
use rand::Rng;

// ============================================================================
// Exercise 11: Find the Mistake
// ============================================================================

/// Exercise for identifying incorrect words in verses
/// Shows a verse with one word substituted from another verse
/// User must identify which word is incorrect
///
/// IMPORTANT: This exercise modifies Quranic text for educational purposes only.
/// The modification is temporary and only for testing memorization.
#[derive(Debug)]
pub struct FindMistakeExercise {
    node_id: i64,
    verse_key: String,
    correct_verse_text: String,
    modified_verse_text: String,
    mistake_position: i32, // 1-indexed position of the incorrect word
    correct_word: String,
    incorrect_word: String,
}

impl FindMistakeExercise {
    /// Create a new Find the Mistake exercise
    ///
    /// Queries the database for:
    /// - The target verse and its words
    /// - Words from other verses in the same chapter (for substitution)
    /// - Randomly selects a word to replace (avoiding first/last words)
    pub async fn new(verse_node_id: i64, content_repo: &dyn ContentRepository) -> Result<Self> {
        // Get the node to access its ukey
        let node = content_repo
            .get_node(verse_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Node not found: {}", verse_node_id))?;

        // Parse verse_key from node_id (format: PREFIX_VERSE + "chapter:verse")
        let verse_key = node
            .ukey
            .strip_prefix(PREFIX_VERSE)
            .ok_or_else(|| anyhow::anyhow!("Invalid verse node ID: {}", node.ukey))?
            .to_string();

        // Parse chapter number
        let parts: Vec<&str> = verse_key.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid verse key format: {}", verse_key));
        }
        let chapter_num: i32 = parts[0].parse()?;

        // Get the correct verse text
        let correct_verse_text = content_repo
            .get_quran_text(verse_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Verse text not found: {}", verse_node_id))?;

        // Get all words for the verse
        let words = content_repo.get_words_for_verse(&verse_key).await?;

        if words.len() < 3 {
            return Err(anyhow::anyhow!(
                "Verse too short for Find the Mistake exercise (need at least 3 words)"
            ));
        }

        // Select a random position to substitute (avoid first and last words)
        let mut rng = rand::thread_rng();
        let mistake_position: i32 = if words.len() > 3 {
            // For longer verses, choose from middle words (not first or last)
            (rng.gen_range(1..words.len() - 1) + 1) as i32
        } else {
            // For 3-word verses, choose the middle word
            2
        };

        // Get the word at that position
        let word_at_position = words
            .iter()
            .find(|w| w.position == mistake_position)
            .ok_or_else(|| anyhow::anyhow!("Word not found at position {}", mistake_position))?;

        let correct_word = word_at_position.text_uthmani.clone();

        // Get a replacement word from another verse in the same chapter
        let incorrect_word =
            Self::get_replacement_word(chapter_num, &verse_key, &correct_word, content_repo)
                .await?;

        // Build modified verse with the mistake
        let modified_verse_text =
            Self::build_modified_verse(&words, mistake_position, &incorrect_word)?;

        Ok(Self {
            node_id: verse_node_id,
            verse_key,
            correct_verse_text,
            modified_verse_text,
            mistake_position,
            correct_word,
            incorrect_word,
        })
    }

    /// Get a replacement word from another verse in the same chapter
    async fn get_replacement_word(
        chapter_num: i32,
        current_verse_key: &str,
        original_word: &str,
        content_repo: &dyn ContentRepository,
    ) -> Result<String> {
        // Get all verses in the chapter
        let verses = content_repo.get_verses_for_chapter(chapter_num).await?;

        // Collect words from other verses (excluding current verse)
        let mut candidate_words: Vec<String> = Vec::new();
        for verse in &verses {
            if verse.key != current_verse_key {
                let words = content_repo.get_words_for_verse(&verse.key).await?;
                for word in words {
                    // Only include words that are different from the original
                    if word.text_uthmani != original_word {
                        candidate_words.push(word.text_uthmani);
                    }
                }
            }
        }

        if candidate_words.is_empty() {
            return Err(anyhow::anyhow!(
                "No suitable replacement words found in chapter {}",
                chapter_num
            ));
        }

        // Select a random word
        let mut rng = rand::thread_rng();
        Ok(candidate_words
            .choose(&mut rng)
            .ok_or_else(|| anyhow::anyhow!("Failed to select random word"))?
            .clone())
    }

    /// Build the modified verse with the incorrect word at the specified position
    fn build_modified_verse(
        words: &[Word],
        mistake_position: i32,
        incorrect_word: &str,
    ) -> Result<String> {
        let mut modified_words = Vec::new();

        for word in words {
            if word.position == mistake_position {
                modified_words.push(incorrect_word.to_string());
            } else {
                modified_words.push(word.text_uthmani.clone());
            }
        }

        Ok(modified_words.join(" "))
    }

    /// Get the position of the mistake (1-indexed)
    pub fn get_mistake_position(&self) -> i32 {
        self.mistake_position
    }

    /// Get the modified verse with the mistake
    pub fn get_modified_verse(&self) -> &str {
        &self.modified_verse_text
    }

    /// Get the correct word
    pub fn get_correct_word(&self) -> &str {
        &self.correct_word
    }

    /// Get the incorrect word
    pub fn get_incorrect_word(&self) -> &str {
        &self.incorrect_word
    }

    /// Check if the user identified the correct position
    pub fn check_position(&self, position: i32) -> bool {
        position == self.mistake_position
    }

    /// Get the correct verse text (for showing correct answer after mistake)
    pub fn get_correct_verse(&self) -> &str {
        &self.correct_verse_text
    }
}

impl Exercise for FindMistakeExercise {
    fn generate_question(&self) -> String {
        format!(
            "Find the mistake in this verse:\n\n{}\n\nWhich word is incorrect? (Indicate position 1-{})",
            self.modified_verse_text,
            self.modified_verse_text.split_whitespace().count()
        )
    }

    fn check_answer(&self, answer: &str) -> bool {
        // Try to parse the answer as a position number
        if let Ok(position) = answer.trim().parse::<i32>() {
            self.check_position(position)
        } else {
            false
        }
    }

    fn get_hint(&self) -> Option<String> {
        // Show the verse reference as a hint
        Some(format!(
            "This verse is from {}. Try to recall what it should say.",
            self.verse_key
        ))
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "find_mistake"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Find mistake tests require database setup
        // Tests are integrated via mockall
    }
}
