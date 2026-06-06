//! Adaptive sizing utilities for determining optimal item counts.
//!
//! Provides algorithms for intelligent selection of how many items to keep
//! from a collection, based on diversity analysis:
//!
//! - **SimHash dedup counting**: count unique items by fingerprint similarity
//! - **Bigram coverage curve**: track cumulative unique bigram growth
//! - **Kneedle knee detection**: find the "elbow" where marginal utility drops
//!
//! Used by filter modules to decide how many results to display before
//! additional items add little new information.
//!
//! These algorithms originate from the [headroom](https://github.com/chopratejas/headroom)
//! project's adaptive sizer module.

#![allow(dead_code)] // Public API — consumed by filter modules

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// SimHash — 64-bit fingerprint from 4-grams via DefaultHasher + bit voting
// ---------------------------------------------------------------------------

/// Compute a 64-bit SimHash fingerprint of a string.
///
/// Uses character 4-grams hashed via `DefaultHasher`, then aggregated via
/// weighted bit voting into a 64-bit fingerprint. Strings that differ in only
/// a few characters will produce fingerprints with low Hamming distance.
///
/// # Examples
/// ```
/// use rtk::adaptive_sizer::simhash;
/// let a = simhash("ERROR: timeout at 192.168.1.1");
/// let b = simhash("ERROR: timeout at 192.168.1.2");
/// assert_ne!(a, b);
/// // Similar strings should have low Hamming distance
/// let dist = (a ^ b).count_ones();
/// assert!(dist < 15);
/// ```
pub fn simhash(text: &str) -> u64 {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return 0;
    }
    if bytes.len() < 4 {
        return hash_bytes_to_u64(bytes);
    }

    let mut bits = [0i64; 64];

    for window in bytes.windows(4) {
        let hash = hash_4gram(window);
        for (i, bit) in bits.iter_mut().enumerate() {
            if (hash >> i) & 1 == 1 {
                *bit += 1;
            } else {
                *bit -= 1;
            }
        }
    }

    let mut fingerprint: u64 = 0;
    for (i, &bit) in bits.iter().enumerate() {
        if bit > 0 {
            fingerprint |= 1u64 << i;
        }
    }
    fingerprint
}

/// Hash a 4-byte window using `DefaultHasher`.
fn hash_4gram(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Hash arbitrary bytes to u64 using `DefaultHasher`.
fn hash_bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Compute Hamming distance between two 64-bit fingerprints.
fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

// ---------------------------------------------------------------------------
// Count Unique by SimHash
// ---------------------------------------------------------------------------

/// Count the number of unique items in a slice using SimHash fingerprinting.
///
/// Two items are considered duplicates if their SimHash fingerprints have
/// Hamming distance <= `threshold` (default: 3). This is effective for CLI
/// output where lines differ only in timestamps, PIDs, or numeric values.
///
/// # Examples
/// ```
/// use rtk::adaptive_sizer::count_unique_by_simhash;
/// let items = vec![
///     "ERROR: timeout at 192.168.1.1",
///     "ERROR: timeout at 192.168.1.1",  // exact duplicate
///     "ERROR: timeout at 192.168.1.2",  // near duplicate
///     "Successfully compiled 42 targets",
/// ];
/// let unique = count_unique_by_simhash(&items);
/// assert!(unique <= 3); // near-duplicates should be collapsed
/// ```
pub fn count_unique_by_simhash(items: &[&str]) -> usize {
    count_unique_by_simhash_with_threshold(items, 3)
}

/// Count unique items with a configurable Hamming distance threshold.
pub fn count_unique_by_simhash_with_threshold(items: &[&str], threshold: u32) -> usize {
    if items.is_empty() {
        return 0;
    }

    let fingerprints: Vec<u64> = items.iter().map(|s| simhash(s)).collect();
    let mut unique_indices: Vec<usize> = Vec::new();

    for (i, &fp) in fingerprints.iter().enumerate() {
        let is_dup = unique_indices
            .iter()
            .any(|&j| hamming_distance(fp, fingerprints[j]) <= threshold);
        if !is_dup {
            unique_indices.push(i);
        }
    }

    unique_indices.len()
}

// ---------------------------------------------------------------------------
// Bigram Coverage Curve
// ---------------------------------------------------------------------------

/// Compute a cumulative unique bigram coverage curve from a text block.
///
/// Splits the text into lines, then for each prefix of lines computes the
/// fraction of unique character bigrams seen so far relative to the total
/// unique bigrams in the entire text. Returns a `Vec<f64>` of length equal
/// to the number of lines, with values normalized to 0.0-1.0.
///
/// # Examples
/// ```
/// use rtk::adaptive_sizer::bigram_coverage_curve;
/// let text = "line one\nline two\nline three\nline four\nline five";
/// let curve = bigram_coverage_curve(text);
/// assert_eq!(curve.len(), 5);
/// // First line should have some coverage
/// assert!(curve[0] > 0.0);
/// // Last line should be 1.0
/// assert!((curve[4] - 1.0).abs() < 0.001);
/// // Curve should be non-decreasing
/// for i in 1..curve.len() {
///     assert!(curve[i] >= curve[i - 1]);
/// }
/// ```
pub fn bigram_coverage_curve(text: &str) -> Vec<f64> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    // Compute all unique content bigrams (excluding newline characters) across all lines.
    // This avoids mismatch between the denominator (which would include '\n' bigrams in
    // joined text) and the numerator (which tracks per-line bigrams + cross-boundary).
    let mut all_bigrams: HashSet<(char, char)> = HashSet::new();
    for line in &lines {
        extract_bigrams(&mut all_bigrams, line);
    }
    // Also include cross-boundary bigrams between consecutive lines
    for w in lines.windows(2) {
        if let (Some(last), Some(first)) = (w[0].chars().last(), w[1].chars().next()) {
            all_bigrams.insert((last, first));
        }
    }

    if all_bigrams.is_empty() {
        // Text has no bigrams (single char or empty lines) — return 1.0 for all
        return vec![1.0; lines.len()];
    }
    let total = all_bigrams.len() as f64;

    // Cumulative coverage
    let mut seen: HashSet<(char, char)> = HashSet::new();
    let mut curve = Vec::with_capacity(lines.len());
    let mut prev_last_char: Option<char> = None;

    for line in &lines {
        extract_bigrams(&mut seen, line);
        // Add cross-boundary bigram (last char of previous line + first char of this line)
        if let (Some(prev), Some(first)) = (prev_last_char, line.chars().next()) {
            seen.insert((prev, first));
        }
        prev_last_char = line.chars().last();
        curve.push(seen.len() as f64 / total);
    }

    // Ensure the last value is exactly 1.0
    if let Some(last) = curve.last_mut() {
        *last = 1.0;
    }

    curve
}

