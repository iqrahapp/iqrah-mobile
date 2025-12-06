pub mod cbor_import;
pub mod domain;
pub mod exercises;
pub mod import;
pub mod ports;
pub mod scheduler_v2;
pub mod semantic;
pub mod services;

#[cfg(test)]
pub mod testing;

// Re-export commonly used types
pub use domain::{
    // V2 Domain Models
    Chapter,
    // Package Management
    ContentPackage,
    DistributionType,
    DomainError,
    // Echo Recall types
    EchoRecallState,
    EchoRecallStats,
    EchoRecallWord,
    Edge,
    EdgeType,
    Exercise,
    Hint,
    ImportStats,
    ImportedEdge,
    ImportedNode,
    InstalledPackage,
    // Knowledge Axis (Phase 4)
    KnowledgeAxis,
    KnowledgeNode,
    Language,
    Lemma,
    MemoryState,
    // Morphology Models
    MorphologySegment,
    Node,
    NodeType,
    PackageType,
    PropagationDetail,
    PropagationEvent,
    ReviewGrade,
    Root,
    Translator,
    Verse,
    Word,
    WordVisibility,
};

pub use ports::{ContentRepository, UserRepository};

pub use services::{LearningService, PackageService, ScoreWeights, ScoredItem, SessionService};

pub use scheduler_v2::{
    calculate_days_overdue, calculate_priority_score, calculate_readiness,
    count_unsatisfied_parents, generate_session, CandidateNode, InMemNode, ParentEnergyMap,
    SessionMode, UserProfile, MASTERY_THRESHOLD,
};

pub use exercises::{
    AyahChainExercise, AyahChainStats, AyahSequenceExercise, ClozeDeletionExercise,
    CrossVerseConnectionExercise, ExerciseResponse, ExerciseService, FindMistakeExercise,
    FirstLetterHintExercise, FullVerseInputExercise, IdentifyRootExercise, McqExercise,
    MissingWordMcqExercise, NextWordDifficulty, NextWordMcqExercise, PosTaggingExercise,
    ReverseClozeExercise, TranslatePhraseExercise,
};

pub use semantic::{
    SemanticEmbedder, SemanticGrade, SemanticGradeLabel, SemanticGrader, SEMANTIC_EMBEDDER,
};

pub use cbor_import::import_cbor_graph_from_bytes;

pub use import::{import_translators_from_json, TranslatorImportStats, TranslatorMetadata};
