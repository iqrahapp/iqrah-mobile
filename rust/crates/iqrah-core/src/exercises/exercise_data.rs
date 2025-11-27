// exercises/exercise_data.rs
// Modern enum-based exercise data architecture
//
// This module defines the core ExerciseData enum that represents all exercise types
// using a lightweight, key-based approach (no full text storage).

use serde::{Deserialize, Serialize};

/// Core exercise data enum - stores only keys/IDs, no full text
///
/// Each variant contains only the minimum data needed to:
/// 1. Identify what content to fetch (node IDs, verse keys, word positions)
/// 2. Store answer keys for validation
/// 3. Store configuration/options specific to that exercise type
///
/// Full text is fetched on-demand during question generation and validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExerciseData {
    /// Exercise 1: Memorization - Recall exact Arabic text
    /// User must type the exact Arabic word/verse from memory
    Memorization {
        /// Node ID
        node_id: i64,
    },

    /// Exercise 2a: MCQ Arabic to English - Multiple choice translation
    /// Given Arabic word, select English translation
    McqArToEn {
        /// Word node ID
        node_id: i64,
        /// Indices of distractor words (to fetch from database)
        distractor_node_ids: Vec<i64>,
    },

    /// Exercise 2b: MCQ English to Arabic - Multiple choice memorization
    /// Given English translation, select Arabic word
    McqEnToAr {
        /// Word node ID
        node_id: i64,
        /// Indices of distractor words (to fetch from database)
        distractor_node_ids: Vec<i64>,
    },

    /// Exercise 3: Translation - Type English translation of Arabic word
    Translation {
        /// Word node ID
        node_id: i64,
    },

    /// Exercise 4: Contextual Translation - Translation with verse context
    ContextualTranslation {
        /// Word node ID
        node_id: i64,
        /// Verse key for context (e.g., "1:1")
        verse_key: String,
    },

    /// Exercise 5: Cloze Deletion - Fill in missing word in verse
    ClozeDeletion {
        /// Verse node ID
        node_id: i64,
        /// Position of the word to blank out (1-indexed)
        blank_position: i32,
    },

    /// Exercise 6: First Letter Hint - Memorization with first letter hint
    FirstLetterHint {
        /// Verse node ID
        node_id: i64,
        /// Position of the word to test (1-indexed)
        word_position: i32,
    },

    /// Exercise 7: Missing Word MCQ - MCQ for missing word in verse
    MissingWordMcq {
        /// Verse node ID
        node_id: i64,
        /// Position of the missing word (1-indexed)
        blank_position: i32,
        /// Node IDs of distractor words
        distractor_node_ids: Vec<i64>,
    },

    /// Exercise 8: Next Word MCQ - Predict next word in sequence
    NextWordMcq {
        /// Verse node ID
        node_id: i64,
        /// Position before the target word (1-indexed, target is position+1)
        context_position: i32,
        /// Node IDs of distractor words
        distractor_node_ids: Vec<i64>,
    },

    /// Exercise 9: Full Verse Input - Type entire verse from memory
    FullVerseInput {
        /// Verse node ID
        node_id: i64,
    },

    /// Exercise 10: Ayah Chain - Continuous verse typing (stateful)
    AyahChain {
        /// Chapter node ID or range
        node_id: i64,
        /// Verse keys in the chain (e.g., ["1:1", "1:2", "1:3"])
        verse_keys: Vec<String>,
        /// Current index in the chain (0-indexed)
        current_index: usize,
        /// Number of completed verses
        completed_count: usize,
    },

    /// Exercise 11: Find the Mistake - Identify incorrect word in modified verse
    FindMistake {
        /// Verse node ID
        node_id: i64,
        /// Position of the mistake (1-indexed)
        mistake_position: i32,
        /// Node ID of the correct word at that position
        correct_word_node_id: i64,
        /// Node ID of the incorrect (substituted) word
        incorrect_word_node_id: i64,
    },

    /// Exercise 12: Ayah Sequence - Put verses/words in correct order
    AyahSequence {
        /// Base node ID (verse or chapter)
        node_id: i64,
        /// Node IDs in correct order (for validation)
        correct_sequence: Vec<i64>,
    },

    /// Exercise 13: Identify Root - Grammar exercise for root identification
    IdentifyRoot {
        /// Word node ID
        node_id: i64,
        /// Root letters (correct answer key)
        root: String,
    },

    /// Exercise 14: Reverse Cloze - Given translation, recall Arabic with blank
    ReverseCloze {
        /// Verse node ID
        node_id: i64,
        /// Position of the word to blank out (1-indexed)
        blank_position: i32,
    },

    /// Exercise 15: Translate Phrase - Type English translation of verse/phrase
    TranslatePhrase {
        /// Node ID (verse or word)
        node_id: i64,
        /// Translator ID for fetching correct translation
        translator_id: i32,
    },

    /// Exercise 16: Part of Speech Tagging - Identify grammatical category
    PosTagging {
        /// Word node ID
        node_id: i64,
        /// Correct POS tag (e.g., "noun", "verb", "particle")
        correct_pos: String,
        /// Options for MCQ
        options: Vec<String>,
    },

    /// Exercise 17: Cross-Verse Connection - Identify thematic connections
    CrossVerseConnection {
        /// Primary verse node ID
        node_id: i64,
        /// Related verse node IDs (correct answers)
        related_verse_ids: Vec<i64>,
        /// Theme/connection type
        connection_theme: String,
    },
}

