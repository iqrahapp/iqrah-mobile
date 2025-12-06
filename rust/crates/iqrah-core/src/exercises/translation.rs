// exercises/translation.rs
// Translation exercises: "What does this mean?"

use super::types::Exercise;
use crate::semantic::grader::{SemanticGradeLabel, SemanticGrader, SEMANTIC_EMBEDDER};
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;
use rand::seq::SliceRandom;

/// Exercise for testing understanding of word meanings
/// Tests the user's knowledge of translation/meaning using semantic similarity
#[derive(Debug)]
pub struct TranslationExercise {
    node_id: i64,
    word_text: String,
    translation: String,
}

impl TranslationExercise {
    /// Create a new translation exercise
    pub async fn new(
        node_id: i64,
        _ukey: &str,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Resolve base ID if this is a knowledge node
        let base_id = if let Some((bid, _)) = crate::domain::node_id::decode_knowledge_id(node_id) {
            bid
        } else {
            node_id
        };

        // Get the word text
        let word_text = content_repo
            .get_quran_text(base_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word text not found for node: {}", base_id))?;

        // Get translation (default to English for now)
        let translation = content_repo
            .get_translation(base_id, "en")
            .await?
            .unwrap_or_else(|| "[Translation not available]".to_string());

        Ok(Self {
            node_id,
            word_text,
            translation,
        })
    }

    /// Get the correct translation (used by ExerciseService for semantic grading)
    pub fn get_translation(&self) -> &str {
        &self.translation
    }
}

impl Exercise for TranslationExercise {
    fn generate_question(&self) -> String {
        format!("What does '{}' mean?", self.word_text)
    }

    fn check_answer(&self, answer: &str) -> bool {
        // Get embedder, return false if not initialized
        let embedder = match SEMANTIC_EMBEDDER.get() {
            Some(e) => e,
            None => {
                tracing::error!("Semantic embedder not initialized! Call ExerciseService::init_semantic_model() first");
                return false;
            }
        };

        let grader = SemanticGrader::new(embedder);

        // Grade the answer, return false on error
        let grade = match grader.grade_answer(answer, &self.translation) {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("Semantic grading failed: {}", e);
                return false;
            }
        };

        tracing::debug!(
            "Semantic grading for '{}': {:?} (similarity: {:.3})",
            answer,
            grade.label,
            grade.similarity
        );

        // Accept Excellent and Partial grades as correct
        // Incorrect grade means similarity is too low (< 0.70)
        grade.label != SemanticGradeLabel::Incorrect
    }

    fn get_hint(&self) -> Option<String> {
        // Provide first word of translation as hint
        self.translation
            .split_whitespace()
            .next()
            .map(|first_word| format!("Starts with: {}", first_word))
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "translation"
    }
}

// ============================================================================
// Exercise 15: Contextual Translation MCQ
// ============================================================================

/// Exercise for testing word translation in verse context (MCQ)
/// Shows the full verse and asks for the meaning of a specific word
#[derive(Debug)]
pub struct ContextualTranslationExercise {
    node_id: i64,
    verse_text: String,
    highlighted_word: String,
    correct_answer: String,
    options: Vec<String>, // 4 options (shuffled)
}

