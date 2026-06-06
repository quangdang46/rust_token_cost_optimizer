//! Keyword-based line classifier using Aho-Corasick multi-pattern matching.
//!
//! Classifies log/output lines into semantic levels and assigns importance
//! scores. Uses the `aho-corasick` crate for efficient single-pass matching
//! against all keyword patterns simultaneously.
//!
//! This module complements [`super::text_stats::classify_severity`] by
//! providing a broader taxonomy (Security, Summary, StackTrace) and
//! a continuous score rather than a discrete severity level.

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};

/// Semantic classification of a log/output line.
///
/// Variants are ordered by importance score (highest first). The derived
/// `Ord` implementation uses declaration order, so lower index = higher
/// priority. Use `score()` for the numeric value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineLevel {
    /// error, exception, fail, failed, failure, fatal, critical, crash,
    /// panic, abort, timeout — score 1.0
    Error,
    /// vulnerability, injection, exploit, breach, denied, rejected — score 0.85
    Security,
    /// warn, warning, deprecated — score 0.75
    Warning,
    /// summary, total, result, passed, skipped — score 0.4
    Summary,
    /// Informational lines — score 0.3
    Info,
    /// Stack trace frames — score 0.2
    StackTrace,
    /// Debug/trace lines — score 0.1
    Debug,
    /// No keywords matched — score 0.0
    Plain,
}

/// Fast keyword-based line classifier backed by Aho-Corasick.
///
/// All keyword patterns are compiled once at construction time into a
/// single automaton, so `classify_line` and `score_line` run in O(n)
/// time over the input length regardless of keyword count.
pub struct KeywordDetector {
    ac: AhoCorasick,
    /// Parallel array mapping each pattern index to its `LineLevel`.
    levels: Vec<LineLevel>,
}

impl KeywordDetector {
    /// Build a detector with the default keyword set.
    ///
    /// Keywords are matched case-insensitively against the lowercased
    /// input line. Each keyword maps to exactly one [`LineLevel`].
    pub fn new() -> Self {
        // (keyword, level) pairs — order matters only for index mapping.
        let pairs: &[(&str, LineLevel)] = &[
            // Error (1.0)
            ("error", LineLevel::Error),
            ("exception", LineLevel::Error),
            ("fail", LineLevel::Error),
            ("failed", LineLevel::Error),
            ("failure", LineLevel::Error),
            ("fatal", LineLevel::Error),
            ("critical", LineLevel::Error),
            ("crash", LineLevel::Error),
            ("panic", LineLevel::Error),
            ("abort", LineLevel::Error),
            ("timeout", LineLevel::Error),
            // Security (0.85)
            ("vulnerability", LineLevel::Security),
            ("injection", LineLevel::Security),
            ("exploit", LineLevel::Security),
            ("breach", LineLevel::Security),
            ("denied", LineLevel::Security),
            ("rejected", LineLevel::Security),
            // Warning (0.75)
            ("warn", LineLevel::Warning),
            ("warning", LineLevel::Warning),
            ("deprecated", LineLevel::Warning),
            // Summary (0.4) — must come after Error keywords since "passed" can
            // coexist with "failed" in the same line; highest-priority wins.
            ("summary", LineLevel::Summary),
            ("total", LineLevel::Summary),
            ("result", LineLevel::Summary),
            ("passed", LineLevel::Summary),
            ("skipped", LineLevel::Summary),
        ];

        let keywords: Vec<&str> = pairs.iter().map(|(k, _)| *k).collect();
        let levels: Vec<LineLevel> = pairs.iter().map(|(_, l)| *l).collect();

        let ac = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .build(keywords)
            .expect("keyword patterns are compile-time constants");

        Self { ac, levels }
    }

    /// Classify a line into its highest-priority [`LineLevel`].
    ///
    /// When multiple keywords match, the highest-priority level wins
    /// (Error > Security > Warning > Summary > Info > Debug > StackTrace > Plain).
    ///
    /// Lines are first lowercased, then scanned by the Aho-Corasick
    /// automaton in a single pass.
    ///
    /// # Examples
    /// ```
    /// use rtco_core::keyword_detector::{KeywordDetector, LineLevel};
    /// let det = KeywordDetector::new();
    /// assert_eq!(det.classify_line("ERROR: connection refused"), LineLevel::Error);
    /// assert_eq!(det.classify_line("vulnerability found in auth"), LineLevel::Security);
    /// assert_eq!(det.classify_line("warning: unused variable"), LineLevel::Warning);
    /// assert_eq!(det.classify_line("hello world"), LineLevel::Plain);
    /// ```
    pub fn classify_line(&self, line: &str) -> LineLevel {
        let lower = line.to_lowercase();
        let mut best = LineLevel::Plain;

        for mat in self.ac.find_iter(&lower) {
            let level = self.levels[mat.pattern()];
            if level < best {
                best = level;
            }
        }

        best
    }

