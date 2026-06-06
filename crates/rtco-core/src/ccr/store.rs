//! SQLite-backed CCR store.
//!
//! Stores compressed original content in a SQLite database with:
//! - WAL mode for concurrent access
//! - Lazy purge: expired entries are silently deleted on `get()`
//! - Configurable TTL (default: 7 days)

use super::CcrStore;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

/// SQLite-backed CCR store.
///
/// Opens a database at the given path and creates the `ccr_entries` table
/// on first access. Uses WAL mode for performance.
#[derive(Debug)]
pub struct SqliteCcrStore {
    conn: Mutex<Connection>,
    default_ttl_seconds: Option<i64>,
}

impl SqliteCcrStore {
    /// Open (or create) a CCR store at the given path.
    ///
    /// Pass `":memory:"` for an ephemeral in-memory database (useful in tests).
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open CCR SQLite database")?;

        // WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .context("Failed to set WAL mode")?;

        // Create table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ccr_entries (
                hash TEXT PRIMARY KEY,
                original BLOB NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                ttl_seconds INTEGER
            );",
            [],
        )
        .context("Failed to create ccr_entries table")?;

        // Index for lazy purge queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ccr_created ON ccr_entries(created_at);",
            [],
        )
        .context("Failed to create idx_ccr_created index")?;

        let store = Self {
            conn: Mutex::new(conn),
            default_ttl_seconds: Some(7 * 24 * 3600), // 7 days
        };

        Ok(store)
    }

    /// Set the default TTL for new entries.
    ///
    /// Pass `None` for no expiry. Default is 7 days.
    pub fn with_default_ttl(mut self, ttl_seconds: Option<i64>) -> Self {
        self.default_ttl_seconds = ttl_seconds;
        self
    }

    /// Prune all expired entries from the database.
    ///
    /// Returns the number of rows deleted.
    pub fn purge_expired(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let deleted = conn
            .execute(
                "DELETE FROM ccr_entries WHERE ttl_seconds IS NOT NULL AND created_at + ttl_seconds < ?1",
                params![now],
            )
            .context("Failed to purge expired CCR entries")?;
        Ok(deleted)
    }

    /// Get the total number of bytes stored in the database.
    pub fn total_bytes(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let total: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(LENGTH(original)), 0) FROM ccr_entries",
                [],
                |row| row.get(0),
            )
            .context("Failed to query total bytes")?;
        Ok(total)
    }
}

impl CcrStore for SqliteCcrStore {
    fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO ccr_entries (hash, original, created_at, ttl_seconds) VALUES (?1, ?2, ?3, ?4)",
            params![key, value, now, self.default_ttl_seconds],
        )
        .context("Failed to insert CCR entry")?;
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        // Check if entry exists and is not expired
        let result: Option<Vec<u8>> = conn
            .query_row(
                "SELECT original FROM ccr_entries WHERE hash = ?1 AND (ttl_seconds IS NULL OR created_at + ttl_seconds > ?2)",
                params![key, now],
                |row| row.get(0),
            )
            .ok();

        // Lazy purge: if entry exists but expired, delete it
        if result.is_none() {
            conn.execute(
                "DELETE FROM ccr_entries WHERE hash = ?1 AND ttl_seconds IS NOT NULL AND created_at + ttl_seconds <= ?2",
                params![key, now],
            )
            .ok();
        }

        Ok(result)
    }

    fn len(&self) -> usize {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM ccr_entries", [], |row| row.get(0))
            .unwrap_or(0)
    }

    fn contains(&self, key: &str) -> Result<bool> {
        Ok(self.get(key)?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ccr::compute_key;

    #[test]
    fn test_put_get_roundtrip() {
        let store = SqliteCcrStore::open(":memory:").unwrap();
        let data = b"original content to store";
        let key = compute_key(data);

        store.put(&key, data).unwrap();
        let retrieved = store.get(&key).unwrap().expect("should exist");
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_get_nonexistent() {
        let store = SqliteCcrStore::open(":memory:").unwrap();
        let result = store.get("nonexistent").unwrap();
        assert!(result.is_none(), "nonexistent key should return None");
    }

    #[test]
    fn test_len() {
        let store = SqliteCcrStore::open(":memory:").unwrap();
        assert_eq!(store.len(), 0);

        store.put("key1", b"data1").unwrap();
        assert_eq!(store.len(), 1);

        store.put("key2", b"data2").unwrap();
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_contains() {
        let store = SqliteCcrStore::open(":memory:").unwrap();
        let key = compute_key(b"test data");
        assert!(!store.contains(&key).unwrap());

        store.put(&key, b"test data").unwrap();
        assert!(store.contains(&key).unwrap());
    }

    #[test]
    fn test_total_bytes() {
        let store = SqliteCcrStore::open(":memory:").unwrap();
        store.put("k1", b"12345").unwrap(); // 5 bytes
        assert!(store.total_bytes().unwrap() >= 5);
    }

    #[test]
    fn test_purge_expired() {
        let store = SqliteCcrStore::open(":memory:")
            .unwrap()
            .with_default_ttl(Some(0)); // Expire immediately

        let key = compute_key(b"ephemeral");
        store.put(&key, b"ephemeral").unwrap();
        assert_eq!(store.len(), 1);

        // Sleep across a second boundary so the entry expires
        std::thread::sleep(std::time::Duration::from_secs(2));
        let purged = store.purge_expired().unwrap();
        assert_eq!(purged, 1);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_lazy_purge_on_get() {
        let store = SqliteCcrStore::open(":memory:")
            .unwrap()
            .with_default_ttl(Some(0)); // Expire immediately

        let key = compute_key(b"lazy purge");
        store.put(&key, b"lazy purge").unwrap();

        // Sleep across a second boundary so the entry expires
        std::thread::sleep(std::time::Duration::from_secs(2));

        // get() should lazily purge and return None
        let result = store.get(&key).unwrap();
        assert!(result.is_none(), "expired entry should return None");
        assert_eq!(store.len(), 0, "expired entry should be purged");
    }
}
