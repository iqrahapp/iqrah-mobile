// exercises/reverse_cloze.rs
// Exercise 8: Reverse Cloze - Given a word, type the next word

use super::memorization::MemorizationExercise;
use super::types::Exercise;
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;

// ============================================================================
// Exercise 8: Reverse Cloze
// ============================================================================

/// Exercise for testing sequential memory
/// Given a word from a verse, user must type the next word
/// Tests word-to-word progression in memorized verses
#[derive(Debug)]
pub struct ReverseClozeExercise {
    node_id: i64,
    current_word_text: String,
    #[allow(dead_code)] // Used for hints showing verse context
    verse_text: String,
    next_word_text: String,
    #[allow(dead_code)] // Used for potential future features
    verse_key: String,
}

impl ReverseClozeExercise {
    /// Create a new Reverse Cloze exercise
    ///
    /// Queries the database for:
    /// - Current word text
    /// - Next word in sequence (correct answer)
    /// - Full verse text (for context/hint)
    pub async fn new(word_node_id: i64, content_repo: &dyn ContentRepository) -> Result<Self> {
        // Get the node to access its ukey
        let node = content_repo
            .get_node(word_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Node not found: {}", word_node_id))?;

        // Parse knowledge node
        let base_node_id = if let Some(kn) = KnowledgeNode::parse(&node.ukey) {
            kn.base_node_id
        } else {
            node.ukey.clone()
        };

        // Parse node_id to get verse_key and position
        // Format: "WORD_INSTANCE:chapter:verse:position"
        let parts: Vec<&str> = base_node_id.split(':').collect();
        if parts.len() != 4 {
            return Err(anyhow::anyhow!(
                "Invalid word node ID format: {}",
                base_node_id
            ));
        }

        let chapter = parts[1];
        let verse_num = parts[2];
        let position: i32 = parts[3].parse()?;
        let verse_key = format!("{}:{}", chapter, verse_num);

        // Get current word text
        let current_word_text = content_repo
            .get_quran_text(word_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word text not found: {}", base_node_id))?;

        // Get all words for the verse
        let words = content_repo.get_words_for_verse(&verse_key).await?;

        // Find next word
        let next_position = position + 1;
        let next_word = words
            .iter()
            .find(|w| w.position == next_position)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No next word found for {}:{} position {} (end of verse)",
                    chapter,
                    verse_num,
                    position
                )
            })?;

        let next_word_ukey = format!(
            "WORD_INSTANCE:{}:{}:{}",
            chapter, verse_num, next_word.position
        );
        let next_word_node = content_repo
            .get_node_by_ukey(&next_word_ukey)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Next word node not found: {}", next_word_ukey))?;

        let next_word_text = content_repo
            .get_quran_text(next_word_node.id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Next word text not found: {}", next_word_ukey))?;

        // Get verse text for context/hints
        let verse_ukey = format!("VERSE:{}", verse_key);
        let verse_node = content_repo
            .get_node_by_ukey(&verse_ukey)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Verse node not found: {}", verse_ukey))?;

        let verse_text = content_repo
            .get_quran_text(verse_node.id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Verse text not found: {}", verse_ukey))?;

        Ok(Self {
            node_id: word_node_id,
            current_word_text,
            verse_text,
            next_word_text,
            verse_key,
        })
    }

    /// Get the next word (correct answer) for testing
    pub fn get_next_word(&self) -> &str {
        &self.next_word_text
    }
}

impl Exercise for ReverseClozeExercise {
    fn generate_question(&self) -> String {
        format!(
            "After the word:\n\n{}\n\nWhat comes next?",
            self.current_word_text
        )
    }

    fn check_answer(&self, answer: &str) -> bool {
        // Use Arabic normalization (same as MemorizationExercise)
        let normalized_answer = MemorizationExercise::normalize_arabic(answer);
        let normalized_correct = MemorizationExercise::normalize_arabic(&self.next_word_text);

        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        // Show first letter of next word
        let normalized = MemorizationExercise::normalize_arabic(&self.next_word_text);
        normalized
            .chars()
            .next()
            .map(|c| format!("Starts with: {}", c))
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "reverse_cloze"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Reverse cloze tests require full database setup
        // See reverse_cloze_tests.rs for comprehensive tests
    }
}

// Include comprehensive integration tests
#[cfg(test)]
#[path = "reverse_cloze_tests.rs"]
mod reverse_cloze_tests;
