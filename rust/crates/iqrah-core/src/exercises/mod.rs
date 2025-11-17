// exercises/mod.rs
// Phase 4.3: Axis-Specific Exercise Generation

mod memorization;
mod service;
mod translation;
mod types;

pub use memorization::MemorizationExercise;
pub use service::ExerciseService;
pub use translation::TranslationExercise;
pub use types::{Exercise, ExerciseResponse, ExerciseType};
