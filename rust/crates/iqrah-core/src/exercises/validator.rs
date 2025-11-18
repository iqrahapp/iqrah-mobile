// exercises/validator.rs
// Answer validation logic for enum-based exercises

use super::exercise_data::ExerciseData;
use crate::semantic::grader::{SemanticGradeLabel, SemanticGrader, SEMANTIC_EMBEDDER};
use crate::ContentRepository;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Answer input from the user
///
/// Different exercise types accept different input formats:
/// - Text: Free-form text (Arabic or English)
/// - Position: Numeric position (1-indexed)
/// - WordId: Node ID of a selected word
/// - VerseKey: Verse reference (e.g., "1:1")
/// - OptionIndex: Index of selected MCQ option (0-indexed)
/// - Sequence: Ordered list of node IDs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnswerInput {
    /// Free-form text answer (for memorization, translation, etc.)
    Text { value: String },
    /// Numeric position (for find_mistake, word selection)
    Position { value: i32 },
    /// Word/node selection by ID
    WordId { value: String },
    /// Verse reference (for sequencing, connections)
    VerseKey { value: String },
    /// MCQ option index (0-indexed)
    OptionIndex { value: usize },
    /// Ordered sequence of IDs (for ayah_sequence)
    Sequence { values: Vec<String> },
}

/// Result of answer validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the answer is correct
    pub is_correct: bool,
    /// Optional feedback message
    pub feedback: Option<String>,
    /// For partial credit: similarity score (0.0 to 1.0)
    pub similarity_score: Option<f32>,
    /// Semantic grading label (Excellent/Partial/Incorrect)
    pub semantic_grade: Option<String>,
    /// The correct answer (if wrong or for review)
    pub correct_answer: Option<String>,
}

/// Answer keys for validation
///
/// Stores the expected correct answer(s) for an exercise
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnswerKeys {
    /// Single text answer (exact match after normalization)
    Text { value: String },
    /// Single position (numeric)
    Position { value: i32 },
    /// Single word/node ID
    WordId { value: String },
    /// Ordered sequence of IDs
    Sequence { values: Vec<String> },
    /// Root letters for grammar exercises
    Root { letters: String },
    /// Part of speech tag
    PosTag { value: String },
}

/// Trait for validating answers to exercises
///
/// This trait provides the core validation logic for checking user answers
/// against the correct answer keys, with support for different input types.
pub trait ExerciseValidator {
    /// Validate an answer input against the exercise data
    ///
    /// Fetches necessary content from the repository and checks if the answer is correct.
    /// Returns a ValidationResult with feedback and scoring.
    fn validate(
        &self,
        exercise: &ExerciseData,
        answer: &AnswerInput,
        content_repo: &dyn ContentRepository,
    ) -> Result<ValidationResult>;

    /// Get answer keys for an exercise (what the correct answer is)
    fn get_answer_keys(
        &self,
        exercise: &ExerciseData,
        content_repo: &dyn ContentRepository,
    ) -> Result<AnswerKeys>;
}

/// Default validator implementation
pub struct DefaultValidator;

impl DefaultValidator {
    /// Create a new default validator
    pub fn new() -> Self {
        Self
    }

