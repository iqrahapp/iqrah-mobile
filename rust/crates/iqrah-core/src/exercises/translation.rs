// exercises/translation.rs
// Translation exercise: "What does this mean?"

use super::types::Exercise;
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;

/// Exercise for testing understanding of word meanings
/// Tests the user's knowledge of translation/meaning
pub struct TranslationExercise {
    node_id: String,
    base_node_id: String,
    word_text: String,
    translation: String,
}

impl TranslationExercise {
    /// Create a new translation exercise
    pub async fn new(
        node_id: String,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Parse the knowledge node to get base content node
        let base_node_id = if let Some(kn) = KnowledgeNode::parse(&node_id) {
            kn.base_node_id
        } else {
            // If not a knowledge node, use the node_id directly
            node_id.clone()
        };

        // Get the word text
        let word_text = content_repo
            .get_quran_text(&base_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word text not found for node: {}", base_node_id))?;

        // Get translation (default to English for now)
        let translation = content_repo
            .get_translation(&base_node_id, "en")
            .await?
            .unwrap_or_else(|| "[Translation not available]".to_string());

        Ok(Self {
            node_id,
            base_node_id,
            word_text,
            translation,
        })
    }

    /// Normalize text for fuzzy matching
    fn normalize_for_matching(text: &str) -> String {
        text.to_lowercase()
            .trim()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Check if answer is close enough to translation (fuzzy match)
    fn fuzzy_match(answer: &str, correct: &str) -> bool {
        let norm_answer = Self::normalize_for_matching(answer);
        let norm_correct = Self::normalize_for_matching(correct);

        // Exact match
        if norm_answer == norm_correct {
            return true;
        }

        // Check if one contains the other (for partial answers)
        if norm_correct.contains(&norm_answer) || norm_answer.contains(&norm_correct) {
            return true;
        }

        // Check word overlap (at least 50% of words match)
        let answer_words: Vec<&str> = norm_answer.split_whitespace().collect();
        let correct_words: Vec<&str> = norm_correct.split_whitespace().collect();

        if answer_words.is_empty() || correct_words.is_empty() {
            return false;
        }

        let matching_words = answer_words
            .iter()
            .filter(|w| correct_words.contains(w))
            .count();

        let overlap_ratio = matching_words as f64 / correct_words.len() as f64;
        overlap_ratio >= 0.5
    }
}

impl Exercise for TranslationExercise {
    fn generate_question(&self) -> String {
        format!("What does '{}' mean?", self.word_text)
    }

    fn check_answer(&self, answer: &str) -> bool {
        Self::fuzzy_match(answer, &self.translation)
    }

    fn get_hint(&self) -> Option<String> {
        // Provide first word of translation as hint
        self.translation
            .split_whitespace()
            .next()
            .map(|first_word| format!("Starts with: {}", first_word))
    }

    fn get_node_id(&self) -> &str {
        &self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "translation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(TranslationExercise::fuzzy_match("in the name", "in the name"));
    }

    #[test]
    fn test_case_insensitive() {
        assert!(TranslationExercise::fuzzy_match("In The Name", "in the name"));
    }

    #[test]
    fn test_partial_match() {
        assert!(TranslationExercise::fuzzy_match("the name", "in the name"));
        assert!(TranslationExercise::fuzzy_match("in the", "in the name"));
    }

    #[test]
    fn test_word_overlap() {
        // At least 50% word overlap required
        assert!(TranslationExercise::fuzzy_match("in name", "in the name")); // 2/3 = 66% > 50%
        assert!(TranslationExercise::fuzzy_match("the name", "in the name")); // Substring match
    }

    #[test]
    fn test_no_match() {
        assert!(!TranslationExercise::fuzzy_match("completely different", "in the name"));
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(
            TranslationExercise::normalize_for_matching("  in   the   name  "),
            "in the name"
        );
    }
}
