//! Compression Context Registry (CCR) — reversible compression storage.
//!
//! Stores original content before compression so it can be restored on demand.
//! Dropped lines are replaced with a short marker (`<<ccr:HASH>>`) and the
//! original is saved to a store (SQLite or in-memory).
//!
//! # Architecture
//!
//! ```text
//!     CcrStore trait
//!        │
//!        ├── SqliteCcrStore  (persistent, production)
//!        └── InMemoryCcrStore (testing, ephemeral)
//! ```
//!
//! # Quick Start
//!
//! ```rust
//! use rtco_core::ccr::{CcrStore, SqliteCcrStore, compute_key, marker_for};
//!
//! let store = SqliteCcrStore::open(":memory:").unwrap();
//! let data = b"original content";
//! let key = compute_key(data);
//! store.put(&key, data).unwrap();
//! assert_eq!(store.len(), 1);
//! ```

mod memory;
mod store;

pub use memory::InMemoryCcrStore;
pub use store::SqliteCcrStore;

use std::fmt::Debug;

/// Trait for CCR storage backends.
///
/// Implementations must be `Send + Sync` to allow use across concurrent
/// filter pipelines.
pub trait CcrStore: Send + Sync + Debug {
    /// Store a value under the given key.
    fn put(&self, key: &str, value: &[u8]) -> anyhow::Result<()>;

    /// Retrieve a value by key, or `None` if not found or expired.
    fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>>;

    /// Return the number of entries currently stored.
    fn len(&self) -> usize;

    /// Return `true` if the store has no entries.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return `true` if the given key exists and is not expired.
    fn contains(&self, key: &str) -> anyhow::Result<bool> {
        Ok(self.get(key)?.is_some())
    }
}

/// Compute a storage key for the given data.
///
/// Uses BLAKE3 truncated to 24 hex characters (96 bits). Collision
/// probability for this application is negligible.
pub fn compute_key(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex()[..24].to_string()
}

/// Build a marker string that replaces compressed content.
///
/// The marker pattern is `<<ccr:HASH>>` where `HASH` is a 24-char hex string
/// from [`compute_key`].
pub fn marker_for(key: &str) -> String {
    format!("<<ccr:{}>>", key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_key_length() {
        let key = compute_key(b"hello world");
        assert_eq!(key.len(), 24, "BLAKE3 key should be 24 hex chars");
    }

    #[test]
    fn test_compute_key_deterministic() {
        let a = compute_key(b"same data");
        let b = compute_key(b"same data");
        assert_eq!(a, b, "same input should produce same key");
    }

    #[test]
    fn test_compute_key_different() {
        let a = compute_key(b"data one");
        let b = compute_key(b"data two");
        assert_ne!(a, b, "different inputs should produce different keys");
    }

    #[test]
    fn test_marker_for_format() {
        let key = "abcdef1234567890abcdef12";
        let marker = marker_for(key);
        assert_eq!(marker, format!("<<ccr:{}>>", key));
    }
}
