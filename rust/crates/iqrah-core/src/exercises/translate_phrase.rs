// exercises/translate_phrase.rs
// Exercise 20: Translate Phrase (Text Input) - Type English translation for Arabic phrase/verse

use super::types::Exercise;
use crate::ContentRepository;
use anyhow::Result;

// ============================================================================
// Exercise 20: Translate Phrase (Text Input)
// ============================================================================

/// Exercise for testing translation comprehension
/// Given an Arabic verse or phrase, user must type the English translation
/// Tests deep understanding of meaning
#[derive(Debug)]
pub struct TranslatePhraseExercise {
    pub node_id: i64,
    arabic_text: String,
    correct_translation: String,
    verse_key: Option<String>,
}

impl TranslatePhraseExercise {
    /// Create a new Translate Phrase exercise
    ///
    /// Queries the database for:
    /// - Arabic text (verse or phrase)
    /// - English translation (correct answer)
    pub async fn new(
        node_id: i64,
        ukey: &str,
        translator_id: i32,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Get Arabic text
        let arabic_text = content_repo
            .get_quran_text(node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Arabic text not found: {}", node_id))?;

        // Determine verse_key and get translation
        let (verse_key, correct_translation) = if ukey.starts_with("VERSE:") {
            // Verse-level exercise
            let parts: Vec<&str> = ukey.split(':').collect();
            if parts.len() != 3 {
                return Err(anyhow::anyhow!("Invalid verse node ID format: {}", ukey));
            }
            let vk = format!("{}:{}", parts[1], parts[2]);
            let translation = content_repo
                .get_verse_translation(&vk, translator_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Verse translation not found for: {}", vk))?;
            (Some(vk), translation)
        } else if ukey.starts_with("WORD_INSTANCE:") {
            // Word-level exercise (phrase)
            let parts: Vec<&str> = ukey.split(':').collect();
            if parts.len() != 4 {
                return Err(anyhow::anyhow!("Invalid word node ID format: {}", ukey));
            }
            let chapter: i32 = parts[1].parse()?;
            let verse: i32 = parts[2].parse()?;
            let position: i32 = parts[3].parse()?;

            // Get word from database to get its ID
            let verse_key = format!("{}:{}", chapter, verse);
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

            let translation = content_repo
                .get_word_translation(word.id, translator_id)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("Word translation not found for word ID: {}", word.id)
                })?;
            (Some(verse_key), translation)
        } else {
            return Err(anyhow::anyhow!(
                "Unsupported node type for translation exercise: {}",
                ukey
            ));
        };

        Ok(Self {
            node_id,
            arabic_text,
            correct_translation,
            verse_key,
        })
    }

    /// Normalize English text for comparison
    /// - Convert to lowercase
    /// - Remove extra whitespace
    /// - Remove common punctuation
    fn normalize_english(text: &str) -> String {
        text.to_lowercase()
            .chars()
            // Remove common punctuation
            .map(|c| match c {
                '.' | ',' | ';' | ':' | '!' | '?' | '"' | '\'' | '(' | ')' | '[' | ']' => ' ',
                _ => c,
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Get the correct translation (for testing)
    pub fn get_correct_translation(&self) -> &str {
        &self.correct_translation
    }

    /// Get the Arabic text (for testing)
    pub fn get_arabic_text(&self) -> &str {
        &self.arabic_text
    }

    /// Get the verse key (for testing)
    pub fn get_verse_key(&self) -> Option<&str> {
        self.verse_key.as_deref()
    }
}

impl Exercise for TranslatePhraseExercise {
    fn generate_question(&self) -> String {
        format!(
            "Translate to English:\n\n{}\n\nType the English translation:",
            self.arabic_text
        )
    }

    fn check_answer(&self, answer: &str) -> bool {
        // Use English normalization for comparison
        let normalized_answer = Self::normalize_english(answer);
        let normalized_correct = Self::normalize_english(&self.correct_translation);

        // For now, use exact match after normalization
        // Future enhancement: use semantic similarity for partial credit
        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        // Show first few words of the translation as a hint
        let words: Vec<&str> = self.correct_translation.split_whitespace().collect();
        if words.len() > 3 {
            Some(format!(
                "Starts with: {} {} {}...",
                words[0], words[1], words[2]
            ))
        } else if !words.is_empty() {
            Some(format!("Starts with: {}...", words[0]))
        } else {
            None
        }
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "translate_phrase"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Translate phrase tests require database setup
    }
}
