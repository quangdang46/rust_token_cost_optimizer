//! In-memory CCR store for testing and ephemeral use.
//!
//! Stores all entries in a `HashMap` backed by `std::sync::Mutex`.
//! No expiry, no persistence — data is lost when the store is dropped.

use super::CcrStore;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Mutex;

/// Thread-safe in-memory CCR store.
///
/// Useful for testing CCR integration without a SQLite dependency.
/// All entries are kept until explicitly removed or the store is dropped.
#[derive(Debug)]
pub struct InMemoryCcrStore {
    data: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryCcrStore {
    /// Create a new empty in-memory store.
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }

    /// Remove all entries from the store.
    pub fn clear(&self) {
        self.data.lock().unwrap().clear();
    }
}

impl Default for InMemoryCcrStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CcrStore for InMemoryCcrStore {
    fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        self.data
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.data.lock().unwrap().get(key).cloned())
    }

    fn len(&self) -> usize {
        self.data.lock().unwrap().len()
    }

    fn contains(&self, key: &str) -> Result<bool> {
        Ok(self.data.lock().unwrap().contains_key(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ccr::compute_key;

    #[test]
    fn test_put_get_roundtrip() {
        let store = InMemoryCcrStore::new();
        let key = compute_key(b"memory data");
        store.put(&key, b"memory data").unwrap();
        let retrieved = store.get(&key).unwrap().expect("should exist");
        assert_eq!(retrieved, b"memory data");
    }

    #[test]
    fn test_len() {
        let store = InMemoryCcrStore::new();
        assert_eq!(store.len(), 0);
        store.put("a", b"1").unwrap();
        store.put("b", b"2").unwrap();
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_clear() {
        let store = InMemoryCcrStore::new();
        store.put("key", b"val").unwrap();
        assert_eq!(store.len(), 1);
        store.clear();
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_contains() {
        let store = InMemoryCcrStore::new();
        store.put("exists", b"yes").unwrap();
        assert!(store.contains("exists").unwrap());
        assert!(!store.contains("missing").unwrap());
    }

    #[test]
    fn test_overwrite() {
        let store = InMemoryCcrStore::new();
        let key = compute_key(b"original");
        store.put(&key, b"original").unwrap();
        store.put(&key, b"updated").unwrap();
        let retrieved = store.get(&key).unwrap().expect("should exist");
        assert_eq!(retrieved, b"updated");
    }
}
