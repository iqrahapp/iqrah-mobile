pub mod cbor_import;
pub mod domain;
pub mod ports;
pub mod services;

// Re-export commonly used types
pub use domain::{
    DistributionType,
    DomainError,
    EchoRecallState,
    EchoRecallWord,
    Edge,
    EdgeType,
    Exercise,
    // Echo Recall types
    Hint,
    ImportStats,
    ImportedEdge,
    ImportedNode,
    MemoryState,
    Node,
    NodeType,
    PropagationDetail,
    PropagationEvent,
    ReviewGrade,
    WordVisibility,
};

pub use ports::{ContentRepository, UserRepository};

pub use services::{LearningService, ScoreWeights, ScoredItem, SessionService};

pub use cbor_import::import_cbor_graph_from_bytes;
