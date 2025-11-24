use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeIdError {
    #[error("Invalid node ID format: {0}")]
    InvalidFormat(String),

    #[error("Invalid node type prefix: {0}")]
    InvalidPrefix(String),

    #[error("Invalid chapter number: {0} (must be 1-114)")]
    InvalidChapter(u8),

    #[error("Invalid verse number: {0} (must be >= 1)")]
    InvalidVerse(u16),

    #[error("Invalid knowledge axis: {0}")]
    InvalidAxis(String),

    #[error("Malformed node ID: {0}")]
    Malformed(String),
}
