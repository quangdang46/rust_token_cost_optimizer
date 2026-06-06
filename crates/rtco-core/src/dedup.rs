//! Streaming near-duplicate detection filter.
//!
//! Uses SimHash fingerprinting to detect and drop near-duplicate lines in
//! streaming CLI output. Particularly effective for build logs, test output,
//! and repetitive command output where lines differ only in timestamps,
//! PIDs, or numeric values.
//!
//! Algorithm ported from headroom's adaptive sizer dedup logic.
//!
//! # Usage
//!
//! ```no_run
//! use rtco_core::dedup::DedupFilter;
//! use rtco_core::stream::StreamFilter;
//!
//! let mut filter = DedupFilter::new(3, 1000); // threshold=3, window=1000
//! let out = filter.feed_line("ERROR: timeout at 192.168.1.1");
//! // First occurrence passes through
//! assert!(out.is_some());
//!
//! let out = filter.feed_line("ERROR: timeout at 192.168.1.2");
//! // Near-duplicate: dropped (Hamming distance likely < 3)
//! // The exact behavior depends on the SimHash values
//! ```

use crate::stream::StreamFilter;
use crate::text_stats::{hamming_distance, simhash};

/// Streaming filter that drops near-duplicate lines based on SimHash similarity.
///
/// Maintains a sliding window of recent line fingerprints. New lines whose
/// fingerprint is within `threshold` Hamming distance of any recent fingerprint
/// are dropped as duplicates.
pub struct DedupFilter {
    /// Maximum Hamming distance to consider as duplicate.
    threshold: u32,
    /// Ring buffer of recent fingerprints.
    fingerprints: Vec<u64>,
    /// Maximum number of fingerprints to remember.
    max_window: usize,
    /// Current write position in the ring buffer.
    write_pos: usize,
    /// Whether the buffer is full (for ring buffer behavior).
    buffer_full: bool,
    /// Statistics: total lines seen.
    lines_seen: usize,
    /// Statistics: lines dropped as duplicates.
    duplicates_dropped: usize,
}

impl DedupFilter {
    /// Create a new dedup filter.
    ///
    /// # Arguments
    /// * `threshold` - Maximum Hamming distance to consider as duplicate (recommended: 3-5)
    /// * `max_window` - Number of recent fingerprints to remember (recommended: 500-2000)
    pub fn new(threshold: u32, max_window: usize) -> Self {
        Self {
            threshold,
            fingerprints: Vec::with_capacity(max_window.min(4096)),
            max_window,
            write_pos: 0,
            buffer_full: false,
            lines_seen: 0,
            duplicates_dropped: 0,
        }
    }

    /// Create a dedup filter with default settings (threshold=3, window=1000).
    pub fn default_config() -> Self {
        Self::new(3, 1000)
    }

    /// Check if a fingerprint is near-duplicate of any recent fingerprint.
    fn is_duplicate(&self, hash: u64) -> bool {
        let iter: Box<dyn Iterator<Item = &u64>> = if self.buffer_full {
            // Ring buffer is full: iterate all entries
            Box::new(self.fingerprints.iter())
        } else {
            // Buffer not full: only iterate filled portion
            Box::new(self.fingerprints[..self.write_pos].iter())
        };

        for &existing in iter {
            if hamming_distance(hash, existing) <= self.threshold {
                return true;
            }
        }
        false
    }

    /// Add a fingerprint to the ring buffer.
    fn remember(&mut self, hash: u64) {
        if self.fingerprints.len() < self.max_window {
            self.fingerprints.push(hash);
        } else {
            self.fingerprints[self.write_pos] = hash;
            self.buffer_full = true;
        }
        self.write_pos = (self.write_pos + 1) % self.max_window;
    }

