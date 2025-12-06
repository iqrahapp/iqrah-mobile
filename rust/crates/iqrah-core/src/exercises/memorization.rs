// exercises/memorization.rs
// Memorization exercises for Quranic learning

use super::types::Exercise;
use crate::semantic::grader::{SemanticGradeLabel, SemanticGrader, SEMANTIC_EMBEDDER};
use crate::{ContentRepository, KnowledgeNode};
use anyhow::Result;
use rand::seq::SliceRandom;

/// Exercise for memorizing Quranic words
/// Tests the user's ability to recall the exact Arabic text using semantic similarity
#[derive(Debug)]
pub struct MemorizationExercise {
    pub node_id: i64,
    word_text: String,
    verse_context: Option<String>,
}

impl MemorizationExercise {
    /// Create a new memorization exercise
    pub async fn new(
        node_id: i64,
        ukey: &str,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Resolve base ID if this is a knowledge node
        let base_id = if let Some((bid, _)) = crate::domain::node_id::decode_knowledge_id(node_id) {
            bid
        } else {
            node_id
        };

        // Get the word text from the repository using the integer ID
        let word_text = content_repo
            .get_quran_text(base_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Word text not found for node ID: {}", base_id))?;

        // Try to get verse context for hints by parsing the ukey
        let verse_context = if ukey.starts_with("WORD:") {
            let parts: Vec<&str> = ukey.split(':').collect();
            if parts.len() >= 3 {
                let verse_ukey = format!("VERSE:{}:{}", parts[1], parts[2]);
                // To get the verse text, we'd need to look up its i64 ID first.
                // This is a simplification for now. A more robust solution might involve
                // a `get_node_by_ukey` call to get the verse's i64 ID.
                // For the purpose of this refactoring, we will assume this logic might
                // need to be adapted or that `get_quran_text` could also accept a ukey.
                // Let's assume a lookup is needed.
                if let Some(verse_node) = content_repo.get_node_by_ukey(&verse_ukey).await? {
                    content_repo
                        .get_quran_text(verse_node.id)
                        .await
                        .ok()
                        .flatten()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            node_id,
            word_text,
            verse_context,
        })
    }

    /// Normalize Arabic text for comparison (remove diacritics/tashkeel and normalize letters)
    pub fn normalize_arabic(text: &str) -> String {
        let normalized = text
            .chars()
            // First, remove all diacritical marks
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
                    '\u{0658}' // Mark noon ghunna
                )
            })
            // Normalize letter variants
            .map(|c| match c {
                'ٱ' => 'ا', // Alif with wasla -> regular Alif
                'أ' => 'ا', // Alif with hamza above -> regular Alif
                'إ' => 'ا', // Alif with hamza below -> regular Alif
                'آ' => 'ا', // Alif with madda -> regular Alif
                'ٰ' => 'ا',  // Alif khanjariyyah -> regular Alif
                'ى' => 'ي', // Alif maqsurah -> Ya
                'ة' => 'ه', // Ta marbuta -> Ha
                'ۀ' => 'ه', // Hamza on Ha -> Ha
                _ => c,
            })
            .collect::<String>();

        // Normalize whitespace: split by whitespace and rejoin with single spaces
        // This handles multiple spaces, tabs, newlines, etc.
        normalized
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Get the correct word text (used by ExerciseService for semantic grading)
    pub fn get_word_text(&self) -> &str {
        &self.word_text
    }
}

impl Exercise for MemorizationExercise {
    fn generate_question(&self) -> String {
        "Recall the word".to_string()
    }

    fn check_answer(&self, answer: &str) -> bool {
        // Normalize both answer and correct text (remove diacritics)
        let normalized_answer = Self::normalize_arabic(answer);
        let normalized_correct = Self::normalize_arabic(&self.word_text);

        // Get embedder, return false if not initialized
        let embedder = match SEMANTIC_EMBEDDER.get() {
            Some(e) => e,
            None => {
                tracing::error!("Semantic embedder not initialized! Call ExerciseService::init_semantic_model() first");
                return false;
            }
        };

        let grader = SemanticGrader::new(embedder);

        // Use semantic grading on normalized Arabic text
        let grade = match grader.grade_answer(&normalized_answer, &normalized_correct) {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("Semantic grading failed: {}", e);
                return false;
            }
        };