impl ExerciseData {
    /// Get the node ID for this exercise
    pub fn node_id(&self) -> i64 {
        match self {
            Self::Memorization { node_id }
            | Self::McqArToEn { node_id, .. }
            | Self::McqEnToAr { node_id, .. }
            | Self::Translation { node_id }
            | Self::ContextualTranslation { node_id, .. }
            | Self::ClozeDeletion { node_id, .. }
            | Self::FirstLetterHint { node_id, .. }
            | Self::MissingWordMcq { node_id, .. }
            | Self::NextWordMcq { node_id, .. }
            | Self::FullVerseInput { node_id }
            | Self::AyahChain { node_id, .. }
            | Self::FindMistake { node_id, .. }
            | Self::AyahSequence { node_id, .. }
            | Self::IdentifyRoot { node_id, .. }
            | Self::ReverseCloze { node_id, .. }
            | Self::TranslatePhrase { node_id, .. }
            | Self::PosTagging { node_id, .. }
            | Self::CrossVerseConnection { node_id, .. } => *node_id,
        }
    }

    /// Get the exercise type name
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Memorization { .. } => "memorization",
            Self::McqArToEn { .. } => "mcq_ar_to_en",
            Self::McqEnToAr { .. } => "mcq_en_to_ar",
            Self::Translation { .. } => "translation",
            Self::ContextualTranslation { .. } => "contextual_translation",
            Self::ClozeDeletion { .. } => "cloze_deletion",
            Self::FirstLetterHint { .. } => "first_letter_hint",
            Self::MissingWordMcq { .. } => "missing_word_mcq",
            Self::NextWordMcq { .. } => "next_word_mcq",
            Self::FullVerseInput { .. } => "full_verse_input",
            Self::AyahChain { .. } => "ayah_chain",
            Self::FindMistake { .. } => "find_mistake",
            Self::AyahSequence { .. } => "ayah_sequence",
            Self::IdentifyRoot { .. } => "identify_root",
            Self::ReverseCloze { .. } => "reverse_cloze",
            Self::TranslatePhrase { .. } => "translate_phrase",
            Self::PosTagging { .. } => "pos_tagging",
            Self::CrossVerseConnection { .. } => "cross_verse_connection",
        }
    }

    /// Check if this exercise type is stateful (requires maintaining state between submissions)
    pub fn is_stateful(&self) -> bool {
        matches!(self, Self::AyahChain { .. })
    }

    /// Check if this exercise type is an MCQ (has predefined options)
    pub fn is_mcq(&self) -> bool {
        matches!(
            self,
            Self::McqArToEn { .. }
                | Self::McqEnToAr { .. }
                | Self::MissingWordMcq { .. }
                | Self::NextWordMcq { .. }
                | Self::PosTagging { .. }
        )
    }

    /// Check if this exercise requires Arabic text input
    pub fn requires_arabic_input(&self) -> bool {
        matches!(
            self,
            Self::Memorization { .. }
                | Self::McqEnToAr { .. }
                | Self::ClozeDeletion { .. }
                | Self::FirstLetterHint { .. }
                | Self::FullVerseInput { .. }
                | Self::AyahChain { .. }
                | Self::ReverseCloze { .. }
        )
    }

    /// Check if this exercise requires English text input
    pub fn requires_english_input(&self) -> bool {
        matches!(
            self,
            Self::Translation { .. }
                | Self::ContextualTranslation { .. }
                | Self::TranslatePhrase { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memorization_node_id() {
        let exercise = ExerciseData::Memorization { node_id: 1 };
        assert_eq!(exercise.node_id(), 1);
        assert_eq!(exercise.type_name(), "memorization");
        assert!(!exercise.is_stateful());
        assert!(exercise.requires_arabic_input());
    }

    #[test]
    fn test_mcq_ar_to_en() {
        let exercise = ExerciseData::McqArToEn {
            node_id: 1,
            distractor_node_ids: vec![2, 3],
        };
        assert_eq!(exercise.type_name(), "mcq_ar_to_en");
        assert!(exercise.is_mcq());
        assert!(!exercise.requires_arabic_input());
    }

    #[test]
    fn test_ayah_chain_stateful() {
        let exercise = ExerciseData::AyahChain {
            node_id: 1,
            verse_keys: vec!["1:1".to_string(), "1:2".to_string()],
            current_index: 0,
            completed_count: 0,
        };
        assert!(exercise.is_stateful());
        assert_eq!(exercise.type_name(), "ayah_chain");
    }

    #[test]
    fn test_serialization_deserialization() {
        let exercise = ExerciseData::FindMistake {
            node_id: 1,
            mistake_position: 3,
            correct_word_node_id: 3,
            incorrect_word_node_id: 5,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&exercise).expect("Failed to serialize");

        // Deserialize back
        let deserialized: ExerciseData =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(exercise, deserialized);
    }

    #[test]
    fn test_pos_tagging_mcq() {
        let exercise = ExerciseData::PosTagging {
            node_id: 1,
            correct_pos: "noun".to_string(),
            options: vec![
                "noun".to_string(),
                "verb".to_string(),
                "particle".to_string(),
            ],
        };
        assert!(exercise.is_mcq());
        assert_eq!(exercise.type_name(), "pos_tagging");
    }

    #[test]
    fn test_translate_phrase_english_input() {
        let exercise = ExerciseData::TranslatePhrase {
            node_id: 1,
            translator_id: 131,
        };
        assert!(exercise.requires_english_input());
        assert!(!exercise.requires_arabic_input());
    }

    #[test]
    fn test_cross_verse_connection() {
        let exercise = ExerciseData::CrossVerseConnection {
            node_id: 1,
            related_verse_ids: vec![2, 112],
            connection_theme: "names_of_allah".to_string(),
        };
        assert_eq!(exercise.type_name(), "cross_verse_connection");
        assert_eq!(exercise.node_id(), 1);
    }
}
