// exercises/translation.rs
// Translation exercise: "What does this mean?"

use super::types::Exercise;
use crate::semantic::grader::{SemanticGradeLabel, SemanticGrader, SEMANTIC_EMBEDDER};
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;

/// Exercise for testing understanding of word meanings
/// Tests the user's knowledge of translation/meaning using semantic similarity
pub struct TranslationExercise {
    node_id: String,
    #[allow(dead_code)]
    base_node_id: String,
    word_text: String,
    translation: String,
}

impl TranslationExercise {
    /// Create a new translation exercise
    pub async fn new(node_id: String, content_repo: &dyn ContentRepository) -> Result<Self> {
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

    fn get_node_id(&self) -> &str {
        &self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "translation"
    }
}

#[cfg(test)]
mod tests {
    // Note: Tests require semantic model to be initialized
    // Run integration tests with a real model instead
}
