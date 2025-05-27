use crate::{BlobId, TenantId, BlobMetadata, Result, error::Error};
use sled::Db;
use std::path::Path;
use serde_json;

pub struct MetadataStore {
    db: Db,
}

impl MetadataStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    /// Creates a metadata key for a tenant's blob list
    fn tenant_key(tenant_id: &TenantId) -> Vec<u8> {
        format!("tenant:{}:blobs", tenant_id.as_str()).into_bytes()
    }

    /// Creates a metadata key for a blob
    fn blob_key(blob_id: &BlobId) -> Vec<u8> {
        format!("blob:{}", blob_id.to_string()).into_bytes()
    }

    /// Stores blob metadata
    pub fn put_metadata(&self, metadata: &BlobMetadata) -> Result<()> {
        let blob_key = Self::blob_key(&metadata.blob_id);
        let tenant_key = Self::tenant_key(&metadata.tenant_id);

        // Store the blob metadata
        let metadata_json = serde_json::to_vec(metadata)?;
        self.db.insert(blob_key, metadata_json)?;

        // Update the tenant's blob list
        let mut blob_list = self.get_tenant_blobs(&metadata.tenant_id)?;
        if !blob_list.contains(&metadata.blob_id) {
            blob_list.push(metadata.blob_id.clone());
            let blob_list_json = serde_json::to_vec(&blob_list)?;
            self.db.insert(tenant_key, blob_list_json)?;
        }

        Ok(())
    }

    /// Retrieves blob metadata
    pub fn get_metadata(&self, blob_id: &BlobId) -> Result<BlobMetadata> {
        let blob_key = Self::blob_key(blob_id);
        let metadata_bytes = self.db
            .get(blob_key)?
            .ok_or_else(|| Error::BlobNotFound(blob_id.to_string()))?;
        
        Ok(serde_json::from_slice(&metadata_bytes)?)
    }

    /// Lists all blobs for a tenant
    pub fn get_tenant_blobs(&self, tenant_id: &TenantId) -> Result<Vec<BlobId>> {
        let tenant_key = Self::tenant_key(tenant_id);
        let blob_list = match self.db.get(tenant_key)? {
            Some(bytes) => serde_json::from_slice(&bytes)?,
            None => Vec::new(),
        };
        Ok(blob_list)
    }

    /// Deletes blob metadata
    pub fn delete_metadata(&self, blob_id: &BlobId, tenant_id: &TenantId) -> Result<()> {
        let blob_key = Self::blob_key(blob_id);
        let tenant_key = Self::tenant_key(tenant_id);

        // Remove from tenant's blob list
        if let Some(blob_list_bytes) = self.db.get(&tenant_key)? {
            let mut blob_list: Vec<BlobId> = serde_json::from_slice(&blob_list_bytes)?;
            blob_list.retain(|id| id != blob_id);
            let updated_list = serde_json::to_vec(&blob_list)?;
            self.db.insert(tenant_key, updated_list)?;
        }

        // Remove blob metadata
        self.db.remove(blob_key)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_metadata_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = MetadataStore::new(temp_dir.path()).unwrap();
        
        let tenant_id = TenantId::new("posts");
        let blob_id = BlobId::new();
        let metadata = BlobMetadata {
            blob_id: blob_id.clone(),
            tenant_id: tenant_id.clone(),
            size: 42,
            checksum: "test_checksum".to_string(),
            created_at: Utc::now(),
        };

        // Test put_metadata
        assert!(store.put_metadata(&metadata).is_ok());

        // Test get_metadata
        let retrieved = store.get_metadata(&blob_id).unwrap();
        assert_eq!(retrieved.blob_id, blob_id);
        assert_eq!(retrieved.tenant_id.as_str(), tenant_id.as_str());

        // Test get_tenant_blobs
        let blobs = store.get_tenant_blobs(&tenant_id).unwrap();
        assert_eq!(blobs.len(), 1);
        assert_eq!(blobs[0], blob_id);

        // Test delete_metadata
        assert!(store.delete_metadata(&blob_id, &tenant_id).is_ok());
        assert!(store.get_metadata(&blob_id).is_err());
        assert_eq!(store.get_tenant_blobs(&tenant_id).unwrap().len(), 0);
    }
} 