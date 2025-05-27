use crate::{TenantId, Result, error::Error};
use std::collections::HashSet;
use sled::Db;
use serde_json;

pub struct TenantManager {
    db: Db,
}

impl TenantManager {
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let db = sled::open(path.as_ref().join("tenants"))?;
        Ok(Self { db })
    }

    /// Registers a new tenant
    pub fn register_tenant(&self, tenant_id: TenantId) -> Result<()> {
        let key = tenant_id.as_str().as_bytes();
        self.db.insert(key, &[])?;
        Ok(())
    }

    /// Checks if a tenant exists
    pub fn tenant_exists(&self, tenant_id: &TenantId) -> Result<bool> {
        let key = tenant_id.as_str().as_bytes();
        Ok(self.db.contains_key(key)?)
    }

    /// Lists all registered tenants
    pub fn list_tenants(&self) -> Result<Vec<TenantId>> {
        let mut tenants = Vec::new();
        for key in self.db.iter().keys() {
            let key = key?;
            if let Ok(tenant_str) = std::str::from_utf8(&key) {
                tenants.push(TenantId::new(tenant_str));
            }
        }
        Ok(tenants)
    }

    /// Validates a tenant ID and returns an error if it doesn't exist
    pub fn validate_tenant(&self, tenant_id: &TenantId) -> Result<()> {
        if !self.tenant_exists(tenant_id)? {
            return Err(Error::InvalidTenant(tenant_id.as_str().to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn test_tenant_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = TenantManager::new(temp_dir.path()).unwrap();
        let tenant1 = TenantId::new("posts");
        let tenant2 = TenantId::new("messages");

        // Test registration
        assert!(manager.register_tenant(tenant1.clone()).is_ok());
        assert!(manager.register_tenant(tenant2.clone()).is_ok());

        // Test existence check
        assert!(manager.tenant_exists(&tenant1).unwrap());
        assert!(manager.tenant_exists(&tenant2).unwrap());
        assert!(!manager.tenant_exists(&TenantId::new("nonexistent")).unwrap());

        // Test validation
        assert!(manager.validate_tenant(&tenant1).is_ok());
        assert!(manager.validate_tenant(&TenantId::new("nonexistent")).is_err());

        // Test listing
        let tenants = manager.list_tenants().unwrap();
        assert_eq!(tenants.len(), 2);
        assert!(tenants.iter().any(|t| t.as_str() == tenant1.as_str()));
        assert!(tenants.iter().any(|t| t.as_str() == tenant2.as_str()));
    }
} 