/// Extract all character bigrams from text into a set.
fn extract_bigrams(set: &mut HashSet<(char, char)>, text: &str) {
    let chars: Vec<char> = text.chars().collect();
    for window in chars.windows(2) {
        set.insert((window[0], window[1]));
    }
}

// ---------------------------------------------------------------------------
// Kneedle Knee Detection — Compute Optimal K
// ---------------------------------------------------------------------------

/// Compute the optimal number of items to keep from a collection.
///
/// Uses a multi-stage approach:
/// 1. **Fast path**: if `items.len() <= 8`, return all items (small set, no need to cut)
/// 2. **SimHash diversity**: if unique-by-SimHash count <= 3, return that count
/// 3. **Kneedle**: compute the bigram coverage curve, normalize both axes to
///    0-1, and find the knee point where `y_norm - x_norm` is maximized
/// 4. **Low diversity fallback**: if `max_diff < 0.05`, scale by diversity ratio
///
/// # Examples
/// ```
/// use rtk::adaptive_sizer::compute_optimal_k;
/// // Small set returns all items
/// let items: Vec<&str> = vec!["a", "b", "c"];
/// assert_eq!(compute_optimal_k(&items, 100), 3);
/// ```
pub fn compute_optimal_k(items: &[&str], max_k: usize) -> usize {
    let n = items.len();
    if n == 0 {
        return 0;
    }

    // Fast path: small set, return everything (capped at max_k)
    if n <= 8 {
        return n.min(max_k);
    }

    // SimHash diversity check
    let unique_count = count_unique_by_simhash(items);
    if unique_count <= 3 {
        return unique_count.min(max_k);
    }

    // Build the bigram coverage curve from joined items
    let joined = items.join("\n");
    let curve = bigram_coverage_curve(&joined);

    if curve.is_empty() {
        return n.min(max_k);
    }

    // Normalize x-axis: 0..1 across indices
    let len = curve.len();
    let x_step = if len > 1 {
        1.0 / (len as f64 - 1.0)
    } else {
        1.0
    };

    // Find knee: maximize (y_norm - x_norm)
    let mut max_diff = f64::MIN;
    let mut knee_idx = 0;

    for (i, &y_val) in curve.iter().enumerate() {
        let x_val = i as f64 * x_step;
        let diff = y_val - x_val;
        if diff > max_diff {
            max_diff = diff;
            knee_idx = i;
        }
    }

    // Convert 0-based index to count (knee_idx + 1), but at least 1
    let k_from_knee = (knee_idx + 1).max(1);

    // Low diversity fallback: if the knee is not pronounced, scale by diversity ratio
    if max_diff < 0.05 {
        let diversity_ratio = unique_count as f64 / n as f64;
        let k = ((n as f64 * diversity_ratio).round() as usize).max(1);
        return k.min(max_k);
    }

    k_from_knee.min(max_k)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- SimHash --

    #[test]
    fn simhash_empty_string() {
        assert_eq!(simhash(""), 0);
    }

    #[test]
    fn simhash_short_string() {
        // Strings shorter than 4 bytes should not panic
        let h = simhash("hi");
        assert_ne!(h, 0);
    }

    #[test]
    fn simhash_identical_strings() {
        let a = simhash("ERROR: timeout at 192.168.1.1");
        let b = simhash("ERROR: timeout at 192.168.1.1");
        assert_eq!(hamming_distance(a, b), 0);
    }

    #[test]
    fn simhash_similar_strings_low_distance() {
        let a = simhash("ERROR: connection timeout at 192.168.1.1 port 8080");
        let b = simhash("ERROR: connection timeout at 192.168.1.2 port 8080");
        let dist = hamming_distance(a, b);
        assert!(
            dist < 15,
            "similar strings should have low Hamming distance, got {}",
            dist
        );
    }

    #[test]
    fn simhash_different_strings_high_distance() {
        let a = simhash("ERROR: connection timeout");
        let b = simhash("Successfully compiled 42 targets");
        let dist = hamming_distance(a, b);
        assert!(
            dist > 10,
            "different strings should have high Hamming distance, got {}",
            dist
        );
    }

    #[test]
    fn simhash_single_char() {
        let h = simhash("x");
        assert_ne!(h, 0);
    }

    // -- Count Unique by SimHash --

    #[test]
    fn count_unique_empty() {
        assert_eq!(count_unique_by_simhash(&[]), 0);
    }

    #[test]
    fn count_unique_exact_duplicates() {
        let items = vec!["hello", "hello", "hello"];
        assert_eq!(count_unique_by_simhash(&items), 1);
    }

    #[test]
    fn count_unique_different_items() {
        let items = vec![
            "ERROR: connection refused",
            "WARNING: disk space low",
            "INFO: server started",
        ];
        assert_eq!(count_unique_by_simhash(&items), 3);
    }

    #[test]
    fn count_unique_near_duplicates() {
        // Items that are extremely similar (differ by only 1-2 characters)
        let items = vec![
            "ERROR: timeout connecting to 192.168.1.1 port 8080 after 30s retry",
            "ERROR: timeout connecting to 192.168.1.1 port 8080 after 30s retries",
            "ERROR: timeout connecting to 192.168.1.1 port 8080 after 30s retried",
        ];
        let unique = count_unique_by_simhash(&items);
        // These are very similar; should collapse to 1-2 unique
        assert!(
            unique <= 2,
            "near-duplicates should collapse, got {} unique",
            unique
        );
    }

    #[test]
    fn count_unique_with_threshold() {
        let items = vec!["abc", "abc", "xyz"];
        // With threshold=0, only exact matches dedup
        assert_eq!(count_unique_by_simhash_with_threshold(&items, 0), 2);
    }

    // -- Bigram Coverage Curve --

    #[test]
    fn bigram_curve_empty() {
        assert!(bigram_coverage_curve("").is_empty());
    }

    #[test]
    fn bigram_curve_single_line() {
        let curve = bigram_coverage_curve("hello");
        assert_eq!(curve.len(), 1);
        assert!((curve[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn bigram_curve_non_decreasing() {
        let text = "alpha\nbeta\ngamma\ndelta\nepsilon\nzeta\neta\ntheta";
        let curve = bigram_coverage_curve(text);
        for i in 1..curve.len() {
            assert!(
                curve[i] >= curve[i - 1],
                "curve must be non-decreasing: curve[{}]={} < curve[{}]={}",
                i,
                curve[i],
                i - 1,
                curve[i - 1]
            );
        }
    }

    #[test]
    fn bigram_curve_ends_at_one() {
        let text = "line one\nline two\nline three";
        let curve = bigram_coverage_curve(text);
        assert!((curve.last().unwrap() - 1.0).abs() < 0.001);
    }

    #[test]
    fn bigram_curve_repetitive_content_plateaus() {
        // Repeated identical lines: coverage should plateau quickly.
        // Each line adds a cross-boundary bigram (last char -> first char)
        // that only appears between lines. After a few lines, all bigrams
        // including boundary ones are covered.
        let lines: Vec<&str> = (0..20).map(|_| "same line").collect();
        let text = lines.join("\n");
        let curve = bigram_coverage_curve(&text);
        // After the first line, within-line bigrams are all covered
        assert!(
            curve[0] >= 0.7,
            "repetitive content should have high initial coverage, got {}",
            curve[0]
        );
        // Curve should be non-decreasing and reach 1.0 at the end
        assert!(
            (curve.last().unwrap() - 1.0).abs() < 0.001,
            "should reach 1.0 at end"
        );
        // With 20 identical lines, the curve should plateau within a few lines
        // and the last value should equal the second-to-last (no new bigrams)
        let n = curve.len();
        assert!(
            (curve[n - 1] - curve[n - 2]).abs() < 0.001,
            "plateau: last two values should be equal"
        );
    }

    #[test]
    fn bigram_curve_diverse_content_grows_gradually() {
        // Each line introduces many new bigrams
        let lines = [
            "alpha beta gamma delta epsilon",
            "zeta eta theta iota kappa",
            "lambda mu nu xi omicron",
            "pi rho sigma tau upsilon",
            "phi chi psi omega finis",
        ];
        let text = lines.join("\n");
        let curve = bigram_coverage_curve(&text);
        // First line should have notably less than 100% coverage
        assert!(
            curve[0] < 0.9,
            "diverse first line should not cover all bigrams, got {}",
            curve[0]
        );
    }

    // -- Compute Optimal K --

    #[test]
    fn optimal_k_empty() {
        assert_eq!(compute_optimal_k(&[], 100), 0);
    }

    #[test]
    fn optimal_k_small_set_returns_all() {
        let items: Vec<&str> = vec!["a", "b", "c", "d", "e"];
        assert_eq!(compute_optimal_k(&items, 100), 5);
    }

    #[test]
    fn optimal_k_small_set_capped_by_max_k() {
        let items: Vec<&str> = vec!["a", "b", "c"];
        assert_eq!(compute_optimal_k(&items, 2), 2);
    }

    #[test]
    fn optimal_k_exactly_eight() {
        let items: Vec<&str> = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
        assert_eq!(compute_optimal_k(&items, 100), 8);
    }

    #[test]
    fn optimal_k_simhash_low_diversity() {
        // Items that are all near-duplicates of each other
        let items: Vec<&str> = (0..20)
            .map(|i| {
                // Leak to get 'static lifetime for the format string result
                let s = format!("ERROR: timeout at 192.168.1.{}:8080", i);
                &*Box::leak(s.into_boxed_str())
            })
            .collect();
        let k = compute_optimal_k(&items, 100);
        // With low SimHash diversity, should return a small number
        assert!(k <= 5, "low diversity should yield small k, got {}", k);
    }

    #[test]
    fn optimal_k_respects_max_k() {
        // Build 20 diverse items
        let items: Vec<&str> = (0..20)
            .map(|i| {
                let s = format!("completely unique item number {} with extra words", i);
                &*Box::leak(s.into_boxed_str())
            })
            .collect();
        let k = compute_optimal_k(&items, 5);
        assert!(k <= 5, "should respect max_k, got {}", k);
    }

    #[test]
    fn optimal_k_diverse_set_returns_more() {
        // Diverse content should yield a larger k
        let items: Vec<&str> = vec![
            "ERROR: database connection failed with timeout",
            "WARNING: disk usage exceeded 90 percent threshold",
            "INFO: application started successfully on port 8080",
            "DEBUG: cache miss for key user_session_12345",
            "TRACE: entering function process_request with payload",
            "FATAL: kernel panic: unable to mount root filesystem",
            "ERROR: out of memory: cannot allocate 1024 bytes",
            "WARNING: deprecated API usage detected in module auth",
            "INFO: health check passed all 12 service endpoints",
            "DEBUG: SQL query executed in 45ms on table users",
            "ERROR: SSL certificate verification failed for host api.example.com",
            "WARNING: rate limit threshold reached for IP 10.0.0.5",
            "INFO: deployment completed successfully to production cluster",
            "DEBUG: garbage collection freed 256MB of heap memory",
            "TRACE: HTTP request received GET /api/v2/users?page=1",
            "ERROR: file not found: /etc/app/config.yaml",
            "WARNING: retry attempt 3 of 5 for external service call",
            "INFO: cache invalidation completed for 1500 entries",
            "DEBUG: websocket connection established with client ID 7890",
            "TRACE: serialization completed in 12ms for response payload",
        ];
        let k = compute_optimal_k(&items, 20);
        // Diverse set should keep more items
        assert!(k >= 3, "diverse set should keep multiple items, got {}", k);
    }

    #[test]
    fn optimal_k_single_item() {
        let items: Vec<&str> = vec!["only one item here"];
        assert_eq!(compute_optimal_k(&items, 100), 1);
    }

    #[test]
    fn optimal_k_two_items() {
        let items: Vec<&str> = vec!["first item", "second item"];
        assert_eq!(compute_optimal_k(&items, 100), 2);
    }
}