        tracing::debug!(
            "Memorization semantic grading for '{}': {:?} (similarity: {:.3})",
            normalized_answer,
            grade.label,
            grade.similarity
        );

        // For Arabic memorization, we can be more strict
        // Accept only Excellent grade (≥ 0.85 similarity)
        grade.label == SemanticGradeLabel::Excellent
    }

    fn get_hint(&self) -> Option<String> {
        if let Some(verse) = &self.verse_context {
            Some(format!("Verse context: {}", verse))
        } else {
            // Provide first character as hint
            self.word_text
                .chars()
                .next()
                .map(|c| format!("First character: {}", c))
        }
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "memorization"
    }
}

// ============================================================================
// Exercise 2: Next Word MCQ
// ============================================================================

/// Exercise for testing memorization of the next word in a verse
/// Given a verse with the last word missing, select the correct word from options
#[derive(Debug)]
pub struct NextWordMcqExercise {
    node_id: i64,
    #[allow(dead_code)] // May be used for debugging or future features
    verse_key: String,
    verse_prefix: String,   // Verse text without last word
    correct_answer: String, // Last word
    options: Vec<String>,   // 4 options (shuffled)
    difficulty: NextWordDifficulty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextWordDifficulty {
    Easy,   // Distractors from same verse
    Medium, // Distractors from same chapter
    Hard,   // Distractors from similar patterns (TODO: requires similarity metrics)
}

impl NextWordMcqExercise {
    /// Create a new Next Word MCQ exercise
    pub async fn new(
        verse_node_id: i64,
        difficulty: NextWordDifficulty,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
        // Get the node to access its ukey
        let node = content_repo
            .get_node(verse_node_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Node not found: {}", verse_node_id))?;

        // Parse knowledge node to get base verse ID
        let base_node_id = if let Some(kn) = KnowledgeNode::parse(&node.ukey) {
            kn.base_node_id
        } else {
            node.ukey.clone()
        };

        // Extract verse key from node ID (e.g., "VERSE:1:1" -> "1:1")
        let verse_key = base_node_id
            .strip_prefix("VERSE:")
            .unwrap_or(&base_node_id)
            .to_string();

        // Get all words for the verse
        let words = content_repo.get_words_for_verse(&verse_key).await?;

        if words.is_empty() {
            return Err(anyhow::anyhow!("No words found for verse: {}", verse_key));
        }

        // Last word is the correct answer
        let last_word = words.last().unwrap();
        let correct_answer = last_word.text_uthmani.clone();

        // Build verse prefix (all words except last)
        let prefix_words: Vec<&str> = words[..words.len() - 1]
            .iter()
            .map(|w| w.text_uthmani.as_str())
            .collect();
        let verse_prefix = prefix_words.join(" ");

        // Generate distractors based on difficulty
        let distractors = match difficulty {
            NextWordDifficulty::Easy => {
                // Use other words from the same verse
                Self::generate_distractors_from_verse(&words, &correct_answer, 3)
            }
            NextWordDifficulty::Medium => {
                // Use words from same chapter
                Self::generate_distractors_from_chapter(
                    content_repo,
                    &verse_key,
                    &correct_answer,
                    3,
                )
                .await?
            }
            NextWordDifficulty::Hard => {
                // TODO: Use phonetically/visually similar words
                // For now, fallback to medium difficulty
                Self::generate_distractors_from_chapter(
                    content_repo,
                    &verse_key,
                    &correct_answer,
                    3,
                )
                .await?
            }
        };

        // Combine correct + distractors and shuffle
        let mut options = vec![correct_answer.clone()];
        options.extend(distractors);
        options.shuffle(&mut rand::thread_rng());

        Ok(Self {
            node_id: verse_node_id,
            verse_key,
            verse_prefix,
            correct_answer,
            options,
            difficulty,
        })
    }

    /// Generate distractors from the same verse
    fn generate_distractors_from_verse(
        words: &[crate::Word],
        correct: &str,
        count: usize,
    ) -> Vec<String> {
        words
            .iter()
            .map(|w| w.text_uthmani.as_str())
            .filter(|&w| w != correct)
            .take(count)
            .map(|s| s.to_string())
            .collect()
    }

    /// Generate distractors from the same chapter
    async fn generate_distractors_from_chapter(
        content_repo: &dyn ContentRepository,
        verse_key: &str,
        correct: &str,
        count: usize,
    ) -> Result<Vec<String>> {
        // Extract chapter number from verse key (e.g., "1:1" -> 1)
        let chapter_number: i32 = verse_key
            .split(':')
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| anyhow::anyhow!("Invalid verse key: {}", verse_key))?;

        // Get all verses for the chapter
        let verses = content_repo.get_verses_for_chapter(chapter_number).await?;

        // Collect words from different verses
        let mut distractor_words = Vec::new();
        for verse in verses.iter().take(10) {
            // Limit to first 10 verses for performance
            if verse.key != verse_key {
                let words = content_repo.get_words_for_verse(&verse.key).await?;
                distractor_words.extend(words.into_iter().map(|w| w.text_uthmani));
            }
        }

        // Filter out the correct answer and take random samples
        let mut distractors: Vec<String> = distractor_words
            .into_iter()
            .filter(|w| w != correct)
            .collect();

        distractors.shuffle(&mut rand::thread_rng());
        distractors.truncate(count);

        // If we don't have enough, pad with duplicates (shouldn't happen in practice)
        while distractors.len() < count {
            distractors.push(format!("[Option {}]", distractors.len() + 1));
        }

        Ok(distractors)
    }

