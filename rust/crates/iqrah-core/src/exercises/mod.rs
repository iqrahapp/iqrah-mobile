// exercises/mod.rs
// Phase 4.3: Axis-Specific Exercise Generation

mod ayah_chain;
mod ayah_sequence;
mod find_mistake;
mod full_verse_input;
mod grammar;
mod graph;
mod mcq;
mod memorization;
mod pos_tagging;
mod reverse_cloze;
mod service;
mod translate_phrase;
mod translation;
mod types;

// Modern enum-based exercise architecture
mod exercise_data;
mod generators;
mod validator;

// Comprehensive tests for enum-based architecture
#[cfg(test)]
#[path = "enum_tests.rs"]
mod enum_tests;

pub use ayah_chain::{AyahChainExercise, AyahChainStats};
pub use ayah_sequence::AyahSequenceExercise;
pub use find_mistake::FindMistakeExercise;
pub use full_verse_input::FullVerseInputExercise;
pub use grammar::IdentifyRootExercise;
pub use graph::CrossVerseConnectionExercise;
pub use mcq::{McqExercise, McqType};
pub use memorization::{
    ClozeDeletionExercise, FirstLetterHintExercise, MemorizationExercise, MissingWordMcqExercise,
    NextWordDifficulty, NextWordMcqExercise,
};
pub use pos_tagging::PosTaggingExercise;
pub use reverse_cloze::ReverseClozeExercise;
pub use service::ExerciseService;
pub use translate_phrase::TranslatePhraseExercise;
pub use translation::{ContextualTranslationExercise, TranslationExercise};
pub use types::{Exercise, ExerciseResponse, ExerciseType};

// Export modern enum-based architecture
pub use exercise_data::ExerciseData;
pub use generators::*;
pub use validator::{
    AnswerInput, AnswerKeys, DefaultValidator, ExerciseValidator, ValidationResult,
};
