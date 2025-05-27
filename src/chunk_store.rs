use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write, Seek, SeekFrom, copy};
use crate::{BlobId, Result, error::Error};
use crate::checksum;

pub struct ChunkStore {
    root_dir: PathBuf,
}

#[derive(Debug)]
pub struct BlobInfo {
    pub size: u64,
    pub checksum: String,
}

impl ChunkStore {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self> {
        let root_dir = root_dir.into();
        fs::create_dir_all(&root_dir)?;
        fs::create_dir_all(root_dir.join("chunks"))?;
        Ok(Self { root_dir })
    }

    /// Returns the path to a blob file
    fn blob_path(&self, blob_id: &BlobId) -> PathBuf {
        self.root_dir
            .join("chunks")
            .join(format!("{}.blob", blob_id.to_string()))
    }

    /// Returns the path to a blob's checksum file
    fn checksum_path(&self, blob_id: &BlobId) -> PathBuf {
        self.root_dir
            .join("chunks")
            .join(format!("{}.blob.chk", blob_id.to_string()))
    }

    /// Stores a blob and its checksum, returns the blob info
    pub fn put_blob(&self, blob_id: &BlobId, mut data: impl Read) -> Result<BlobInfo> {
        let blob_path = self.blob_path(blob_id);
        let checksum_path = self.checksum_path(blob_id);

        // Create a temporary file for the blob
        let mut temp_file = tempfile::NamedTempFile::new()?;
        
        // Copy data to temp file while computing size
        let size = copy(&mut data, &mut temp_file)?;
        temp_file.flush()?;
        temp_file.seek(SeekFrom::Start(0))?;

        // Compute checksum
        let checksum = checksum::compute_sha256(&mut temp_file)?;

        // Write checksum file
        fs::write(&checksum_path, &checksum)?;

        // Persist the blob file
        temp_file.persist(&blob_path)?;

        Ok(BlobInfo { size, checksum })
    }

    /// Retrieves a blob and verifies its checksum
    pub fn get_blob(&self, blob_id: &BlobId) -> Result<(impl Read, BlobInfo)> {
        let blob_path = self.blob_path(blob_id);
        let checksum_path = self.checksum_path(blob_id);

        if !blob_path.exists() {
            return Err(Error::BlobNotFound(blob_id.to_string()));
        }

        let expected_checksum = fs::read_to_string(&checksum_path)?;
        let file = File::open(&blob_path)?;
        let size = file.metadata()?.len();
        
        // Create a copy for checksum verification
        let mut verify_file = File::open(&blob_path)?;
        
        // Verify checksum
        if !checksum::verify_checksum(&mut verify_file, &expected_checksum)? {
            return Err(Error::ChecksumMismatch {
                expected: expected_checksum,
                actual: checksum::compute_sha256(&mut verify_file)?,
            });
        }

        Ok((file, BlobInfo { size, checksum: expected_checksum }))
    }

    /// Deletes a blob and its checksum file
    pub fn delete_blob(&self, blob_id: &BlobId) -> Result<()> {
        let blob_path = self.blob_path(blob_id);
        let checksum_path = self.checksum_path(blob_id);

        if blob_path.exists() {
            fs::remove_file(&blob_path)?;
        }
        if checksum_path.exists() {
            fs::remove_file(&checksum_path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_chunk_store_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = ChunkStore::new(temp_dir.path()).unwrap();
        let blob_id = BlobId::new();
        let data = b"Hello, World!";

        // Test put_blob
        let info = store.put_blob(&blob_id, Cursor::new(data)).unwrap();
        assert_eq!(info.size, data.len() as u64);
        assert!(!info.checksum.is_empty());

        // Test get_blob
        let (mut reader, retrieved_info) = store.get_blob(&blob_id).unwrap();
        assert_eq!(retrieved_info.size, info.size);
        assert_eq!(retrieved_info.checksum, info.checksum);

        let mut retrieved = Vec::new();
        reader.read_to_end(&mut retrieved).unwrap();
        assert_eq!(&retrieved, data);

        // Test delete_blob
        store.delete_blob(&blob_id).unwrap();
        assert!(store.get_blob(&blob_id).is_err());
    }
} 