use std::path::PathBuf;
use std::io::Read;
use chrono::Utc;

use crate::{
    BlobId, TenantId, BlobMetadata, Result,
    chunk_store::ChunkStore,
    metadata::MetadataStore,
    tenant::TenantManager,
};

pub struct Coordinator {
    chunk_store: ChunkStore,
    metadata_store: MetadataStore,
    tenant_manager: TenantManager,
}

impl Coordinator {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self> {
        let root_dir = root_dir.into();
        let chunk_store = ChunkStore::new(&root_dir)?;
        let metadata_store = MetadataStore::new(root_dir.join("metadata"))?;
        let tenant_manager = TenantManager::new(&root_dir)?;

        Ok(Self {
            chunk_store,
            metadata_store,
            tenant_manager,
        })
    }

    /// Registers a new tenant
    pub fn register_tenant(&self, tenant_id: TenantId) -> Result<()> {
        self.tenant_manager.register_tenant(tenant_id)
    }

    /// Lists all registered tenants
    pub fn list_tenants(&self) -> Result<Vec<TenantId>> {
        self.tenant_manager.list_tenants()
    }

    /// Stores a new blob
    pub fn put_blob(&self, tenant_id: &TenantId, data: impl Read) -> Result<BlobId> {
        // Validate tenant
        self.tenant_manager.validate_tenant(tenant_id)?;

        // Generate new blob ID
        let blob_id = BlobId::new();

        // Store the blob and get its info
        let blob_info = self.chunk_store.put_blob(&blob_id, data)?;

        // Create and store metadata
        let metadata = BlobMetadata {
            blob_id: blob_id.clone(),
            tenant_id: tenant_id.clone(),
            size: blob_info.size,
            checksum: blob_info.checksum,
            created_at: Utc::now(),
        };
        self.metadata_store.put_metadata(&metadata)?;

        Ok(blob_id)
    }

    /// Retrieves a blob
    pub fn get_blob(&self, tenant_id: &TenantId, blob_id: &BlobId) -> Result<impl Read> {
        // Validate tenant
        self.tenant_manager.validate_tenant(tenant_id)?;

        // Get metadata to verify tenant ownership
        let metadata = self.metadata_store.get_metadata(blob_id)?;
        if metadata.tenant_id != *tenant_id {
            return Err(crate::error::Error::InvalidTenant(
                "Blob does not belong to this tenant".into(),
            ));
        }

        // Get the blob (this will also verify checksum)
        let (reader, _) = self.chunk_store.get_blob(blob_id)?;
        Ok(reader)
    }

    /// Lists all blobs for a tenant
    pub fn list_blobs(&self, tenant_id: &TenantId) -> Result<Vec<BlobMetadata>> {
        // Validate tenant
        self.tenant_manager.validate_tenant(tenant_id)?;

        // Get all blob IDs for the tenant
        let blob_ids = self.metadata_store.get_tenant_blobs(tenant_id)?;

        // Get metadata for each blob
        let mut metadata_list = Vec::new();
        for blob_id in blob_ids {
            if let Ok(metadata) = self.metadata_store.get_metadata(&blob_id) {
                metadata_list.push(metadata);
            }
        }

        Ok(metadata_list)
    }

    /// Deletes a blob
    pub fn delete_blob(&self, tenant_id: &TenantId, blob_id: &BlobId) -> Result<()> {
        // Validate tenant
        self.tenant_manager.validate_tenant(tenant_id)?;

        // Get metadata to verify tenant ownership
        let metadata = self.metadata_store.get_metadata(blob_id)?;
        if metadata.tenant_id != *tenant_id {
            return Err(crate::error::Error::InvalidTenant(
                "Blob does not belong to this tenant".into(),
            ));
        }

        // Delete the blob and its metadata
        self.chunk_store.delete_blob(blob_id)?;
        self.metadata_store.delete_metadata(blob_id, tenant_id)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_coordinator_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let coordinator = Coordinator::new(temp_dir.path()).unwrap();

        // Register tenant
        let tenant_id = TenantId::new("posts");
        assert!(coordinator.register_tenant(tenant_id.clone()).is_ok());

        // Put blob
        let data = b"Hello, World!";
        let blob_id = coordinator.put_blob(&tenant_id, Cursor::new(data)).unwrap();

        // Get blob
        let mut retrieved = Vec::new();
        coordinator.get_blob(&tenant_id, &blob_id).unwrap().read_to_end(&mut retrieved).unwrap();
        assert_eq!(&retrieved, data);

        // List blobs
        let blobs = coordinator.list_blobs(&tenant_id).unwrap();
        assert_eq!(blobs.len(), 1);
        assert_eq!(blobs[0].blob_id, blob_id);
        assert_eq!(blobs[0].size, data.len() as u64);

        // Delete blob
        assert!(coordinator.delete_blob(&tenant_id, &blob_id).is_ok());
        assert!(coordinator.get_blob(&tenant_id, &blob_id).is_err());
    }
} 