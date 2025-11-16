pub mod domain;
pub mod ports;
pub mod services;
pub mod cbor_import;

// Re-export commonly used types
pub use domain::{
    Node, NodeType, Edge, EdgeType, DistributionType,
    MemoryState, ReviewGrade, Exercise,
    PropagationEvent, PropagationDetail,
    DomainError,
    ImportedNode, ImportedEdge, ImportStats,
    // Echo Recall types
    Hint, WordVisibility, EchoRecallWord, EchoRecallState,
};

pub use ports::{ContentRepository, UserRepository};

pub use services::{LearningService, SessionService, ScoredItem, ScoreWeights};

pub use cbor_import::import_cbor_graph_from_bytes;