    /// Get the options for this MCQ
    pub fn get_options(&self) -> &[String] {
        &self.options
    }

    /// Get the difficulty level
    pub fn get_difficulty(&self) -> NextWordDifficulty {
        self.difficulty
    }
}

impl Exercise for NextWordMcqExercise {
    fn generate_question(&self) -> String {
        format!("{} _____", self.verse_prefix)
    }

    fn check_answer(&self, answer: &str) -> bool {
        let normalized_answer = MemorizationExercise::normalize_arabic(answer);
        let normalized_correct = MemorizationExercise::normalize_arabic(&self.correct_answer);
        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        // Provide first letter as hint
        self.correct_answer
            .chars()
            .next()
            .map(|c| format!("First letter: {}", c))
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "next_word_mcq"
    }
}

// ============================================================================
// Exercise 3: Find the Missing Word (MCQ)
// ============================================================================

/// Exercise for finding a missing word in the middle of a verse
/// A word is removed from the verse, user selects the correct missing word
#[derive(Debug)]
pub struct MissingWordMcqExercise {
    node_id: i64,
    verse_with_blank: String, // "بِسْمِ _____ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"
    correct_answer: String,   // "ٱللَّهِ"
    options: Vec<String>,     // 4 options (shuffled)
}

impl MissingWordMcqExercise {
    /// Create a new Missing Word MCQ exercise
    /// `word_node_id` should be the i64 ID of the word
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

        // Parse word node ID to extract verse key and position
        // Format: "WORD_INSTANCE:chapter:verse:position" or old format "WORD:chapter:verse:position"
        let parts: Vec<&str> = base_node_id.split(':').collect();
        if parts.len() < 4 {
            return Err(anyhow::anyhow!(
                "Invalid word node ID format: {}",
                base_node_id
            ));
        }

        let verse_key = format!("{}:{}", parts[1], parts[2]);
        let target_position: i32 = parts[3]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid position in word ID: {}", base_node_id))?;

        // Get all words for the verse
        let words = content_repo.get_words_for_verse(&verse_key).await?;

        if words.is_empty() {
            return Err(anyhow::anyhow!("No words found for verse: {}", verse_key));
        }

        // Find the target word
        let target_word = words
            .iter()
            .find(|w| w.position == target_position)
            .ok_or_else(|| anyhow::anyhow!("Word at position {} not found", target_position))?;

