// exercises/ayah_chain.rs
// Exercise 10: Ayah Chain - Continuous verse typing until mistake or completion

use super::memorization::MemorizationExercise;
use super::types::Exercise;
use crate::{ContentRepository, Verse};
use anyhow::Result;

// ============================================================================
// Exercise 10: Ayah Chain
// ============================================================================

/// Stateful exercise for continuous verse typing
/// User types verses in sequence until making a mistake or completing the chain
/// Tracks current position and progress through a chapter or range
#[derive(Debug)]
pub struct AyahChainExercise {
    node_id: String,
    verses: Vec<Verse>,
    current_index: usize,
    completed_count: usize,
    is_complete: bool,
    mistake_made: bool,
}

impl AyahChainExercise {
    /// Create a new Ayah Chain exercise
    ///
    /// Queries the database for all verses in the specified chapter
    /// User will type verses in sequence starting from the first verse
    pub async fn new(
        chapter_node_id: String,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Parse chapter number from node_id (format: "CHAPTER:n")
        let chapter_num: i32 = chapter_node_id
            .strip_prefix("CHAPTER:")
            .ok_or_else(|| anyhow::anyhow!("Invalid chapter node ID: {}", chapter_node_id))?
            .parse()?;

        // Get all verses for the chapter
        let verses = content_repo.get_verses_for_chapter(chapter_num).await?;

        if verses.is_empty() {
            return Err(anyhow::anyhow!(
                "No verses found for chapter {}",
                chapter_num
            ));
        }

        Ok(Self {
            node_id: chapter_node_id,
            verses,
            current_index: 0,
            completed_count: 0,
            is_complete: false,
            mistake_made: false,
        })
    }

    /// Create an Ayah Chain for a specific verse range within a chapter
    pub async fn new_range(
        chapter_num: i32,
        start_verse: i32,
        end_verse: i32,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Get all verses for the chapter
        let all_verses = content_repo.get_verses_for_chapter(chapter_num).await?;

        // Filter to requested range
        let verses: Vec<Verse> = all_verses
            .into_iter()
            .filter(|v| v.verse_number >= start_verse && v.verse_number <= end_verse)
            .collect();

        if verses.is_empty() {
            return Err(anyhow::anyhow!(
                "No verses found for chapter {} range {}:{}",
                chapter_num,
                start_verse,
                end_verse
            ));
        }

        let node_id = format!("CHAPTER:{}:{}:{}", chapter_num, start_verse, end_verse);

        Ok(Self {
            node_id,
            verses,
            current_index: 0,
            completed_count: 0,
            is_complete: false,
            mistake_made: false,
        })
    }

    /// Get the current verse being tested
    pub fn current_verse(&self) -> Option<&Verse> {
        if self.is_complete || self.mistake_made {
            return None;
        }
        self.verses.get(self.current_index)
    }

    /// Get the current verse reference (e.g., "1:1")
    pub fn current_verse_ref(&self) -> Option<String> {
        self.current_verse().map(|v| v.key.clone())
    }

    /// Submit an answer for the current verse and advance
    ///
    /// Returns Ok(true) if answer was correct and chain continues
    /// Returns Ok(false) if answer was wrong (chain ends)
    /// Returns Err if exercise is already complete or invalid state
    pub fn submit_answer(&mut self, user_input: &str) -> Result<bool> {
        if self.is_complete {
            return Err(anyhow::anyhow!("Exercise already complete"));
        }

        if self.mistake_made {
            return Err(anyhow::anyhow!("Chain already broken by mistake"));
        }

        let current_verse = self
            .current_verse()
            .ok_or_else(|| anyhow::anyhow!("No current verse"))?;

        // Check answer using Arabic normalization
        let normalized_input = MemorizationExercise::normalize_arabic(user_input);
        let normalized_correct =
            MemorizationExercise::normalize_arabic(&current_verse.text_uthmani);

        if normalized_input == normalized_correct {
            // Correct! Advance to next verse
            self.completed_count += 1;
            self.current_index += 1;

            // Check if we've completed all verses
            if self.current_index >= self.verses.len() {
                self.is_complete = true;
            }

            Ok(true)
        } else {
            // Mistake! Chain broken
            self.mistake_made = true;
            Ok(false)
        }
    }

    /// Get completion stats
    pub fn get_stats(&self) -> AyahChainStats {
        AyahChainStats {
            total_verses: self.verses.len(),
            completed_count: self.completed_count,
            is_complete: self.is_complete,
            mistake_made: self.mistake_made,
        }
    }

    /// Reset the chain to start over
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.completed_count = 0;
        self.is_complete = false;
        self.mistake_made = false;
    }
}

/// Statistics for Ayah Chain performance
#[derive(Debug, Clone)]
pub struct AyahChainStats {
    pub total_verses: usize,
    pub completed_count: usize,
    pub is_complete: bool,
    pub mistake_made: bool,
}

impl Exercise for AyahChainExercise {
    fn generate_question(&self) -> String {
        if let Some(verse) = self.current_verse() {
            format!(
                "Ayah Chain: {}/{}\n\nType verse {}:",
                self.completed_count + 1,
                self.verses.len(),
                verse.key
            )
        } else if self.is_complete {
            format!(
                "🎉 Chain Complete! You successfully typed all {} verses!",
                self.verses.len()
            )
        } else if self.mistake_made {
            format!(
                "Chain broken. You completed {}/{} verses.",
                self.completed_count,
                self.verses.len()
            )
        } else {
            "No current verse".to_string()
        }
    }

    fn check_answer(&self, answer: &str) -> bool {
        if let Some(verse) = self.current_verse() {
            let normalized_input = MemorizationExercise::normalize_arabic(answer);
            let normalized_correct = MemorizationExercise::normalize_arabic(&verse.text_uthmani);
            normalized_input == normalized_correct
        } else {
            false
        }
    }

    fn get_hint(&self) -> Option<String> {
        self.current_verse().map(|verse| {
            // Show first 3 words of the verse
            let words: Vec<&str> = verse.text_uthmani.split_whitespace().collect();
            let hint_words: Vec<&str> = words.iter().take(3).copied().collect();
            format!("Starts with: {}", hint_words.join(" "))
        })
    }

    fn get_node_id(&self) -> &str {
        &self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "ayah_chain"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Ayah chain tests require full database setup
        // See ayah_chain_tests.rs for comprehensive tests
    }
}

// Include comprehensive integration tests
#[cfg(test)]
#[path = "ayah_chain_tests.rs"]
mod ayah_chain_tests;
