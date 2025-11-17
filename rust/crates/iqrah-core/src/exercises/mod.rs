// exercises/mod.rs
// Phase 4.3: Axis-Specific Exercise Generation

mod ayah_sequence;
mod grammar;
mod graph;
mod mcq;
mod memorization;
mod reverse_cloze;
mod service;
mod translation;
mod types;

pub use ayah_sequence::AyahSequenceExercise;
pub use grammar::IdentifyRootExercise;
pub use graph::CrossVerseConnectionExercise;
pub use mcq::{McqExercise, McqType};
pub use memorization::{
    ClozeDeletionExercise, FirstLetterHintExercise, MemorizationExercise, MissingWordMcqExercise,
    NextWordDifficulty, NextWordMcqExercise,
};
pub use reverse_cloze::ReverseClozeExercise;
pub use service::ExerciseService;
pub use translation::{ContextualTranslationExercise, TranslationExercise};
pub use types::{Exercise, ExerciseResponse, ExerciseType};