impl ContextualTranslationExercise {
    /// Create a new contextual translation exercise
    ///
    /// Queries the database for:
    /// - The word and its verse context
    /// - The word's translation (correct answer)
    /// - Alternative translations from other words (distractors)
    pub async fn new(
        word_node_id: i64,
        ukey: &str,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Parse knowledge node
        let base_ukey = if let Some(kn) = KnowledgeNode::parse(ukey) {
            kn.base_node_id
        } else {
            ukey.to_string()
        };

        // Get the word text
        let highlighted_word = content_repo
            .get_quran_text(word_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word text not found for node: {}", word_node_id))?;

        // Parse node_id to get verse_key
        // Format: "WORD_INSTANCE:chapter:verse:position"
        let parts: Vec<&str> = base_ukey.split(':').collect();
        if parts.len() != 4 {
            return Err(anyhow::anyhow!(
                "Invalid word node ID format: {}",
                base_ukey
            ));
        }

        let chapter = parts[1];
        let verse_num = parts[2];
        let verse_key = format!("{}:{}", chapter, verse_num);

        // Get verse text for context
        let verse_node_ukey = format!("VERSE:{}", verse_key);
        let verse_node = content_repo
            .get_node_by_ukey(&verse_node_ukey)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Verse node not found: {}", verse_node_ukey))?;
        let verse_text = content_repo
            .get_quran_text(verse_node.id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Verse text not found: {}", verse_node_ukey))?;

        // Get the correct translation for this word
        let correct_answer = content_repo
            .get_translation(word_node_id, "en")
            .await?
            .ok_or_else(|| anyhow::anyhow!("Translation not found for word: {}", word_node_id))?;

        // Generate distractors from other words in the same chapter
        let chapter_num: i32 = chapter.parse()?;
        let distractors =
            Self::generate_distractors(&correct_answer, chapter_num, &verse_key, content_repo)
                .await?;

        // Combine correct answer with distractors and shuffle
        let mut options = vec![correct_answer.clone()];
        options.extend(distractors);
        options.shuffle(&mut rand::thread_rng());

        Ok(Self {
            node_id: word_node_id,
            verse_text,
            highlighted_word,
            correct_answer,
            options,
        })
    }

    /// Generate distractor translations from other words in the same chapter
    async fn generate_distractors(
        correct: &str,
        chapter_num: i32,
        exclude_verse_key: &str,
        content_repo: &dyn ContentRepository,
    ) -> Result<Vec<String>> {
        let mut distractors = Vec::new();

        // Get all verses from the chapter
        let verses = content_repo.get_verses_for_chapter(chapter_num).await?;

        // Collect unique translations from different verses
        for verse in verses.iter().take(20) {
            // Limit to first 20 verses for performance
            if verse.key == exclude_verse_key {
                continue; // Skip the target verse
            }

            // Get words for this verse
            let words = content_repo.get_words_for_verse(&verse.key).await?;

            // Try to get translations for some words
            for word in words.iter().take(5) {
                // Sample a few words per verse
                if let Some(translation) = content_repo.get_word_translation(word.id, 1).await?
                // Default translator ID = 1
                {
                    if translation != correct && !distractors.contains(&translation) {
                        distractors.push(translation);
                        if distractors.len() >= 3 {
                            return Ok(distractors);
                        }
                    }
                }
            }
        }

        // If we don't have enough distractors, add generic ones
        while distractors.len() < 3 {
            let generic = match distractors.len() {
                0 => "the one",
                1 => "those",
                _ => "it",
            };
            if generic != correct {
                distractors.push(generic.to_string());
            }
        }

        Ok(distractors.into_iter().take(3).collect())
    }

    /// Get the MCQ options
    pub fn get_options(&self) -> &[String] {
        &self.options
    }

    /// Get the correct answer
    pub fn get_correct_answer(&self) -> &str {
        &self.correct_answer
    }
}

impl Exercise for ContextualTranslationExercise {
    fn generate_question(&self) -> String {
        format!(
            "In the verse:\n\n{}\n\nWhat does '{}' mean in this context?",
            self.verse_text, self.highlighted_word
        )
    }

    fn check_answer(&self, answer: &str) -> bool {
        // Normalize for comparison
        let normalized_answer = answer.trim().to_lowercase();
        let normalized_correct = self.correct_answer.trim().to_lowercase();

        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        // Show first letter of correct answer
        self.correct_answer
            .chars()
            .next()
            .map(|c| format!("Starts with: {}", c.to_uppercase()))
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "contextual_translation"
    }
}

#[cfg(test)]
mod tests {
    // Note: Tests require semantic model to be initialized
    // Run integration tests with a real model instead
}