    /// Normalize Arabic text for comparison (remove diacritics)
    pub fn normalize_arabic(text: &str) -> String {
        let normalized = text
            .chars()
            // Remove diacritical marks
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
                'ٱ' => 'ا', // Alif with wasla
                'أ' => 'ا', // Alif with hamza above
                'إ' => 'ا', // Alif with hamza below
                'آ' => 'ا', // Alif with madda
                'ٰ' => 'ا',  // Alif khanjariyyah
                'ى' => 'ي', // Alif maqsurah
                'ة' => 'ه', // Ta marbuta
                'ۀ' => 'ه', // Hamza on Ha
                _ => c,
            })
            .collect::<String>();

        // Normalize whitespace
        normalized
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Normalize English text for comparison
    pub fn normalize_english(text: &str) -> String {
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

    /// Normalize POS tag for comparison
    pub fn normalize_pos(text: &str) -> String {
        text.trim().to_lowercase()
    }

    /// Perform semantic similarity grading (for Arabic text)
    pub fn semantic_grade(user_text: &str, correct_text: &str) -> Result<ValidationResult> {
        // Get embedder
        let embedder = SEMANTIC_EMBEDDER.get().ok_or_else(|| {
            anyhow::anyhow!(
                "Semantic embedder not initialized. Call ExerciseService::init_semantic_model()"
            )
        })?;

        let grader = SemanticGrader::new(embedder);
        let grade = grader.grade_answer(user_text, correct_text)?;

        Ok(ValidationResult {
            is_correct: grade.label == SemanticGradeLabel::Excellent,
            feedback: Some(format!("Similarity: {:.1}%", grade.similarity * 100.0)),
            similarity_score: Some(grade.similarity),
            semantic_grade: Some(format!("{:?}", grade.label)),
            correct_answer: if grade.label != SemanticGradeLabel::Excellent {
                Some(correct_text.to_string())
            } else {
                None
            },
        })
    }
}

impl Default for DefaultValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ExerciseValidator for DefaultValidator {
    fn validate(
        &self,
        exercise: &ExerciseData,
        answer: &AnswerInput,
        content_repo: &dyn ContentRepository,
    ) -> Result<ValidationResult> {
        match exercise {
            ExerciseData::Memorization { node_id } => {
                let AnswerInput::Text { value: user_text } = answer else {
                    return Ok(ValidationResult {
                        is_correct: false,
                        feedback: Some("Expected text input".to_string()),
                        similarity_score: None,
                        semantic_grade: None,
                        correct_answer: None,
                    });
                };

                // Fetch correct text - this is async, so we'd need to adjust the trait
                // For now, we'll return an error indicating async fetch is needed
                // In a real implementation, this would be async
                let _ = (node_id, content_repo, user_text);
                Err(anyhow::anyhow!(
                    "Memorization validation requires async content fetching - use ExerciseService"
                ))
            }

            ExerciseData::FindMistake {
                mistake_position, ..
            } => {
                let AnswerInput::Position { value: user_pos } = answer else {
                    return Ok(ValidationResult {
                        is_correct: false,
                        feedback: Some("Expected position input".to_string()),
                        similarity_score: None,
                        semantic_grade: None,
                        correct_answer: Some(mistake_position.to_string()),
                    });
                };

                let is_correct = user_pos == mistake_position;
                Ok(ValidationResult {
                    is_correct,
                    feedback: if is_correct {
                        Some("Correct!".to_string())
                    } else {
                        Some(format!("The mistake is at position {}", mistake_position))
                    },
                    similarity_score: None,
                    semantic_grade: None,
                    correct_answer: if is_correct {
                        None
                    } else {
                        Some(mistake_position.to_string())
                    },
                })
            }

            ExerciseData::IdentifyRoot { root, .. } => {
                let AnswerInput::Text { value: user_text } = answer else {
                    return Ok(ValidationResult {
                        is_correct: false,
                        feedback: Some("Expected text input".to_string()),
                        similarity_score: None,
                        semantic_grade: None,
                        correct_answer: Some(root.clone()),
                    });
                };

                let normalized_user = Self::normalize_arabic(user_text);
                let normalized_root = Self::normalize_arabic(root);
                let is_correct = normalized_user == normalized_root;

                Ok(ValidationResult {
                    is_correct,
                    feedback: if is_correct {
                        Some("Correct!".to_string())
                    } else {
                        Some(format!("The root is: {}", root))
                    },
                    similarity_score: None,
                    semantic_grade: None,
                    correct_answer: if is_correct { None } else { Some(root.clone()) },
                })
            }

            ExerciseData::PosTagging { correct_pos, .. } => {
                let AnswerInput::Text { value: user_text } = answer else {
                    return Ok(ValidationResult {
                        is_correct: false,
                        feedback: Some("Expected text input".to_string()),
                        similarity_score: None,
                        semantic_grade: None,
                        correct_answer: Some(correct_pos.clone()),
                    });
                };

                let normalized_user = Self::normalize_pos(user_text);
                let normalized_correct = Self::normalize_pos(correct_pos);
                let is_correct = normalized_user == normalized_correct;

                Ok(ValidationResult {
                    is_correct,
                    feedback: if is_correct {
                        Some("Correct!".to_string())
                    } else {
                        Some(format!("The correct part of speech is: {}", correct_pos))
                    },
                    similarity_score: None,
                    semantic_grade: None,
                    correct_answer: if is_correct {
                        None
                    } else {
                        Some(correct_pos.clone())
                    },
                })
            }

            ExerciseData::AyahSequence {
                correct_sequence, ..
            } => {
                let AnswerInput::Sequence { values: user_seq } = answer else {
                    return Ok(ValidationResult {
                        is_correct: false,
                        feedback: Some("Expected sequence input".to_string()),
                        similarity_score: None,
                        semantic_grade: None,
                        correct_answer: None,
                    });
                };

                let is_correct = user_seq == correct_sequence;
                Ok(ValidationResult {
                    is_correct,
                    feedback: if is_correct {
                        Some("Correct sequence!".to_string())
                    } else {
                        Some("Incorrect order".to_string())
                    },
                    similarity_score: None,
                    semantic_grade: None,
                    correct_answer: if is_correct {
                        None
                    } else {
                        Some(correct_sequence.join(", "))
                    },
                })
            }

            // Other exercise types require async content fetching
            _ => Err(anyhow::anyhow!(
                "Exercise type {} requires async validation - use ExerciseService",
                exercise.type_name()
            )),
        }
    }

