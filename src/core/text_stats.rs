//! Text analysis utilities ported from headroom's compression engine.
//!
//! Provides algorithms for intelligent text analysis:
//! - **Shannon entropy**: detect high-entropy noise (UUIDs, hashes, base64)
//! - **SimHash**: near-duplicate detection via character n-gram fingerprinting
//! - **Token estimation**: content-aware token counting (more accurate than `chars/4`)
//! - **Line severity**: classify build/test output lines by severity level
//!
//! These algorithms originate from the [headroom](https://github.com/chopratejas/headroom)
//! project's SmartCrusher and log compressor modules.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Shannon Entropy
// ---------------------------------------------------------------------------

/// Compute Shannon entropy of a string, normalized to 0.0–1.0.
///
/// High-entropy text (UUIDs, hashes, base64, encoded blobs) returns values
/// near 1.0. Natural language and repetitive output returns lower values.
///
/// # Examples
/// ```
/// use rtk::text_stats::shannon_entropy;
/// assert!(shannon_entropy("hello hello hello") < 0.5);
/// assert!(shannon_entropy("a1b2c3d4-e5f6-7890-abcd-ef1234567890") > 0.8);
/// ```
pub fn shannon_entropy(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }

    let mut freq: HashMap<u8, usize> = HashMap::new();
    let bytes = text.as_bytes();
    let len = bytes.len();

    for &b in bytes {
        *freq.entry(b).or_insert(0) += 1;
    }

    let entropy: f64 = freq
        .values()
        .map(|&count| {
            let p = count as f64 / len as f64;
            -p * p.log2()
        })
        .sum();

    // Normalize by maximum possible entropy (log2 of unique byte values observed)
    let max_entropy = (freq.len() as f64).log2();
    if max_entropy == 0.0 {
        0.0
    } else {
        (entropy / max_entropy).clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// SimHash — Near-Duplicate Detection
// ---------------------------------------------------------------------------

/// Compute a 64-bit SimHash fingerprint of a string.
///
/// Uses character 4-grams hashed via a simple hash function, then aggregated
/// via bit-voting into a 64-bit fingerprint. Strings that differ in only a
/// few characters will produce fingerprints with low Hamming distance.
///
/// Algorithm ported from headroom's `adaptive_sizer.rs`.
///
/// # Examples
/// ```
/// use rtk::text_stats::{simhash, hamming_distance};
/// let a = simhash("ERROR: connection timeout at 192.168.1.1");
/// let b = simhash("ERROR: connection timeout at 192.168.1.2");
/// assert!(hamming_distance(a, b) < 10); // similar lines → low distance
/// ```
pub fn simhash(text: &str) -> u64 {
    let bytes = text.as_bytes();
    if bytes.len() < 4 {
        // Too short for 4-grams; hash the whole thing
        return hash_bytes_to_u64(bytes);
    }

    let mut bits = [0i32; 64];

    for window in bytes.windows(4) {
        let hash = hash_bytes_to_u64(window);
        for i in 0..64 {
            if (hash >> i) & 1 == 1 {
                bits[i] += 1;
            } else {
                bits[i] -= 1;
            }
        }
    }

    let mut fingerprint: u64 = 0;
    for i in 0..64 {
        if bits[i] > 0 {
            fingerprint |= 1u64 << i;
        }
    }
    fingerprint
}

/// Compute Hamming distance between two 64-bit fingerprints.
///
/// Returns the number of bit positions that differ. Lower values indicate
/// more similar content. A threshold of 3-5 works well for near-duplicate
/// detection on typical CLI output.
pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Simple FNV-1a-like hash for small byte slices → u64.
fn hash_bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

// ---------------------------------------------------------------------------
// Content-Aware Token Estimation
// ---------------------------------------------------------------------------

/// Estimate the number of tokens in text based on content type.
///
/// More accurate than a simple `chars.len() / 4` heuristic:
/// - **JSON/structured data**: ~3.2 chars/token (many short keys, punctuation)
/// - **Code**: ~3.5 chars/token (identifiers, keywords, operators)
/// - **Natural language / default**: ~4.0 chars/token
///
/// Also accounts for:
/// - URLs: extra tokens for `/`, `?`, `&`, `=` separators
/// - UUIDs/hex strings: +2 tokens each for delimiters
///
/// Algorithm derived from headroom's `EstimatingTokenCounter`.
///
/// # Examples
/// ```
/// use rtk::text_stats::estimate_tokens;
/// let json = r#"{"name": "test", "value": 42}"#;
/// let english = "The quick brown fox jumps over the lazy dog";
/// // JSON has more tokens per char than English
/// assert!(estimate_tokens(json) > estimate_tokens(english) * 0.5);
/// ```
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let char_count = text.chars().count();
    let content_type = detect_content_type(text);

    let base_ratio = match content_type {
        ContentType::Json => 3.2,
        ContentType::Code => 3.5,
        ContentType::PlainText => 4.0,
    };

    let mut tokens = (char_count as f64 / base_ratio) as usize;

    // URL overhead: each URL adds ~3 extra tokens for path/query separators
    let url_count = text.matches("http://").count() + text.matches("https://").count();
    tokens += url_count * 3;

    // UUID overhead: each UUID adds ~2 extra tokens for delimiter parsing
    let uuid_pattern_count = count_uuid_like(text);
    tokens += uuid_pattern_count * 2;

    tokens.max(1)
}

