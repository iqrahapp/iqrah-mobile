pub mod domain;
pub mod ports;

// Re-export commonly used types
pub use domain::{
    Node, NodeType, Edge, EdgeType, DistributionType,
    MemoryState, ReviewGrade, Exercise,
    PropagationEvent, PropagationDetail,
    DomainError,
};

pub use ports::{ContentRepository, UserRepository};
