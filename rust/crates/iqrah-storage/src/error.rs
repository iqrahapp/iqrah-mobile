use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Incompatible schema version: DB {db_version}, App {app_version} - {message}")]
    IncompatibleSchema {
        db_version: String,
        app_version: String,
        message: String,
    },

    #[error("Invalid node ID: {node_id} - {reason}")]
    InvalidNodeId { node_id: String, reason: String },

    #[error("Other error: {0}")]
    Other(String),
}

impl From<iqrah_core::domain::error::NodeIdError> for StorageError {
    fn from(err: iqrah_core::domain::error::NodeIdError) -> Self {
        StorageError::InvalidNodeId {
            node_id: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, StorageError>;