    /// Assign an importance score to a line based on keyword matches.
    ///
    /// Returns the score of the highest-priority keyword found:
    /// - Error: **1.0**
    /// - Security: **0.85**
    /// - Warning: **0.75**
    /// - Summary: **0.4**
    /// - Info: **0.3**
    /// - Debug: **0.1**
    /// - StackTrace: **0.2**
    /// - Plain (no match): **0.0**
    ///
    /// # Examples
    /// ```
    /// use rtco_core::keyword_detector::KeywordDetector;
    /// let det = KeywordDetector::new();
    /// assert_eq!(det.score_line("FATAL: out of memory"), 1.0);
    /// assert_eq!(det.score_line("All 42 tests passed"), 0.4);
    /// assert_eq!(det.score_line("Building project..."), 0.0);
    /// ```
    pub fn score_line(&self, line: &str) -> f64 {
        self.classify_line(line).score()
    }
}

impl Default for KeywordDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LineLevel {
    /// Numeric importance score (higher = more important to preserve).
    pub fn score(&self) -> f64 {
        match self {
            LineLevel::Error => 1.0,
            LineLevel::Security => 0.85,
            LineLevel::Warning => 0.75,
            LineLevel::Summary => 0.4,
            LineLevel::Info => 0.3,
            LineLevel::StackTrace => 0.2,
            LineLevel::Debug => 0.1,
            LineLevel::Plain => 0.0,
        }
    }

    /// Returns `true` if this level indicates an error condition.
    pub fn is_error(&self) -> bool {
        matches!(self, LineLevel::Error)
    }

