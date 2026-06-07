//! Field-name hashing for cache keys.
//!
//! Ported from headroom's `transforms/smart_crusher/hashing.rs`.
//! SHA-256 of the UTF-8 bytes, hex-encoded, truncated to 8 chars.
//! Used to look up TOIN-anonymized `preserve_fields`.

use sha2::{Digest, Sha256};

/// SHA-256 of the UTF-8 bytes, hex-encoded, truncated to 8 chars.
///
/// Python equivalent: `hashlib.sha256(field_name.encode()).hexdigest()[:8]`.
pub fn hash_field_name(field_name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(field_name.as_bytes());
    let digest = hasher.finalize();
    let hex = format!("{:x}", digest);
    hex[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_python_sha256_truncated_8() {
        assert_eq!(hash_field_name("customer_id"), "1e38d67d");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(hash_field_name(""), "e3b0c442");
    }

    #[test]
    fn test_unicode() {
        assert_eq!(hash_field_name("café"), "850f7dc4");
    }

    #[test]
    fn test_deterministic() {
        assert_eq!(hash_field_name("test"), hash_field_name("test"));
    }

    #[test]
    fn test_output_length_8() {
        assert_eq!(hash_field_name("a").len(), 8);
        assert_eq!(hash_field_name(&"x".repeat(1000)).len(), 8);
    }
}
