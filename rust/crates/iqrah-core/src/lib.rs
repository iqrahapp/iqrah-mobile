pub mod domain;
pub mod ports;
pub mod services;

// Re-export commonly used types
pub use domain::{
    Node, NodeType, Edge, EdgeType, DistributionType,
    MemoryState, ReviewGrade, Exercise,
    PropagationEvent, PropagationDetail,
    DomainError,
};

pub use ports::{ContentRepository, UserRepository};

pub use services::{LearningService, SessionService, ScoredItem, ScoreWeights};