    /// Returns `true` if this level is high-priority (Error or Security).
    pub fn is_critical(&self) -> bool {
        matches!(self, LineLevel::Error | LineLevel::Security)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- classify_line tests ---

    #[test]
    fn classify_error_keywords() {
        let det = KeywordDetector::new();
        let cases = [
            "ERROR: connection refused",
            "unhandled exception in handler",
            "test fail at line 42",
            "build failed with exit code 1",
            "assertion failure in test_foo",
            "FATAL: out of memory",
            "critical error in database",
            "segfault crash detected",
            "thread 'main' panicked at 'boom'",
            "process abort signal received",
            "connection timeout after 30s",
        ];
        for line in &cases {
            assert_eq!(
                det.classify_line(line),
                LineLevel::Error,
                "expected Error for: {}",
                line
            );
        }
    }

    #[test]
    fn classify_security_keywords() {
        let det = KeywordDetector::new();
        let cases = [
            "SQL injection detected in query",
            "known vulnerability CVE-2024-1234",
            "remote code exploit found",
            "data breach reported",
            "access denied for user root",
            "request rejected by firewall",
        ];
        for line in &cases {
            assert_eq!(
                det.classify_line(line),
                LineLevel::Security,
                "expected Security for: {}",
                line
            );
        }
    }

    #[test]
    fn classify_warning_keywords() {
        let det = KeywordDetector::new();
        let cases = [
            "warning: unused variable `x`",
            "WARN: deprecated API usage",
            "this function is deprecated",
        ];
        for line in &cases {
            assert_eq!(
                det.classify_line(line),
                LineLevel::Warning,
                "expected Warning for: {}",
                line
            );
        }
    }

    #[test]
    fn classify_summary_keywords() {
        let det = KeywordDetector::new();
        let cases = [
            "test summary: all good",
            "total: 100 files processed",
            "result: OK",
            "3 passed, 1 skipped",
            "5 skipped due to missing deps",
        ];
        for line in &cases {
            assert_eq!(
                det.classify_line(line),
                LineLevel::Summary,
                "expected Summary for: {}",
                line
            );
        }
    }

    #[test]
    fn classify_plain_line() {
        let det = KeywordDetector::new();
        assert_eq!(det.classify_line("Hello world"), LineLevel::Plain);
        assert_eq!(det.classify_line("Building project..."), LineLevel::Plain);
        assert_eq!(det.classify_line(""), LineLevel::Plain);
    }

    #[test]
    fn classify_highest_priority_wins() {
        let det = KeywordDetector::new();
        // Both "error" (Error=1.0) and "warning" (Warning=0.75) present
        assert_eq!(
            det.classify_line("error: deprecated function warning"),
            LineLevel::Error
        );
        // Both "security" (Security) and "failed" (Error) present
        assert_eq!(
            det.classify_line("vulnerability check failed"),
            LineLevel::Error
        );
        // Both "summary" and "warning"
        assert_eq!(
            det.classify_line("warning: test summary incomplete"),
            LineLevel::Warning
        );
        // Both "passed" (Summary) and "failed" (Error) — Error wins
        assert_eq!(
            det.classify_line("test summary: 42 passed, 0 failed"),
            LineLevel::Error
        );
    }

    #[test]
    fn classify_case_insensitive() {
        let det = KeywordDetector::new();
        assert_eq!(det.classify_line("ERROR: boom"), LineLevel::Error);
        assert_eq!(det.classify_line("error: boom"), LineLevel::Error);
        assert_eq!(det.classify_line("Error: boom"), LineLevel::Error);
        assert_eq!(det.classify_line("FATAL: oom"), LineLevel::Error);
        assert_eq!(det.classify_line("fatal: oom"), LineLevel::Error);
        assert_eq!(det.classify_line("WARNING: x"), LineLevel::Warning);
        assert_eq!(det.classify_line("Warning: x"), LineLevel::Warning);
    }

    // --- score_line tests ---

    #[test]
    fn score_error_line() {
        let det = KeywordDetector::new();
        assert_eq!(det.score_line("ERROR: something broke"), 1.0);
        assert_eq!(det.score_line("process crash detected"), 1.0);
    }

    #[test]
    fn score_security_line() {
        let det = KeywordDetector::new();
        assert!((det.score_line("SQL injection attempt") - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn score_warning_line() {
        let det = KeywordDetector::new();
        assert!((det.score_line("deprecated: use new_func instead") - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn score_summary_line() {
        let det = KeywordDetector::new();
        assert!((det.score_line("42 tests passed") - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn score_plain_line() {
        let det = KeywordDetector::new();
        assert_eq!(det.score_line("Building project..."), 0.0);
        assert_eq!(det.score_line(""), 0.0);
    }

    #[test]
    fn score_ordering() {
        let det = KeywordDetector::new();
        let error = det.score_line("FATAL crash");
        let security = det.score_line("vulnerability found");
        let warning = det.score_line("deprecated function");
        let summary = det.score_line("tests passed");
        let plain = det.score_line("hello");

        assert!(error > security, "error > security");
        assert!(security > warning, "security > warning");
        assert!(warning > summary, "warning > summary");
        assert!(summary > plain, "summary > plain");
    }

    // --- LineLevel method tests ---

    #[test]
    fn level_is_error() {
        assert!(LineLevel::Error.is_error());
        assert!(!LineLevel::Warning.is_error());
        assert!(!LineLevel::Plain.is_error());
    }

    #[test]
    fn level_is_critical() {
        assert!(LineLevel::Error.is_critical());
        assert!(LineLevel::Security.is_critical());
        assert!(!LineLevel::Warning.is_critical());
        assert!(!LineLevel::Summary.is_critical());
        assert!(!LineLevel::Plain.is_critical());
    }

    #[test]
    fn level_ordering() {
        assert!(LineLevel::Error < LineLevel::Security);
        assert!(LineLevel::Security < LineLevel::Warning);
        assert!(LineLevel::Warning < LineLevel::Summary);
        assert!(LineLevel::Summary < LineLevel::Info);
        assert!(LineLevel::Info < LineLevel::StackTrace);
        assert!(LineLevel::StackTrace < LineLevel::Debug);
        assert!(LineLevel::Debug < LineLevel::Plain);
    }

    // --- Default trait ---

    #[test]
    fn default_detector_works() {
        let det = KeywordDetector::default();
        assert_eq!(det.classify_line("ERROR: test"), LineLevel::Error);
        assert_eq!(det.classify_line("hello"), LineLevel::Plain);
    }
}
