use sha2::{Sha256, Digest};
use std::io::{self, Read};

/// Computes the SHA-256 hash of the given data
pub fn compute_sha256<R: Read>(mut reader: R) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192]; // 8KB buffer

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verifies that the given data matches the expected checksum
pub fn verify_checksum<R: Read>(reader: R, expected: &str) -> crate::Result<bool> {
    let actual = compute_sha256(reader)?;
    Ok(actual == expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_compute_sha256() {
        let data = b"Hello, World!";
        let hash = compute_sha256(Cursor::new(data)).unwrap();
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn test_verify_checksum() {
        let data = b"Hello, World!";
        let expected = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";
        
        assert!(verify_checksum(Cursor::new(data), expected).unwrap());
        assert!(!verify_checksum(Cursor::new(b"Different data"), expected).unwrap());
    }
} 