    /// Get deduplication statistics.
    pub fn stats(&self) -> DedupStats {
        DedupStats {
            lines_seen: self.lines_seen,
            duplicates_dropped: self.duplicates_dropped,
            savings_percent: if self.lines_seen > 0 {
                (self.duplicates_dropped as f64 / self.lines_seen as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Reset the filter state (clear all remembered fingerprints).
    pub fn reset(&mut self) {
        self.fingerprints.clear();
        self.write_pos = 0;
        self.buffer_full = false;
        self.lines_seen = 0;
        self.duplicates_dropped = 0;
    }
}

impl StreamFilter for DedupFilter {
    fn feed_line(&mut self, line: &str) -> Option<String> {
        self.lines_seen += 1;

        // Empty lines always pass through
        if line.trim().is_empty() {
            return Some(line.to_string());
        }

        let hash = simhash(line);

        if self.is_duplicate(hash) {
            self.duplicates_dropped += 1;
            return None;
        }

        self.remember(hash);
        Some(line.to_string())
    }

    fn flush(&mut self) -> String {
        String::new()
    }
}

/// Statistics from the dedup filter.
#[derive(Debug, Clone)]
pub struct DedupStats {
    /// Total lines processed.
    pub lines_seen: usize,
    /// Lines dropped as near-duplicates.
    pub duplicates_dropped: usize,
    /// Percentage of lines dropped (0.0-100.0).
    pub savings_percent: f64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_identical_lines() {
        let mut filter = DedupFilter::new(0, 100); // exact match only
        assert!(filter.feed_line("ERROR: timeout").is_some());
        // Second identical line should be dropped
        assert!(filter.feed_line("ERROR: timeout").is_none());
    }

    #[test]
    fn dedup_different_lines_pass() {
        let mut filter = DedupFilter::new(3, 100);
        assert!(filter.feed_line("ERROR: connection timeout").is_some());
        assert!(filter
            .feed_line("Successfully compiled 42 targets")
            .is_some());
    }

    #[test]
    fn dedup_empty_lines_pass() {
        let mut filter = DedupFilter::new(0, 100);
        assert!(filter.feed_line("").is_some());
        assert!(filter.feed_line("  ").is_some());
    }

    #[test]
    fn dedup_stats_tracking() {
        let mut filter = DedupFilter::new(0, 100);
        filter.feed_line("line 1");
        filter.feed_line("line 1"); // duplicate
        filter.feed_line("line 2");

        let stats = filter.stats();
        assert_eq!(stats.lines_seen, 3);
        assert_eq!(stats.duplicates_dropped, 1);
        assert!((stats.savings_percent - 33.3).abs() < 0.1);
    }

    #[test]
    fn dedup_ring_buffer_eviction() {
        let mut filter = DedupFilter::new(0, 2); // tiny window
        filter.feed_line("a");
        filter.feed_line("b");
        // Window is full [a, b], write_pos wraps
        filter.feed_line("c"); // evicts "a"
                               // "a" should now pass through again (evicted from window)
        assert!(filter.feed_line("a").is_some());
    }

    #[test]
    fn dedup_reset() {
        let mut filter = DedupFilter::new(0, 100);
        filter.feed_line("ERROR: timeout");
        filter.feed_line("ERROR: timeout"); // duplicate
        assert_eq!(filter.stats().duplicates_dropped, 1);

        filter.reset();
        assert_eq!(filter.stats().lines_seen, 0);
        assert_eq!(filter.stats().duplicates_dropped, 0);
        // After reset, same line should pass through again
        assert!(filter.feed_line("ERROR: timeout").is_some());
    }

    #[test]
    fn dedup_default_config() {
        let filter = DedupFilter::default_config();
        assert_eq!(filter.threshold, 3);
        assert_eq!(filter.max_window, 1000);
    }

    #[test]
    fn dedup_similar_lines_with_threshold() {
        let mut filter = DedupFilter::new(5, 100);
        // These lines are very similar - should be caught by threshold
        assert!(filter
            .feed_line("ERROR: timeout connecting to 192.168.1.1:8080")
            .is_some());
        // Very similar but different IP - may or may not be caught depending on SimHash
        // Just verify it doesn't panic
        filter.feed_line("ERROR: timeout connecting to 192.168.1.2:8080");
    }
}