    fn get_answer_keys(
        &self,
        exercise: &ExerciseData,
        _content_repo: &dyn ContentRepository,
    ) -> Result<AnswerKeys> {
        match exercise {
            ExerciseData::FindMistake {
                mistake_position, ..
            } => Ok(AnswerKeys::Position {
                value: *mistake_position,
            }),

            ExerciseData::IdentifyRoot { root, .. } => Ok(AnswerKeys::Root {
                letters: root.clone(),
            }),

            ExerciseData::PosTagging { correct_pos, .. } => Ok(AnswerKeys::PosTag {
                value: correct_pos.clone(),
            }),

            ExerciseData::AyahSequence {
                correct_sequence, ..
            } => Ok(AnswerKeys::Sequence {
                values: correct_sequence.clone(),
            }),

            // Other exercise types require async content fetching
            _ => Err(anyhow::anyhow!(
                "Exercise type {} requires async answer key retrieval",
                exercise.type_name()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_arabic() {
        let text = "بِسْمِ ٱللَّهِ";
        let normalized = DefaultValidator::normalize_arabic(text);
        // Should remove diacritics and normalize letters
        assert!(!normalized.contains('\u{064E}')); // No Fatha
        assert!(!normalized.contains('\u{0650}')); // No Kasra
    }

    #[test]
    fn test_normalize_english() {
        let text = "In the name of Allah, the Most Gracious!";
        let normalized = DefaultValidator::normalize_english(text);
        assert_eq!(
            normalized,
            "in the name of allah the most gracious".to_string()
        );
    }

    #[test]
    fn test_normalize_pos() {
        assert_eq!(DefaultValidator::normalize_pos("  NOUN  "), "noun");
        assert_eq!(DefaultValidator::normalize_pos("Verb"), "verb");
    }

    // Note: Validation tests that require ContentRepository are integration tests
    // and should be in a separate test file with database setup.
    // The validator logic is tested through the ExerciseService in integration tests.

    #[test]
    fn test_answer_input_serialization() {
        let answer = AnswerInput::Text {
            value: "test".to_string(),
        };
        let json = serde_json::to_string(&answer).unwrap();
        let deserialized: AnswerInput = serde_json::from_str(&json).unwrap();
        assert_eq!(answer, deserialized);
    }

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            is_correct: true,
            feedback: Some("Great!".to_string()),
            similarity_score: Some(0.95),
            semantic_grade: Some("Excellent".to_string()),
            correct_answer: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let _deserialized: ValidationResult = serde_json::from_str(&json).unwrap();
    }
}
