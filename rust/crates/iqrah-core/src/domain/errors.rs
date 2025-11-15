use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid energy value: {0} (must be 0.0-1.0)")]
    InvalidEnergy(f64),

    #[error("Invalid review grade: {0}")]
    InvalidGrade(u8),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}