        let correct_answer = target_word.text_uthmani.clone();

        // Build verse with blank
        let verse_with_blank = words
            .iter()
            .map(|w| {
                if w.position == target_position {
                    "_____"
                } else {
                    w.text_uthmani.as_str()
                }
            })
            .collect::<Vec<&str>>()
            .join(" ");

        // Generate distractors from other words in the same verse
        let mut distractors: Vec<String> = words
            .iter()
            .filter(|w| w.position != target_position)
            .map(|w| w.text_uthmani.clone())
            .collect();

        distractors.shuffle(&mut rand::thread_rng());
        distractors.truncate(3);

        // If we don't have enough distractors, pad with placeholders
        while distractors.len() < 3 {
            distractors.push(format!("[Option {}]", distractors.len() + 1));
        }

        // Combine correct + distractors and shuffle
        let mut options = vec![correct_answer.clone()];
        options.extend(distractors);
        options.shuffle(&mut rand::thread_rng());

        Ok(Self {
            node_id: word_node_id,
            verse_with_blank,
            correct_answer,
            options,
        })
    }

    /// Get the options for this MCQ
    pub fn get_options(&self) -> &[String] {
        &self.options
    }
}

impl Exercise for MissingWordMcqExercise {
    fn generate_question(&self) -> String {
        self.verse_with_blank.clone()
    }

    fn check_answer(&self, answer: &str) -> bool {
        let normalized_answer = MemorizationExercise::normalize_arabic(answer);
        let normalized_correct = MemorizationExercise::normalize_arabic(&self.correct_answer);
        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        // Provide first letter as hint
        self.correct_answer
            .chars()
            .next()
            .map(|c| format!("First letter: {}", c))
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "missing_word_mcq"
    }
}

// ============================================================================
// Exercise 6: Cloze Deletion (Text Input)
// ============================================================================

/// Exercise for recalling a missing word with text input
/// Similar to MissingWordMcqExercise but requires typing the answer
#[derive(Debug)]
pub struct ClozeDeletionExercise {
    node_id: i64,
    verse_with_blank: String,        // "بِسْمِ _____ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"
    correct_answer: String,          // "ٱللَّهِ"
    hint_letters: Option<Vec<char>>, // Optional: jumbled letters of the word
}

impl ClozeDeletionExercise {
    /// Create a new Cloze Deletion exercise
    pub async fn new(
        word_node_id: i64,
        show_letters_hint: bool,
        content_repo: &dyn ContentRepository,
    ) -> Result<Self> {
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

        // Parse word node ID
        let parts: Vec<&str> = base_node_id.split(':').collect();
        if parts.len() < 4 {
            return Err(anyhow::anyhow!(
                "Invalid word node ID format: {}",
                base_node_id
            ));
        }

        let verse_key = format!("{}:{}", parts[1], parts[2]);
        let target_position: i32 = parts[3]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid position in word ID: {}", base_node_id))?;

        // Get all words for the verse
        let words = content_repo.get_words_for_verse(&verse_key).await?;

        if words.is_empty() {
            return Err(anyhow::anyhow!("No words found for verse: {}", verse_key));
        }

        // Find the target word
        let target_word = words
            .iter()
            .find(|w| w.position == target_position)
            .ok_or_else(|| anyhow::anyhow!("Word at position {} not found", target_position))?;

        let correct_answer = target_word.text_uthmani.clone();

        // Build verse with blank
        let verse_with_blank = words
            .iter()
            .map(|w| {
                if w.position == target_position {
                    "_____"
                } else {
                    w.text_uthmani.as_str()
                }
            })
            .collect::<Vec<&str>>()
            .join(" ");

        // Generate jumbled letters hint if requested
        let hint_letters = if show_letters_hint {
            let mut chars: Vec<char> = correct_answer.chars().collect();
            chars.shuffle(&mut rand::thread_rng());
            Some(chars)
        } else {
            None
        };

        Ok(Self {
            node_id: word_node_id,
            verse_with_blank,
            correct_answer,
            hint_letters,
        })
    }

