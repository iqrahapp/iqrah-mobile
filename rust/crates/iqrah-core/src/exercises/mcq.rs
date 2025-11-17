// exercises/mcq.rs
// Multiple Choice Question exercises

use super::types::Exercise;
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;
use rand::seq::SliceRandom;

/// Multiple Choice Question exercise (MCQ)
/// Supports both Arabic (memorization) and Translation questions
#[derive(Debug)]
pub struct McqExercise {
    node_id: String,
    question: String,
    correct_answer: String,
    options: Vec<String>, // All options including correct answer, shuffled
    mcq_type: McqType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McqType {
    /// Arabic -> English (test translation understanding)
    ArToEn,
    /// English -> Arabic (test memorization)
    EnToAr,
}

impl McqExercise {
    /// Create a new MCQ exercise (Arabic to English)
    pub async fn new_ar_to_en(
        node_id: String,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Parse knowledge node to get base content
        let base_node_id = if let Some(kn) = KnowledgeNode::parse(&node_id) {
            kn.base_node_id
        } else {
            node_id.clone()
        };

        // Get the word text (Arabic)
        let word_text = content_repo
            .get_quran_text(&base_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word text not found for node: {}", base_node_id))?;

        // Get correct translation
        let correct_answer = content_repo
            .get_translation(&base_node_id, "en")
            .await?
            .unwrap_or_else(|| "[Translation not available]".to_string());

        // Generate distractors (for now, using simple approach)
        // In production, you'd fetch similar words from database
        let distractors = Self::generate_translation_distractors(&correct_answer, 3);

        // Combine correct + distractors and shuffle
        let mut options = vec![correct_answer.clone()];
        options.extend(distractors);
        options.shuffle(&mut rand::thread_rng());

        let question = format!("What does '{}' mean?", word_text);

        Ok(Self {
            node_id,
            question,
            correct_answer,
            options,
            mcq_type: McqType::ArToEn,
        })
    }

    /// Create a new MCQ exercise (English to Arabic)
    pub async fn new_en_to_ar(
        node_id: String,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Parse knowledge node to get base content
        let base_node_id = if let Some(kn) = KnowledgeNode::parse(&node_id) {
            kn.base_node_id
        } else {
            node_id.clone()
        };

        // Get the word text (Arabic) - this is the correct answer
        let correct_answer = content_repo
            .get_quran_text(&base_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word text not found for node: {}", base_node_id))?;

        // Get translation for the question
        let translation = content_repo
            .get_translation(&base_node_id, "en")
            .await?
            .unwrap_or_else(|| "[Translation not available]".to_string());

        // Generate distractors (Arabic words)
        // In production, you'd fetch similar Arabic words from database
        let distractors = Self::generate_arabic_distractors(&correct_answer, 3);

        // Combine correct + distractors and shuffle
        let mut options = vec![correct_answer.clone()];
        options.extend(distractors);
        options.shuffle(&mut rand::thread_rng());

        let question = format!("Which Arabic word means '{}'?", translation);

        Ok(Self {
            node_id,
            question,
            correct_answer,
            options,
            mcq_type: McqType::EnToAr,
        })
    }

    /// Get the options for this MCQ
    pub fn get_options(&self) -> &[String] {
        &self.options
    }

    /// Get the MCQ type
    pub fn get_mcq_type(&self) -> McqType {
        self.mcq_type
    }

    /// Generate translation distractors (simplified version)
    fn generate_translation_distractors(correct: &str, count: usize) -> Vec<String> {
        // In production, fetch actual similar translations from database
        // For now, using common Islamic/Quranic terms as distractors
        let common_translations = vec![
            "the merciful",
            "the compassionate",
            "the lord",
            "the sustainer",
            "the creator",
            "the master",
            "the king",
            "the judge",
            "the guide",
            "the protector",
        ];

        common_translations
            .iter()
            .filter(|&t| t != &correct.to_lowercase())
            .take(count)
            .map(|s| s.to_string())
            .collect()
    }

    /// Generate Arabic distractors (simplified version)
    fn generate_arabic_distractors(correct: &str, count: usize) -> Vec<String> {
        // In production, fetch actual similar Arabic words from database
        // For now, using common Arabic words as distractors
        let common_words = vec![
            "ٱلْحَمْدُ",
            "رَبِّ",
            "ٱلْعَٰلَمِينَ",
            "ٱلرَّحْمَٰنِ",
            "ٱلرَّحِيمِ",
            "مَٰلِكِ",
            "يَوْمِ",
            "ٱلدِّينِ",
        ];

        common_words
            .iter()
            .filter(|&w| w != &correct)
            .take(count)
            .map(|s| s.to_string())
            .collect()
    }

    /// Normalize for comparison
    fn normalize(text: &str) -> String {
        text.trim().to_lowercase()
    }
}

impl Exercise for McqExercise {
    fn generate_question(&self) -> String {
        self.question.clone()
    }

    fn check_answer(&self, answer: &str) -> bool {
        Self::normalize(answer) == Self::normalize(&self.correct_answer)
    }

    fn get_hint(&self) -> Option<String> {
        // For MCQ, hint could be eliminating one wrong option
        let wrong_options: Vec<_> = self
            .options
            .iter()
            .filter(|opt| Self::normalize(opt) != Self::normalize(&self.correct_answer))
            .collect();

        if !wrong_options.is_empty() {
            Some(format!("It's not: {}", wrong_options[0]))
        } else {
            None
        }
    }

    fn get_node_id(&self) -> &str {
        &self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        match self.mcq_type {
            McqType::ArToEn => "mcq_ar_to_en",
            McqType::EnToAr => "mcq_en_to_ar",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(McqExercise::normalize("  Test  "), "test");
        assert_eq!(McqExercise::normalize("TeSt"), "test");
    }

    #[test]
    fn test_generate_translation_distractors() {
        let distractors = McqExercise::generate_translation_distractors("in the name", 3);
        assert_eq!(distractors.len(), 3);
        assert!(!distractors.contains(&"in the name".to_string()));
    }

    #[test]
    fn test_generate_arabic_distractors() {
        let distractors = McqExercise::generate_arabic_distractors("بِسْمِ", 3);
        assert_eq!(distractors.len(), 3);
        assert!(!distractors.contains(&"بِسْمِ".to_string()));
    }
}
