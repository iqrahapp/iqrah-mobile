// exercises/types.rs
// Exercise trait and common types

use serde::{Deserialize, Serialize};

/// Trait for all exercise types
/// Each exercise can generate questions, check answers, and provide hints
pub trait Exercise: Send + Sync {
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
}

/// Enum for different exercise types (Phase 4.3)
pub enum ExerciseType {
    Memorization(Box<dyn Exercise>),
    Translation(Box<dyn Exercise>),
}

impl ExerciseType {
    /// Get the underlying exercise trait object
    pub fn as_exercise(&self) -> &dyn Exercise {
        match self {
            ExerciseType::Memorization(ex) => ex.as_ref(),
            ExerciseType::Translation(ex) => ex.as_ref(),
        }
    }

    /// Get the exercise type name
    pub fn get_type_name(&self) -> &'static str {
        match self {
            ExerciseType::Memorization(_) => "memorization",
            ExerciseType::Translation(_) => "translation",
        }
    }
}
