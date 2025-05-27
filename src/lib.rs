pub mod cli;
pub mod coordinator;
pub mod metadata;
pub mod chunk_store;
pub mod checksum;
pub mod tenant;
pub mod error;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a unique identifier for a blob
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BlobId(Uuid);

impl BlobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// Represents a tenant in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TenantId(String);

impl TenantId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Represents metadata about a stored blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobMetadata {
    pub blob_id: BlobId,
    pub tenant_id: TenantId,
    pub size: u64,
    pub checksum: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Result type for operations that can fail
pub type Result<T> = std::result::Result<T, error::Error>; 