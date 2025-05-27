use thiserror::Error;
use tempfile::PersistError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Storage error: {0}")]
    Storage(#[from] sled::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("File persist error: {0}")]
    Persist(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch {
        expected: String,
        actual: String,
    },

    #[error("Blob not found: {0}")]
    BlobNotFound(String),

    #[error("Invalid tenant: {0}")]
    InvalidTenant(String),

    #[error("System error: {0}")]
    System(String),
}

impl From<PersistError> for Error {
    fn from(err: PersistError) -> Self {
        Error::Persist(err.to_string())
    }
} 