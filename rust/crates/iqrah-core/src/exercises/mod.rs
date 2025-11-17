// exercises/mod.rs
// Phase 4.3: Axis-Specific Exercise Generation

mod mcq;
mod memorization;
mod service;
mod translation;
mod types;

pub use mcq::{McqExercise, McqType};
pub use memorization::{
    ClozeDeletionExercise, FirstLetterHintExercise, MemorizationExercise, MissingWordMcqExercise,
    NextWordDifficulty, NextWordMcqExercise,
};
pub use service::ExerciseService;
pub use translation::TranslationExercise;
pub use types::{Exercise, ExerciseResponse, ExerciseType};
