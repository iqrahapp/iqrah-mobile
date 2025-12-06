// exercises/pos_tagging.rs
// Exercise 22: Part of Speech Tagging - Identify word as Noun/Verb/Particle

use super::types::Exercise;
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;
use rand::seq::SliceRandom;

// ============================================================================
// Exercise 22: Part of Speech Tagging
// ============================================================================

/// Exercise for testing grammatical knowledge
/// Given a word from a verse, user must identify its part of speech
/// Tests understanding of Arabic grammar basics
#[derive(Debug)]
pub struct PosTaggingExercise {
    node_id: i64,
    word_text: String,
    verse_context: String,
    correct_pos: String,
    options: Vec<String>,
}

impl PosTaggingExercise {
    /// Create a new Part of Speech Tagging exercise
    ///
    /// Queries the database for:
    /// - Word text
    /// - Morphology segments with POS tags
    /// - Verse context
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

        // Parse node_id to get verse_key and word position
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
        let verse_key = format!("{}:{}", chapter, verse_num);
        let position: i32 = parts[3].parse()?;

        // Get word text
        let word_text = content_repo
            .get_quran_text(word_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word text not found: {}", base_node_id))?;

        // Get word from database to get its ID
        let words = content_repo.get_words_for_verse(&verse_key).await?;
        let word = words
            .iter()
            .find(|w| w.position == position)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Word not found at position {} in verse {}",
                    position,
                    verse_key
                )
            })?;

        // Get morphology data for the word
        let morphology = content_repo.get_morphology_for_word(word.id).await?;

        // Find the first segment with a POS tag (stem usually has the main POS)
        let correct_pos = morphology
            .iter()
            .find_map(|seg| seg.pos_tag.clone())
            .ok_or_else(|| anyhow::anyhow!("No POS tag found for word ID {}", word.id))?;

        // Get verse context
        let verse_ukey = format!("VERSE:{}", verse_key);
        let verse_node = content_repo
            .get_node_by_ukey(&verse_ukey)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Verse node not found: {}", verse_ukey))?;

        let verse_context = content_repo
            .get_quran_text(verse_node.id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Verse text not found: {}", verse_ukey))?;

        // Generate MCQ options
        let options = Self::generate_options(&correct_pos);

        Ok(Self {
            node_id: word_node_id,
            word_text,
            verse_context,
            correct_pos,
            options,
        })
    }

    /// Generate MCQ options for part of speech
    /// Returns shuffled options including the correct answer
    fn generate_options(correct_pos: &str) -> Vec<String> {
        let all_pos = [
            "noun".to_string(),
            "verb".to_string(),
            "particle".to_string(),
            "pronoun".to_string(),
        ];

        let mut rng = rand::thread_rng();
        let mut options = Vec::new();

        // Add correct answer
        options.push(correct_pos.to_string());

        // Add 3 distractors (or fewer if correct_pos is not in standard list)
        for pos in &all_pos {
            if pos.to_lowercase() != correct_pos.to_lowercase() && options.len() < 4 {
                options.push(pos.clone());
            }
        }

        // Ensure we have exactly 4 options
        while options.len() < 4 {
            options.push("adjective".to_string());
        }

        // Shuffle options
        options.shuffle(&mut rng);
        options
    }

    /// Get the options (for testing)
    pub fn get_options(&self) -> &[String] {
        &self.options
    }

    /// Get the correct POS (for testing)
    pub fn get_correct_pos(&self) -> &str {
        &self.correct_pos
    }

    /// Normalize POS tag for comparison (lowercase, trim whitespace)
    fn normalize_pos(pos: &str) -> String {
        pos.trim().to_lowercase()
    }
}

impl Exercise for PosTaggingExercise {
    fn generate_question(&self) -> String {
        format!(
            "In the verse:\n\n{}\n\nWhat part of speech is the word \"{}\"?\n\nOptions:\n{}",
            self.verse_context,
            self.word_text,
            self.options
                .iter()
                .enumerate()
                .map(|(i, opt)| format!("{}. {}", i + 1, opt))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn check_answer(&self, answer: &str) -> bool {
        let normalized_answer = Self::normalize_pos(answer);
        let normalized_correct = Self::normalize_pos(&self.correct_pos);

        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        // Provide a hint based on the part of speech
        match self.correct_pos.to_lowercase().as_str() {
            "noun" => Some("Hint: This word is a person, place, thing, or idea.".to_string()),
            "verb" => Some("Hint: This word describes an action or state.".to_string()),
            "particle" => Some(
                "Hint: This word is a function word (preposition, conjunction, etc.).".to_string(),
            ),
            "pronoun" => Some("Hint: This word replaces a noun.".to_string()),
            _ => Some("Hint: Consider the grammatical function of this word.".to_string()),
        }
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "pos_tagging"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // POS tagging tests require database setup
    }
}