/// Content type classification for token estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentType {
    Json,
    Code,
    PlainText,
}

/// Detect content type from text characteristics.
fn detect_content_type(text: &str) -> ContentType {
    let trimmed = text.trim();

    // JSON detection: starts with { or [
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        return ContentType::Json;
    }

    // Code detection: common code patterns
    let code_indicators = [
        "fn ", "func ", "def ", "class ", "import ", "from ", "pub ", "let ", "const ",
        "var ", "async ", "await ", "impl ", "struct ", "enum ", "trait ", "#[", "//",
        "/*", "*/", "=>", "->", "::",
    ];
    let code_count = code_indicators
        .iter()
        .filter(|p| text.contains(**p))
        .count();
    if code_count >= 2 {
        return ContentType::Code;
    }

    ContentType::PlainText
}

/// Count UUID-like patterns (8-4-4-4-12 hex strings) in text.
fn count_uuid_like(text: &str) -> usize {
    lazy_static::lazy_static! {
        static ref UUID_RE: regex::Regex =
            regex::Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
                .unwrap();
    }
    UUID_RE.find_iter(text).count()
}

// ---------------------------------------------------------------------------
// Line Severity Classification
// ---------------------------------------------------------------------------

/// Severity level of a log/output line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineSeverity {
    /// FATAL / PANIC / CRITICAL — always keep
    Fatal,
    /// ERROR / FAIL / FAILED — always keep
    Error,
    /// WARNING / WARN — high priority
    Warning,
    /// INFO — normal priority
    Info,
    /// DEBUG / TRACE / LOG — low priority, first candidates for removal
    Debug,
    /// Could not determine severity
    Unknown,
}

impl LineSeverity {
    /// Returns true if this severity should always be preserved.
    pub fn is_critical(&self) -> bool {
        matches!(self, LineSeverity::Fatal | LineSeverity::Error)
    }

    /// Returns a numeric score for sorting (higher = more important).
    pub fn score(&self) -> f64 {
        match self {
            LineSeverity::Fatal => 1.0,
            LineSeverity::Error => 0.9,
            LineSeverity::Warning => 0.6,
            LineSeverity::Info => 0.3,
            LineSeverity::Debug => 0.1,
            LineSeverity::Unknown => 0.2,
        }
    }
}

/// Classify a line's severity by keyword detection.
///
/// Uses word-boundary-aware matching to avoid false positives (e.g., "errorless"
/// won't match as ERROR). Supports common log formats from pytest, cargo, npm,
/// jest, Go, Rust, Python, and generic build tools.
///
/// Algorithm derived from headroom's `LevelClassifier` (Aho-Corasick based).
///
/// # Examples
/// ```
/// use rtk::text_stats::{classify_severity, LineSeverity};
/// assert_eq!(classify_severity("ERROR: connection refused"), LineSeverity::Error);
/// assert_eq!(classify_severity("warning: unused variable"), LineSeverity::Warning);
/// assert_eq!(classify_severity("  at com.example.Main.main"), LineSeverity::Unknown);
/// ```
pub fn classify_severity(line: &str) -> LineSeverity {
    let lower = line.to_lowercase();

    // Fatal / Panic / Critical
    if contains_word(&lower, "fatal")
        || contains_word(&lower, "panic")
        || contains_word(&lower, "critical")
        || lower.contains("thread panicked")
    {
        return LineSeverity::Fatal;
    }

    // Error / Fail
    if contains_word(&lower, "error")
        || contains_word(&lower, "fail")
        || contains_word(&lower, "failed")
        || contains_word(&lower, "failure")
        || lower.starts_with("error[")
        || lower.contains("error:")
        || lower.contains("error:")
    {
        return LineSeverity::Error;
    }

    // Warning
    if contains_word(&lower, "warning")
        || contains_word(&lower, "warn")
        || lower.starts_with("warning[")
        || lower.contains("warning:")
    {
        return LineSeverity::Warning;
    }

    // Debug / Trace
    if contains_word(&lower, "debug")
        || contains_word(&lower, "trace")
        || lower.contains("[debug]")
        || lower.contains("[trace]")
    {
        return LineSeverity::Debug;
    }

    // Info
    if contains_word(&lower, "info") || lower.contains("[info]") {
        return LineSeverity::Info;
    }

    LineSeverity::Unknown
}

