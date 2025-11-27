// exercises/enum_tests.rs
// Comprehensive tests for enum-based exercise architecture

#[cfg(test)]
mod tests {
    use crate::exercises::{AnswerInput, ExerciseData, ValidationResult};
    use serde_json;

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
    fn test_identify_root_variant() {
        let exercise = ExerciseData::IdentifyRoot {
            node_id: 1,
            root: "كتب".to_string(),
        };

        assert_eq!(exercise.type_name(), "identify_root");
        // IdentifyRoot stores answer as text (Arabic root letters)
        // but in practice it could be MCQ or text input depending on implementation
        assert!(!exercise.is_mcq()); // Not currently marked as MCQ in the enum
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
        assert_eq!(original_len, 18, "Expected 18 exercise types");
    }

    #[test]
    fn test_stateful_exercises_identified_correctly() {
        let ayah_chain = ExerciseData::AyahChain {
            node_id: 1,
            verse_keys: vec![],
            current_index: 0,
            completed_count: 0,
        };

        let memorization = ExerciseData::Memorization { node_id: 1 };

        assert!(ayah_chain.is_stateful());
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
