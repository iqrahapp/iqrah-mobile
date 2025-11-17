pub mod cbor_import;
pub mod domain;
pub mod import;
pub mod ports;
pub mod services;

// Re-export commonly used types
pub use domain::{
    // V2 Domain Models
    Chapter,
    DistributionType,
    DomainError,
    // Echo Recall types
    EchoRecallState,
    EchoRecallWord,
    Edge,
    EdgeType,
    Exercise,
    Hint,
    ImportStats,
    ImportedEdge,
    ImportedNode,
    Language,
    MemoryState,
    Node,
    NodeType,
    PropagationDetail,
    PropagationEvent,
    ReviewGrade,
    Translator,
    Verse,
    Word,
    WordVisibility,
};

pub use ports::{ContentRepository, UserRepository};

pub use services::{LearningService, ScoreWeights, ScoredItem, SessionService};

pub use cbor_import::import_cbor_graph_from_bytes;

pub use import::{import_translators_from_json, TranslatorImportStats, TranslatorMetadata};