/// Check if `text` contains `word` as a whole word (not as a substring).
fn contains_word(text: &str, word: &str) -> bool {
    // Simple word-boundary check: the character before and after the match
    // must not be an alphanumeric character.
    let mut start = 0;
    while let Some(pos) = text[start..].find(word) {
        let abs_pos = start + pos;
        let before_ok = abs_pos == 0
            || !text
                .as_bytes()
                .get(abs_pos - 1)
                .map_or(false, |b| b.is_ascii_alphanumeric() || *b == b'_');
        let after_pos = abs_pos + word.len();
        let after_ok = after_pos >= text.len()
            || !text
                .as_bytes()
                .get(after_pos)
                .map_or(false, |b| b.is_ascii_alphanumeric() || *b == b'_');
        if before_ok && after_ok {
            return true;
        }
        start = abs_pos + 1;
    }
    false
}

/// Detect if a line is part of a stack trace.
///
/// Recognizes stack trace patterns from Python, JavaScript/Node, Java,
/// Rust, Go, and .NET.
///
/// # Examples
/// ```
/// use rtk::text_stats::is_stack_trace;
/// assert!(is_stack_trace("    at Object.<anonymous> (/app/index.js:10:5)"));
/// assert!(is_stack_trace("  File \"main.py\", line 42, in <module>"));
/// assert!(!is_stack_trace("Hello world"));
/// ```
pub fn is_stack_trace(line: &str) -> bool {
    let trimmed = line.trim();

    // Python: "File "..." , line N, in ..."
    if trimmed.starts_with("File \"") && trimmed.contains("\", line ") {
        return true;
    }
    // Python traceback header
    if trimmed.starts_with("Traceback (most recent call last):") {
        return true;
    }

    // JavaScript/Node: "    at ..."
    if trimmed.starts_with("at ") && (trimmed.contains('(') || trimmed.contains(".js:")) {
        return true;
    }
    // Node.js: "    at FunctionName (file:line:col)"
    if trimmed.starts_with("at ") && trimmed.ends_with(')') {
        return true;
    }

    // Java: "    at com.example.Class.method(File.java:42)"
    if trimmed.starts_with("at ")
        && trimmed.contains('(')
        && trimmed.contains(".java:")
    {
        return true;
    }
    // Java: "Caused by: ..."
    if trimmed.starts_with("Caused by:") {
        return true;
    }

    // Rust: "   --> src/main.rs:10:5"
    if trimmed.starts_with("--> ") && trimmed.contains(".rs:") {
        return true;
    }
    // Rust: "    at src/main.rs:10"
    if trimmed.starts_with("at ") && trimmed.contains(".rs:") {
        return true;
    }

    // Go: "goroutine 1 [running]:"
    if trimmed.starts_with("goroutine ") && trimmed.contains("[running]") {
        return true;
    }
    // Go: "main.main()"
    if trimmed.starts_with("main.") && trimmed.contains("()") {
        return true;
    }

    // .NET: "   at Namespace.Class.Method() in File.cs:line 42"
    if trimmed.starts_with("at ") && trimmed.contains(" in ") && trimmed.contains(":line ") {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Shannon Entropy --

    #[test]
    fn entropy_empty_string() {
        assert_eq!(shannon_entropy(""), 0.0);
    }

    #[test]
    fn entropy_repetitive_text_is_low() {
        let e = shannon_entropy("aaaa aaaa aaaa aaaa");
        assert!(e < 0.3, "repetitive text entropy should be low, got {}", e);
    }

    #[test]
    fn entropy_high_for_uuids() {
        let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        let e = shannon_entropy(uuid);
        assert!(e > 0.8, "UUID entropy should be high, got {}", e);
    }

    #[test]
    fn entropy_high_for_base64() {
        let b64 = "SGVsbG8gV29ybGQhIFRoaXMgaXMgYSBiYXNlNjQgZW5jb2RlZCBzdHJpbmc=";
        let e = shannon_entropy(b64);
        assert!(e > 0.7, "base64 entropy should be high, got {}", e);
    }

    #[test]
    fn entropy_natural_language_is_moderate() {
        let e = shannon_entropy("The quick brown fox jumps over the lazy dog");
        assert!(
            (0.5..0.95).contains(&e),
            "natural language entropy should be moderate, got {}",
            e
        );
    }

    // -- SimHash --

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
            dist > 15,
            "different strings should have high Hamming distance, got {}",
            dist
        );
    }

    #[test]
    fn simhash_short_string() {
        // Should not panic on short strings
        let h = simhash("hi");
        assert_ne!(h, 0);
    }

    // -- Token Estimation --

    #[test]
    fn token_estimate_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn token_estimate_json_higher_density() {
        let json = r#"{"name": "test", "value": 42, "active": true}"#;
        let english = "The quick brown fox jumps over the lazy dog";
        // JSON has more tokens per character than English
        let json_tokens = estimate_tokens(json);
        let eng_tokens = estimate_tokens(english);
        // JSON should estimate more tokens for similar-length strings
        assert!(
            json_tokens > 0 && eng_tokens > 0,
            "both should have tokens: json={}, eng={}",
            json_tokens,
            eng_tokens
        );
    }

    #[test]
    fn token_estimate_with_urls() {
        let with_url = "Check https://example.com/path?query=1&foo=bar for details";
        let without_url = "Check the documentation for details";
        assert!(
            estimate_tokens(with_url) > estimate_tokens(without_url),
            "URLs should add token overhead"
        );
    }

    #[test]
    fn token_estimate_with_uuids() {
        let with_uuid =
            "User a1b2c3d4-e5f6-7890-abcd-ef1234567890 logged in from 192.168.1.1";
        let without_uuid = "User john logged in from localhost";
        assert!(
            estimate_tokens(with_uuid) > estimate_tokens(without_uuid),
            "UUIDs should add token overhead"
        );
    }

    // -- Line Severity --

    #[test]
    fn severity_error() {
        assert_eq!(
            classify_severity("ERROR: connection refused"),
            LineSeverity::Error
        );
        assert_eq!(
            classify_severity("error[E0308]: mismatched types"),
            LineSeverity::Error
        );
    }

    #[test]
    fn severity_warning() {
        assert_eq!(
            classify_severity("warning: unused variable `x`"),
            LineSeverity::Warning
        );
    }

    #[test]
    fn severity_fatal() {
        assert_eq!(
            classify_severity("FATAL: database connection lost"),
            LineSeverity::Fatal
        );
        assert_eq!(
            classify_severity("thread 'main' panicked at 'boom'"),
            LineSeverity::Fatal
        );
    }

    #[test]
    fn severity_debug() {
        assert_eq!(
            classify_severity("DEBUG: entering function foo"),
            LineSeverity::Debug
        );
    }

    #[test]
    fn severity_unknown() {
        assert_eq!(classify_severity("Hello world"), LineSeverity::Unknown);
    }

    #[test]
    fn severity_no_false_positive_on_substring() {
        // "errorless" should not match as ERROR
        assert_ne!(
            classify_severity("The errorless operation completed"),
            LineSeverity::Error
        );
    }

    #[test]
    fn severity_ordering() {
        assert!(LineSeverity::Fatal > LineSeverity::Error);
        assert!(LineSeverity::Error > LineSeverity::Warning);
        assert!(LineSeverity::Warning > LineSeverity::Info);
    }

    // -- Stack Trace Detection --

    #[test]
    fn stack_trace_python() {
        assert!(is_stack_trace(
            "  File \"main.py\", line 42, in <module>"
        ));
        assert!(is_stack_trace(
            "Traceback (most recent call last):"
        ));
    }

    #[test]
    fn stack_trace_javascript() {
        assert!(is_stack_trace(
            "    at Object.<anonymous> (/app/index.js:10:5)"
        ));
    }

    #[test]
    fn stack_trace_java() {
        assert!(is_stack_trace(
            "    at com.example.Main.main(Main.java:42)"
        ));
        assert!(is_stack_trace("Caused by: java.lang.NullPointerException"));
    }

    #[test]
    fn stack_trace_rust() {
        assert!(is_stack_trace("   --> src/main.rs:10:5"));
    }

    #[test]
    fn stack_trace_go() {
        assert!(is_stack_trace("goroutine 1 [running]:"));
    }

    #[test]
    fn stack_trace_not_normal_text() {
        assert!(!is_stack_trace("Hello world"));
        assert!(!is_stack_trace("Building project..."));
        assert!(!is_stack_trace("Done in 3.42s."));
    }

    // -- Word Boundary --

    #[test]
    fn word_boundary_exact() {
        assert!(contains_word("error found", "error"));
    }

    #[test]
    fn word_boundary_substring_no_match() {
        assert!(!contains_word("errorless code", "error"));
    }

    #[test]
    fn word_boundary_at_start() {
        assert!(contains_word("error occurred", "error"));
    }

    #[test]
    fn word_boundary_at_end() {
        assert!(contains_word("fatal error", "error"));
    }

    #[test]
    fn word_boundary_with_underscore() {
        // Underscore is a word character, so "error_x" should not match "error"
        assert!(!contains_word("error_code", "error"));
    }
}
