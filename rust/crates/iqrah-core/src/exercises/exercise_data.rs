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

    /// Exercise 13: Sequence Recall - Select the correct continuation sequence
    SequenceRecall {
        /// Verse node ID used as prompt context
        node_id: i64,
        /// Correct sequence of verse IDs
        correct_sequence: Vec<i64>,
        /// Multiple choice options (each is a sequence of verse IDs)
        options: Vec<Vec<i64>>,
    },

    /// Exercise 14: First Word Recall - Type the first word of a verse
    FirstWordRecall {
        /// Verse node ID
        node_id: i64,
        /// Verse key (e.g., "1:1")
        verse_key: String,
    },

    /// Exercise 15: Identify Root - Grammar exercise for root identification
    IdentifyRoot {
        /// Word node ID
        node_id: i64,
        /// Root letters (correct answer key)
        root: String,
    },

    /// Exercise 16: Reverse Cloze - Given translation, recall Arabic with blank
    ReverseCloze {
        /// Verse node ID
        node_id: i64,
        /// Position of the word to blank out (1-indexed)
        blank_position: i32,
    },

    /// Exercise 17: Translate Phrase - Type English translation of verse/phrase
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
        /// Related verse node IDs (first is correct, rest are distractors)
        related_verse_ids: Vec<i64>,
        /// Theme/connection type
        connection_theme: String,
    },

    /// Exercise 18: Echo Recall - Progressive blur memorization
    /// A stateful exercise that displays words with varying visibility
    /// based on energy levels. User taps through words, timing is tracked.
    EchoRecall {
        /// Node IDs of ayahs to practice (verse nodes)
        ayah_node_ids: Vec<i64>,
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
            | Self::SequenceRecall { node_id, .. }
            | Self::FirstWordRecall { node_id, .. }
            | Self::IdentifyRoot { node_id, .. }
            | Self::ReverseCloze { node_id, .. }
            | Self::TranslatePhrase { node_id, .. }
            | Self::PosTagging { node_id, .. }
            | Self::CrossVerseConnection { node_id, .. } => *node_id,
            // EchoRecall uses first ayah as representative node for scheduling
            Self::EchoRecall { ayah_node_ids } => ayah_node_ids.first().copied().unwrap_or(0),
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
            Self::SequenceRecall { .. } => "sequence_recall",
            Self::FirstWordRecall { .. } => "first_word_recall",
            Self::IdentifyRoot { .. } => "identify_root",
            Self::ReverseCloze { .. } => "reverse_cloze",
            Self::TranslatePhrase { .. } => "translate_phrase",
            Self::PosTagging { .. } => "pos_tagging",
            Self::CrossVerseConnection { .. } => "cross_verse_connection",
            Self::EchoRecall { .. } => "echo_recall",
        }
    }

    /// Check if this exercise type is stateful (requires maintaining state between submissions)
    pub fn is_stateful(&self) -> bool {
        matches!(self, Self::AyahChain { .. } | Self::EchoRecall { .. })
    }

    /// Check if this exercise type is an MCQ (has predefined options)
    pub fn is_mcq(&self) -> bool {
        matches!(
            self,
            Self::McqArToEn { .. }
                | Self::McqEnToAr { .. }
                | Self::MissingWordMcq { .. }
                | Self::NextWordMcq { .. }
                | Self::SequenceRecall { .. }
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
                | Self::FirstWordRecall { .. }
                | Self::FullVerseInput { .. }
                | Self::AyahChain { .. }
                | Self::ReverseCloze { .. }
                | Self::EchoRecall { .. }
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
    use crate::exercises::{AnswerInput, ValidationResult};

    // ========================================================================
    // ExerciseData Enum Tests
    // ========================================================================

    #[test]
    fn test_memorization_variant() {
        let exercise = ExerciseData::Memorization { node_id: 1 };

        assert_eq!(exercise.node_id(), 1);
        assert_eq!(exercise.type_name(), "memorization");
        assert!(!exercise.is_stateful());
        assert!(exercise.requires_arabic_input());
        assert!(!exercise.requires_english_input());
        assert!(!exercise.is_mcq());
    }

    #[test]
    fn test_mcq_ar_to_en_variant() {
        let exercise = ExerciseData::McqArToEn {
            node_id: 1,
            distractor_node_ids: vec![2, 3, 4],
        };

        assert_eq!(exercise.type_name(), "mcq_ar_to_en");
        assert!(exercise.is_mcq());
        assert!(!exercise.is_stateful());
        assert!(!exercise.requires_arabic_input());
    }

    #[test]
    fn test_mcq_en_to_ar_variant() {
        let exercise = ExerciseData::McqEnToAr {
            node_id: 1,
            distractor_node_ids: vec![2, 3],
        };

        assert_eq!(exercise.type_name(), "mcq_en_to_ar");
        assert!(exercise.is_mcq());
        assert!(exercise.requires_arabic_input());
    }

    #[test]
    fn test_translation_variant() {
        let exercise = ExerciseData::Translation { node_id: 1 };

        assert_eq!(exercise.type_name(), "translation");
        assert!(exercise.requires_english_input());
        assert!(!exercise.requires_arabic_input());
    }

    #[test]
    fn test_contextual_translation_variant() {
        let exercise = ExerciseData::ContextualTranslation {
            node_id: 1,
            verse_key: "1:1".to_string(),
        };

        assert_eq!(exercise.type_name(), "contextual_translation");
        assert!(exercise.requires_english_input());
    }

    #[test]
    fn test_cloze_deletion_variant() {
        let exercise = ExerciseData::ClozeDeletion {
            node_id: 1,
            blank_position: 3,
        };

        assert_eq!(exercise.type_name(), "cloze_deletion");
        assert!(exercise.requires_arabic_input());
    }

    #[test]
    fn test_first_letter_hint_variant() {
        let exercise = ExerciseData::FirstLetterHint {
            node_id: 1,
            word_position: 2,
        };

        assert_eq!(exercise.type_name(), "first_letter_hint");
        assert!(exercise.requires_arabic_input());
    }

    #[test]
    fn test_missing_word_mcq_variant() {
        let exercise = ExerciseData::MissingWordMcq {
            node_id: 1,
            blank_position: 3,
            distractor_node_ids: vec![2, 3],
        };

        assert_eq!(exercise.type_name(), "missing_word_mcq");
        assert!(exercise.is_mcq());
    }

    #[test]
    fn test_next_word_mcq_variant() {
        let exercise = ExerciseData::NextWordMcq {
            node_id: 1,
            context_position: 2,
            distractor_node_ids: vec![2],
        };

        assert_eq!(exercise.type_name(), "next_word_mcq");
        assert!(exercise.is_mcq());
    }

    #[test]
    fn test_full_verse_input_variant() {
        let exercise = ExerciseData::FullVerseInput { node_id: 1 };

        assert_eq!(exercise.type_name(), "full_verse_input");
        assert!(exercise.requires_arabic_input());
    }

    #[test]
    fn test_ayah_chain_variant() {
        let exercise = ExerciseData::AyahChain {
            node_id: 1,
            verse_keys: vec!["1:1".to_string(), "1:2".to_string(), "1:3".to_string()],
            current_index: 0,
            completed_count: 0,
        };

        assert_eq!(exercise.type_name(), "ayah_chain");
        assert!(exercise.is_stateful());
        assert!(exercise.requires_arabic_input());
    }

    #[test]
    fn test_find_mistake_variant() {
        let exercise = ExerciseData::FindMistake {
            node_id: 1,
            mistake_position: 3,
            correct_word_node_id: 2,
            incorrect_word_node_id: 3,
        };

        assert_eq!(exercise.type_name(), "find_mistake");
        assert!(!exercise.requires_arabic_input());
        assert!(!exercise.requires_english_input());
    }

    #[test]
    fn test_ayah_sequence_variant() {
        let exercise = ExerciseData::AyahSequence {
            node_id: 1,
            correct_sequence: vec![2, 3, 4],
        };

        assert_eq!(exercise.type_name(), "ayah_sequence");
    }

    #[test]
    fn test_sequence_recall_variant() {
        let exercise = ExerciseData::SequenceRecall {
            node_id: 1,
            correct_sequence: vec![2, 3],
            options: vec![vec![2, 3], vec![4, 5]],
        };

        assert_eq!(exercise.type_name(), "sequence_recall");
        assert!(exercise.is_mcq());
    }

    #[test]
    fn test_first_word_recall_variant() {
        let exercise = ExerciseData::FirstWordRecall {
            node_id: 1,
            verse_key: "1:1".to_string(),
        };

        assert_eq!(exercise.type_name(), "first_word_recall");
        assert!(exercise.requires_arabic_input());
    }

    #[test]
    fn test_identify_root_variant() {
        let exercise = ExerciseData::IdentifyRoot {
            node_id: 1,
            root: "كتب".to_string(),
        };

        assert_eq!(exercise.type_name(), "identify_root");
        assert!(!exercise.is_mcq());
    }

    #[test]
    fn test_reverse_cloze_variant() {
        let exercise = ExerciseData::ReverseCloze {
            node_id: 1,
            blank_position: 3,
        };

        assert_eq!(exercise.type_name(), "reverse_cloze");
        assert!(exercise.requires_arabic_input());
    }

    #[test]
    fn test_translate_phrase_variant() {
        let exercise = ExerciseData::TranslatePhrase {
            node_id: 1,
            translator_id: 131,
        };

        assert_eq!(exercise.type_name(), "translate_phrase");
        assert!(exercise.requires_english_input());
    }

    #[test]
    fn test_pos_tagging_variant() {
        let exercise = ExerciseData::PosTagging {
            node_id: 1,
            correct_pos: "noun".to_string(),
            options: vec![
                "noun".to_string(),
                "verb".to_string(),
                "particle".to_string(),
            ],
        };

        assert_eq!(exercise.type_name(), "pos_tagging");
        assert!(exercise.is_mcq());
    }

    #[test]
    fn test_cross_verse_connection_variant() {
        let exercise = ExerciseData::CrossVerseConnection {
            node_id: 1,
            related_verse_ids: vec![2, 3],
            connection_theme: "names_of_allah".to_string(),
        };

        assert_eq!(exercise.type_name(), "cross_verse_connection");
    }

    // ========================================================================
    // Serialization Tests
    // ========================================================================

    #[test]
    fn test_memorization_serialization() {
        let exercise = ExerciseData::Memorization { node_id: 1 };

        let json = serde_json::to_string(&exercise).unwrap();
        let deserialized: ExerciseData = serde_json::from_str(&json).unwrap();

        assert_eq!(exercise, deserialized);
    }

    #[test]
    fn test_mcq_ar_to_en_serialization() {
        let exercise = ExerciseData::McqArToEn {
            node_id: 1,
            distractor_node_ids: vec![2, 3],
        };

        let json = serde_json::to_string(&exercise).unwrap();
        let deserialized: ExerciseData = serde_json::from_str(&json).unwrap();

        assert_eq!(exercise, deserialized);
    }

    #[test]
    fn test_ayah_chain_serialization() {
        let exercise = ExerciseData::AyahChain {
            node_id: 1,
            verse_keys: vec!["1:1".to_string(), "1:2".to_string()],
            current_index: 0,
            completed_count: 0,
        };

        let json = serde_json::to_string(&exercise).unwrap();
        let deserialized: ExerciseData = serde_json::from_str(&json).unwrap();

        assert_eq!(exercise, deserialized);

        // Verify JSON structure contains type tag
        assert!(json.contains("\"type\":\"ayah_chain\""));
    }

    #[test]
    fn test_find_mistake_serialization() {
        let exercise = ExerciseData::FindMistake {
            node_id: 1,
            mistake_position: 3,
            correct_word_node_id: 2,
            incorrect_word_node_id: 3,
        };

        let json = serde_json::to_string(&exercise).unwrap();
        let deserialized: ExerciseData = serde_json::from_str(&json).unwrap();

        assert_eq!(exercise, deserialized);
    }

    #[test]
    fn test_pos_tagging_serialization() {
        let exercise = ExerciseData::PosTagging {
            node_id: 1,
            correct_pos: "noun".to_string(),
            options: vec!["noun".to_string(), "verb".to_string()],
        };

        let json = serde_json::to_string(&exercise).unwrap();
        let deserialized: ExerciseData = serde_json::from_str(&json).unwrap();

        assert_eq!(exercise, deserialized);
    }

    // ========================================================================
    // AnswerInput Tests
    // ========================================================================

    #[test]
    fn test_answer_input_text() {
        let answer = AnswerInput::Text {
            value: "بِسْمِ".to_string(),
        };

        let json = serde_json::to_string(&answer).unwrap();
        let deserialized: AnswerInput = serde_json::from_str(&json).unwrap();

        assert_eq!(answer, deserialized);
    }

    #[test]
    fn test_answer_input_position() {
        let answer = AnswerInput::Position { value: 3 };

        let json = serde_json::to_string(&answer).unwrap();
        let deserialized: AnswerInput = serde_json::from_str(&json).unwrap();

        assert_eq!(answer, deserialized);
    }

    #[test]
    fn test_answer_input_word_id() {
        let answer = AnswerInput::WordId { value: 1 };

        let json = serde_json::to_string(&answer).unwrap();
        let deserialized: AnswerInput = serde_json::from_str(&json).unwrap();

        assert_eq!(answer, deserialized);
    }

    #[test]
    fn test_answer_input_verse_key() {
        let answer = AnswerInput::VerseKey {
            value: "1:1".to_string(),
        };

        let json = serde_json::to_string(&answer).unwrap();
        let deserialized: AnswerInput = serde_json::from_str(&json).unwrap();

        assert_eq!(answer, deserialized);
    }

    #[test]
    fn test_answer_input_option_index() {
        let answer = AnswerInput::OptionIndex { value: 2 };

        let json = serde_json::to_string(&answer).unwrap();
        let deserialized: AnswerInput = serde_json::from_str(&json).unwrap();

        assert_eq!(answer, deserialized);
    }

    #[test]
    fn test_answer_input_sequence() {
        let answer = AnswerInput::Sequence {
            values: vec![1, 2, 3],
        };

        let json = serde_json::to_string(&answer).unwrap();
        let deserialized: AnswerInput = serde_json::from_str(&json).unwrap();

        assert_eq!(answer, deserialized);
    }

    // ========================================================================
    // ValidationResult Tests
    // ========================================================================

    #[test]
    fn test_validation_result_correct() {
        let result = ValidationResult {
            is_correct: true,
            feedback: Some("Excellent!".to_string()),
            similarity_score: Some(0.98),
            semantic_grade: Some("Excellent".to_string()),
            correct_answer: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let _deserialized: ValidationResult = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_validation_result_incorrect() {
        let result = ValidationResult {
            is_correct: false,
            feedback: Some("Try again".to_string()),
            similarity_score: Some(0.45),
            semantic_grade: Some("Incorrect".to_string()),
            correct_answer: Some("بِسْمِ ٱللَّهِ".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        let _deserialized: ValidationResult = serde_json::from_str(&json).unwrap();
    }

    // ========================================================================
    // Edge Cases and Type Safety
    // ========================================================================

    #[test]
    fn test_all_exercise_types_have_unique_type_names() {
        let exercises = vec![
            ExerciseData::Memorization { node_id: 1 },
            ExerciseData::McqArToEn {
                node_id: 1,
                distractor_node_ids: vec![],
            },
            ExerciseData::McqEnToAr {
                node_id: 1,
                distractor_node_ids: vec![],
            },
            ExerciseData::Translation { node_id: 1 },
            ExerciseData::ContextualTranslation {
                node_id: 1,
                verse_key: "1:1".to_string(),
            },
            ExerciseData::ClozeDeletion {
                node_id: 1,
                blank_position: 1,
            },
            ExerciseData::FirstLetterHint {
                node_id: 1,
                word_position: 1,
            },
            ExerciseData::MissingWordMcq {
                node_id: 1,
                blank_position: 1,
                distractor_node_ids: vec![],
            },
            ExerciseData::NextWordMcq {
                node_id: 1,
                context_position: 1,
                distractor_node_ids: vec![],
            },
            ExerciseData::FullVerseInput { node_id: 1 },
            ExerciseData::AyahChain {
                node_id: 1,
                verse_keys: vec![],
                current_index: 0,
                completed_count: 0,
            },
            ExerciseData::FindMistake {
                node_id: 1,
                mistake_position: 1,
                correct_word_node_id: 2,
                incorrect_word_node_id: 3,
            },
            ExerciseData::AyahSequence {
                node_id: 1,
                correct_sequence: vec![],
            },
            ExerciseData::SequenceRecall {
                node_id: 1,
                correct_sequence: vec![],
                options: vec![],
            },
            ExerciseData::FirstWordRecall {
                node_id: 1,
                verse_key: "1:1".to_string(),
            },
            ExerciseData::IdentifyRoot {
                node_id: 1,
                root: "كتب".to_string(),
            },
            ExerciseData::ReverseCloze {
                node_id: 1,
                blank_position: 1,
            },
            ExerciseData::TranslatePhrase {
                node_id: 1,
                translator_id: 131,
            },
            ExerciseData::PosTagging {
                node_id: 1,
                correct_pos: "noun".to_string(),
                options: vec![],
            },
            ExerciseData::CrossVerseConnection {
                node_id: 1,
                related_verse_ids: vec![],
                connection_theme: "theme".to_string(),
            },
            ExerciseData::EchoRecall {
                ayah_node_ids: vec![1, 2, 3],
            },
        ];

        let mut type_names: Vec<&str> = exercises.iter().map(|e| e.type_name()).collect();
        let original_len = type_names.len();
        type_names.sort();
        type_names.dedup();

        assert_eq!(
            type_names.len(),
            original_len,
            "All exercise types should have unique type names"
        );
        assert_eq!(original_len, 21, "Expected 21 exercise types");
    }

    #[test]
    fn test_stateful_exercises_identified_correctly() {
        let ayah_chain = ExerciseData::AyahChain {
            node_id: 1,
            verse_keys: vec![],
            current_index: 0,
            completed_count: 0,
        };

        let echo_recall = ExerciseData::EchoRecall {
            ayah_node_ids: vec![1, 2, 3],
        };

        let memorization = ExerciseData::Memorization { node_id: 1 };

        assert!(ayah_chain.is_stateful());
        assert!(echo_recall.is_stateful());
        assert!(!memorization.is_stateful());
    }

    #[test]
    fn test_mcq_exercises_identified_correctly() {
        let mcq_exercises = vec![
            ExerciseData::McqArToEn {
                node_id: 1,
                distractor_node_ids: vec![],
            },
            ExerciseData::McqEnToAr {
                node_id: 1,
                distractor_node_ids: vec![],
            },
            ExerciseData::MissingWordMcq {
                node_id: 1,
                blank_position: 1,
                distractor_node_ids: vec![],
            },
            ExerciseData::NextWordMcq {
                node_id: 1,
                context_position: 1,
                distractor_node_ids: vec![],
            },
            ExerciseData::SequenceRecall {
                node_id: 1,
                correct_sequence: vec![],
                options: vec![],
            },
            ExerciseData::PosTagging {
                node_id: 1,
                correct_pos: "noun".to_string(),
                options: vec![],
            },
        ];

        for exercise in mcq_exercises {
            assert!(
                exercise.is_mcq(),
                "{} should be identified as MCQ",
                exercise.type_name()
            );
        }

        let non_mcq = ExerciseData::Memorization { node_id: 1 };
        assert!(!non_mcq.is_mcq());
    }
}