    /// Get the jumbled letters hint (if enabled)
    pub fn get_hint_letters(&self) -> Option<&[char]> {
        self.hint_letters.as_deref()
    }
}

impl Exercise for ClozeDeletionExercise {
    fn generate_question(&self) -> String {
        if let Some(letters) = &self.hint_letters {
            let letters_str: String = letters.iter().collect::<String>();
            format!(
                "{}\nAvailable letters: {}",
                self.verse_with_blank, letters_str
            )
        } else {
            self.verse_with_blank.clone()
        }
    }

    fn check_answer(&self, answer: &str) -> bool {
        let normalized_answer = MemorizationExercise::normalize_arabic(answer);
        let normalized_correct = MemorizationExercise::normalize_arabic(&self.correct_answer);
        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        // Provide first AND last letter as hint
        let chars: Vec<char> = self.correct_answer.chars().collect();
        if chars.len() >= 2 {
            Some(format!(
                "Starts with '{}' and ends with '{}'",
                chars[0],
                chars[chars.len() - 1]
            ))
        } else {
            chars.first().map(|c| format!("First letter: {}", c))
        }
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "cloze_deletion"
    }
}

// ============================================================================
// Exercise 7: First Letter Hint Recall
// ============================================================================

/// Exercise for recalling a word with first letter provided as hint
/// Bridges the gap between recognition and pure recall
#[derive(Debug)]
pub struct FirstLetterHintExercise {
    node_id: i64,
    verse_with_hint: String, // "بِسْمِ ٱـ_____ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ"
    correct_answer: String,  // "ٱللَّهِ"
    first_letter: char,      // 'ٱ'
}

impl FirstLetterHintExercise {
    /// Create a new First Letter Hint exercise
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

        // Parse word node ID
        let parts: Vec<&str> = base_node_id.split(':').collect();
        if parts.len() < 4 {
            return Err(anyhow::anyhow!(
                "Invalid word node ID format: {}",
                base_node_id
            ));
        }

        let verse_key = format!("{}:{}", parts[1], parts[2]);
        let target_position: i32 = parts[3]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid position in word ID: {}", base_node_id))?;

        // Get all words for the verse
        let words = content_repo.get_words_for_verse(&verse_key).await?;

        if words.is_empty() {
            return Err(anyhow::anyhow!("No words found for verse: {}", verse_key));
        }

        // Find the target word
        let target_word = words
            .iter()
            .find(|w| w.position == target_position)
            .ok_or_else(|| anyhow::anyhow!("Word at position {} not found", target_position))?;

        let correct_answer = target_word.text_uthmani.clone();

        // Get first letter
        let first_letter = correct_answer
            .chars()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty word text"))?;

        // Build verse with first letter hint
        let verse_with_hint = words
            .iter()
            .map(|w| {
                if w.position == target_position {
                    format!("{}ـ_____", first_letter)
                } else {
                    w.text_uthmani.clone()
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        Ok(Self {
            node_id: word_node_id,
            verse_with_hint,
            correct_answer,
            first_letter,
        })
    }

    /// Get the first letter hint
    pub fn get_first_letter(&self) -> char {
        self.first_letter
    }
}

impl Exercise for FirstLetterHintExercise {
    fn generate_question(&self) -> String {
        self.verse_with_hint.clone()
    }

    fn check_answer(&self, answer: &str) -> bool {
        let normalized_answer = MemorizationExercise::normalize_arabic(answer);
        let normalized_correct = MemorizationExercise::normalize_arabic(&self.correct_answer);
        normalized_answer == normalized_correct
    }

    fn get_hint(&self) -> Option<String> {
        // Provide word length hint
        let normalized = MemorizationExercise::normalize_arabic(&self.correct_answer);
        Some(format!(
            "Word length: {} letters",
            normalized.chars().count()
        ))
    }

    fn get_node_id(&self) -> i64 {
        self.node_id
    }

    fn get_type_name(&self) -> &'static str {
        "first_letter_hint"
    }
}

// ============================================================================
// Tests
// ============================================================================

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
