// exercises/grammar.rs
// Grammar and etymology exercises (Nahw & Sarf)

use super::types::Exercise;
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;
use rand::seq::SliceRandom;

// ============================================================================
// Exercise 21: Identify the Root (MCQ)
// ============================================================================

/// Exercise for identifying the Arabic root of a word
/// Tests understanding of Arabic morphology
#[derive(Debug)]
pub struct IdentifyRootExercise {
    node_id: i64,
    word_text: String,    // The word to analyze (e.g., "يَعْلَمُونَ")
    correct_root: String, // The correct root (e.g., "ع-ل-м")
    options: Vec<String>, // 4 root options (shuffled)
}

impl IdentifyRootExercise {
    /// Create a new Identify Root exercise
    ///
    /// Queries the morphology database to get the actual root for the word
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
        let word_text = content_repo
            .get_quran_text(word_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word text not found for node: {}", word_node_id))?;

        // Parse node_id to get verse_key and position
        // Format: "WORD_INSTANCE:chapter:verse:position"
        let parts: Vec<&str> = base_ukey.split(':').collect();
        if parts.len() != 4 {
            return Err(anyhow::anyhow!(
                "Invalid word node ID format: {}",
                base_ukey
            ));
        }

        let chapter = parts[1];
        let verse = parts[2];
        let position: i32 = parts[3].parse()?;
        let verse_key = format!("{}:{}", chapter, verse);

        // Get all words for this verse
        let words = content_repo.get_words_for_verse(&verse_key).await?;

        // Find the word at the specified position
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

        // Query morphology segments for this word
        let morphology = content_repo.get_morphology_for_word(word.id).await?;

        // Get the root_id from the first morphology segment (if available)
        let root_id = morphology
            .iter()
            .find_map(|seg| seg.root_id.clone())
            .ok_or_else(|| anyhow::anyhow!("No root information found for word ID {}", word.id))?;

        // Query the root details
        let root = content_repo
            .get_root_by_id(&root_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Root not found: {}", root_id))?;

        let correct_root = root.root_id.clone();

        // Generate distractor roots
        let distractors = Self::generate_root_distractors(&correct_root, 3);

        // Combine correct + distractors and shuffle
        let mut options = vec![correct_root.clone()];
        options.extend(distractors);
        options.shuffle(&mut rand::thread_rng());

        Ok(Self {
            node_id: word_node_id,
            word_text,
            correct_root,
            options,
        })
    }

    /// Get the options for this MCQ
    pub fn get_options(&self) -> &[String] {
        &self.options
    }

    /// Get the correct root
    pub fn get_correct_root(&self) -> &str {
        &self.correct_root
    }

    /// Generate distractor roots
    fn generate_root_distractors(correct: &str, count: usize) -> Vec<String> {
        // Common trilateral roots in the Quran
        let common_roots = [
            "ك-ت-ب", // write
            "ق-ر-ا", // read
            "ع-ل-م", // know
            "ف-ع-ل", // do
            "ق-و-ل", // say
            "ذ-ه-ب", // go
            "ج-ع-ل", // make
            "ن-ز-ل", // descend
            "ا-م-ن", // believe
            "ك-ف-ر", // disbelieve
            "ر-ح-م", // mercy
            "ح-م-د", // praise
            "س-م-ع", // hear
            "ب-ص-ر", // see
            "خ-ل-ق", // create
            "ر-ز-ق", // provide
            "ش-ك-ر", // thank
            "غ-ف-ر", // forgive
            "ت-و-ب", // repent
            "ص-ل-ي", // pray
        ];

        common_roots
            .iter()
            .filter(|&r| *r != correct)
            .take(count)
            .map(|s| s.to_string())
            .collect()
    }

    /// Normalize root for comparison (remove spaces and dashes)
    fn normalize_root(root: &str) -> String {
        root.chars()
            .filter(|c| *c != '-' && *c != ' ')
            .collect::<String>()
            .trim()
            .to_string()
    }
}

impl Exercise for IdentifyRootExercise {
    fn generate_question(&self) -> String {
        format!("What is the root of '{}'?", self.word_text)
    }

    fn check_answer(&self, answer: &str) -> bool {
        let normalized_answer = Self::normalize_root(answer);
        let normalized_correct = Self::normalize_root(&self.correct_root);
        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        // Provide the number of letters in the root
        let letter_count = self.correct_root.chars().filter(|c| *c != '-').count();
        Some(format!("{}-letter root", letter_count))
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "identify_root"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_root() {
        assert_eq!(IdentifyRootExercise::normalize_root("ع-ل-م"), "علم");
        assert_eq!(IdentifyRootExercise::normalize_root("ع ل م"), "علم");
        assert_eq!(IdentifyRootExercise::normalize_root("علم"), "علم");
        assert_eq!(IdentifyRootExercise::normalize_root("  ك-ت-ب  "), "كتب");
    }

    #[test]
    fn test_generate_root_distractors() {
        let distractors = IdentifyRootExercise::generate_root_distractors("ع-ل-م", 3);
        assert_eq!(distractors.len(), 3);
        assert!(!distractors.contains(&"ع-ل-م".to_string()));
    }
}

// Include comprehensive integration tests
#[cfg(test)]
#[path = "grammar_tests.rs"]
mod grammar_tests;
