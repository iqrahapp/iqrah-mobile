// exercises/memorization.rs
// Memorization exercise: "Recall the word"

use super::types::Exercise;
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;

/// Exercise for memorizing Quranic words
/// Tests the user's ability to recall the exact Arabic text
pub struct MemorizationExercise {
    node_id: String,
    base_node_id: String,
    word_text: String,
    verse_context: Option<String>,
}

impl MemorizationExercise {
    /// Create a new memorization exercise
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

        // Try to get verse context for hints
        // Extract verse key from word node ID (e.g., "WORD:1:1:1" -> "VERSE:1:1")
        let verse_context = if base_node_id.starts_with("WORD:") {
            let parts: Vec<&str> = base_node_id.split(':').collect();
            if parts.len() >= 3 {
                let verse_key = format!("VERSE:{}:{}", parts[1], parts[2]);
                content_repo.get_quran_text(&verse_key).await.ok().flatten()
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            node_id,
            base_node_id,
            word_text,
            verse_context,
        })
    }

    /// Normalize Arabic text for comparison (remove diacritics/tashkeel)
    fn normalize_arabic(text: &str) -> String {
        // Remove common Arabic diacritical marks
        text.chars()
            .filter(|c| {
                !matches!(
                    *c,
                    '\u{064B}' | // Fathatan
                    '\u{064C}' | // Dammatan
                    '\u{064D}' | // Kasratan
                    '\u{064E}' | // Fatha
                    '\u{064F}' | // Damma
                    '\u{0650}' | // Kasra
                    '\u{0651}' | // Shadda
                    '\u{0652}' | // Sukun
                    '\u{0653}' | // Maddah
                    '\u{0654}' | // Hamza above
                    '\u{0655}' | // Hamza below
                    '\u{0656}' | // Subscript alef
                    '\u{0657}' | // Inverted damma
                    '\u{0658}'   // Mark noon ghunna
                )
            })
            .collect::<String>()
            .trim()
            .to_string()
    }
}

impl Exercise for MemorizationExercise {
    fn generate_question(&self) -> String {
        "Recall the word".to_string()
    }

    fn check_answer(&self, answer: &str) -> bool {
        let normalized_answer = Self::normalize_arabic(answer);
        let normalized_correct = Self::normalize_arabic(&self.word_text);
        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        if let Some(verse) = &self.verse_context {
            Some(format!("Verse context: {}", verse))
        } else {
            // Provide first character as hint
            self.word_text.chars().next().map(|c| format!("First character: {}", c))
        }
    }

    fn get_node_id(&self) -> &str {
        &self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "memorization"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_arabic() {
        // Test with diacritics
        let with_tashkeel = "بِسْمِ";
        let without_tashkeel = "بسم";

        assert_eq!(
            MemorizationExercise::normalize_arabic(with_tashkeel),
            MemorizationExercise::normalize_arabic(without_tashkeel)
        );
    }

    #[test]
    fn test_normalize_handles_whitespace() {
        let text1 = "  بسم  ";
        let text2 = "بسم";

        assert_eq!(
            MemorizationExercise::normalize_arabic(text1),
            MemorizationExercise::normalize_arabic(text2)
        );
    }
}
