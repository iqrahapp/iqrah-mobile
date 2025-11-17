// exercises/mod.rs
// Phase 4.3: Axis-Specific Exercise Generation

mod ayah_sequence;
mod full_verse_input;
mod grammar;
mod graph;
mod mcq;
mod memorization;
mod reverse_cloze;
mod service;
mod translate_phrase;
mod translation;
mod types;

pub use ayah_sequence::AyahSequenceExercise;
pub use full_verse_input::FullVerseInputExercise;
pub use grammar::IdentifyRootExercise;
pub use graph::CrossVerseConnectionExercise;
pub use mcq::{McqExercise, McqType};
pub use memorization::{
    ClozeDeletionExercise, FirstLetterHintExercise, MemorizationExercise, MissingWordMcqExercise,
    NextWordDifficulty, NextWordMcqExercise,
};
pub use reverse_cloze::ReverseClozeExercise;
pub use service::ExerciseService;
pub use translate_phrase::TranslatePhraseExercise;
pub use translation::{ContextualTranslationExercise, TranslationExercise};
pub use types::{Exercise, ExerciseResponse, ExerciseType};
