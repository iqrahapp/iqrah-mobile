// exercises/types.rs
// Exercise trait and common types

use serde::{Deserialize, Serialize};

/// Trait for all exercise types
/// Each exercise can generate questions, check answers, and provide hints
pub trait Exercise: Send + Sync + std::any::Any {
    /// Generate the question/prompt for this exercise
    fn generate_question(&self) -> String;

    /// Check if the provided answer is correct
    fn check_answer(&self, answer: &str) -> bool;

    /// Get a hint for this exercise (optional)
    fn get_hint(&self) -> Option<String>;

    /// Get the node ID this exercise is for
    fn get_node_id(&self) -> &str;

    /// Get the exercise type name
    fn get_type_name(&self) -> &'static str;
}

/// Response after checking an exercise answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseResponse {
    pub is_correct: bool,
    pub correct_answer: Option<String>,
    pub hint: Option<String>,
    /// For MCQ exercises, include the options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    /// Semantic grading label (Excellent/Partial/Incorrect) - only present for semantic grading
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_grade: Option<String>,
    /// Semantic similarity score (0.0 to 1.0) - only present for semantic grading
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_score: Option<f32>,
}

/// Enum for different exercise types (Phase 4.3)
pub enum ExerciseType {
    Memorization(Box<dyn Exercise>),
    Translation(Box<dyn Exercise>),
    McqArToEn(Box<dyn Exercise>),
    McqEnToAr(Box<dyn Exercise>),
}

impl ExerciseType {
    /// Get the underlying exercise trait object
    pub fn as_exercise(&self) -> &dyn Exercise {
        match self {
            ExerciseType::Memorization(ex) => ex.as_ref(),
            ExerciseType::Translation(ex) => ex.as_ref(),
            ExerciseType::McqArToEn(ex) => ex.as_ref(),
            ExerciseType::McqEnToAr(ex) => ex.as_ref(),
        }
    }

    /// Get the exercise type name
    pub fn get_type_name(&self) -> &'static str {
        match self {
            ExerciseType::Memorization(_) => "memorization",
            ExerciseType::Translation(_) => "translation",
            ExerciseType::McqArToEn(_) => "mcq_ar_to_en",
            ExerciseType::McqEnToAr(_) => "mcq_en_to_ar",
        }
    }
}
