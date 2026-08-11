//! Built-in line importance detectors.
//!
//! Provides concrete implementations of [`LineImportanceDetector`]:
//!
//! - [`ErrorWarningDetector`]: error/warning pattern matching
//! - [`SeparatorDetector`]: structural separator lines
//! - [`LengthDetector`]: line-length heuristic

use std::sync::LazyLock;

use super::{ImportanceSignal, LineImportanceDetector, SignalCategory, SignalContext};
use aho_corasick::{AhoCorasick, AhoCorasickBuilder};

// ---------------------------------------------------------------------------
// Error/Warning pattern detector
// ---------------------------------------------------------------------------

/// Detects error, warning, failure, and fatal lines using keyword patterns.
///
/// Priority boosts:
///
/// | Pattern | Category  | Priority | Confidence |
/// |---------|-----------|----------|------------|
/// | Fatal/panic | Error | 1.0 | 0.98 |
/// | Error/fail | Error | 0.95 | 0.95 |
/// | Exception | Error | 0.90 | 0.90 |
/// | Warning/warn | Warning | 0.80 | 0.85 |
/// | Deprecated | Warning | 0.75 | 0.80 |
///
/// Uses `AhoCorasick` for fast case-insensitive multi-pattern matching.
#[derive(Debug, Clone)]
pub struct ErrorWarningDetector;

impl ErrorWarningDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ErrorWarningDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Case-insensitive pattern sets built once via LazyLock.
struct ErrorPatterns {
    /// Substrings that indicate a fatal/critical condition.
    fatal: AhoCorasick,
    /// Exception class names.
    exceptions: AhoCorasick,
    /// Failure indicators.
    failure: AhoCorasick,
    /// Warning indicators.
    warning: AhoCorasick,
    /// Deprecation indicators.
    deprecated: AhoCorasick,
}

fn build_ci_ac(patterns: &[&str]) -> AhoCorasick {
    AhoCorasickBuilder::new()
        .ascii_case_insensitive(true)
        .build(patterns)
        .expect("valid AhoCorasick patterns")
}

// Because AhoCorasick can't express line-start anchors but does case-insensitive
// substring matching, we split the problem:
//   - Substring patterns (like "FATAL", "FAILED") → AhoCorasick (fast)
//   - Line-start patterns (like "error:" as first word) → checked with str methods
const ERROR_START_PATTERNS: &[&str] = &[
    "error:",
    "fatal:",
    "panic:",
    "critical:",
    "error]",
    "fatal]",
    "panic]",
    "critical]",
];

static ERROR_PATTERNS: LazyLock<ErrorPatterns> = LazyLock::new(|| ErrorPatterns {
    fatal: build_ci_ac(&["FATAL", "PANIC", "CRITICAL"]),
    exceptions: build_ci_ac(&["exception", "errorcode", "errno"]),
    failure: build_ci_ac(&["failed", " FAIL ", " fail ", "FAILED", "failure"]),
    warning: build_ci_ac(&["warning", "WARN"]),
    deprecated: build_ci_ac(&["deprecated"]),
});

/// Check if a line starts with any of the given case-insensitive patterns.
fn line_starts_with_any(line: &str, patterns: &[&str]) -> bool {
    let lower = line.to_lowercase();
    patterns.iter().any(|p| lower.starts_with(p))
}

impl LineImportanceDetector for ErrorWarningDetector {
    fn score(&self, line: &str, _context: &SignalContext) -> Option<ImportanceSignal> {
        // 1. Fatal/panic/critical — check both start-of-line and substring
        if line_starts_with_any(line, &["fatal:", "panic:", "critical:"])
            || ERROR_PATTERNS.fatal.is_match(line)
        {
            return Some(ImportanceSignal::new(SignalCategory::Error, 1.0, 0.98));
        }

        // 2. Error indicators — line starts with "error:" / "Error]"
        if line_starts_with_any(line, ERROR_START_PATTERNS) {
            return Some(ImportanceSignal::new(SignalCategory::Error, 0.95, 0.95));
        }

        // 3. Exception type names (ValueError, TypeError, etc.)
        if ERROR_PATTERNS.exceptions.is_match(line) {
            return Some(ImportanceSignal::new(SignalCategory::Error, 0.90, 0.90));
        }

        // 4. Failure indicators
        if ERROR_PATTERNS.failure.is_match(line) {
            return Some(ImportanceSignal::new(SignalCategory::Error, 0.85, 0.85));
        }

        // 5. Warning indicators
        if ERROR_PATTERNS.warning.is_match(line) {
            return Some(ImportanceSignal::new(SignalCategory::Warning, 0.80, 0.85));
        }

        // 6. Deprecation
        if ERROR_PATTERNS.deprecated.is_match(line) {
            return Some(ImportanceSignal::new(SignalCategory::Warning, 0.75, 0.80));
        }

        None
    }
}

// ---------------------------------------------------------------------------
// Separator line detector
// ---------------------------------------------------------------------------

/// Detects structural separator lines (dashes, equals, asterisks, etc.).
///
/// These are lines composed primarily of repeated separator characters
/// with no semantic content. Priority is intentionally low so they
/// are removed early during aggressive truncation.
#[derive(Debug, Clone)]
pub struct SeparatorDetector {
    /// Minimum ratio of separator characters to total length (default: 0.8).
    threshold: f64,
}

impl SeparatorDetector {
    pub fn new() -> Self {
        Self { threshold: 0.8 }
    }

    /// Create a detector with a custom separator ratio threshold.
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            threshold: threshold.clamp(0.0, 1.0),
        }
    }
}

impl Default for SeparatorDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LineImportanceDetector for SeparatorDetector {
    fn score(&self, line: &str, _context: &SignalContext) -> Option<ImportanceSignal> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let chars: Vec<char> = trimmed.chars().collect();
        if chars.len() < 3 {
            return None;
        }

        // Count separator characters
        let separator_count = chars
            .iter()
            .filter(|c| matches!(c, '-' | '=' | '*' | '_' | '~' | '#' | '.' | '—' | '─'))
            .count();

        let ratio = separator_count as f64 / chars.len() as f64;
        if ratio >= self.threshold {
            return Some(ImportanceSignal::new(
                SignalCategory::Separator,
                SignalCategory::Separator.priority(),
                ratio,
            ));
        }

        None
    }
}

// ---------------------------------------------------------------------------
// Line length detector
// ---------------------------------------------------------------------------

/// Scores lines by their length — very short and very long lines are
/// considered lower importance.
///
/// - Very short lines (<10 chars after trim): lower confidence
/// - Lines of moderate length (30–200 chars): higher priority
/// - Very long lines (>200 chars): lower confidence (often noise or data)
#[derive(Debug, Clone, Default)]
pub struct LengthDetector;

impl LengthDetector {
    pub fn new() -> Self {
        Self
    }
}

impl LineImportanceDetector for LengthDetector {
    fn score(&self, line: &str, _context: &SignalContext) -> Option<ImportanceSignal> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let len = trimmed.len();
        let priority = if len < 10 {
            0.2
        } else if len < 30 {
            0.4
        } else if len < 100 {
            0.6
        } else if len < 200 {
            0.5
        } else {
            0.3
        };

        // Confidence decreases at extremes (too short = probably noise,
        // too long = probably data)
        let confidence = if len < 10 {
            0.3
        } else if len < 30 {
            0.5
        } else if len < 200 {
            0.7
        } else {
            0.4
        };

        Some(ImportanceSignal::new(
            SignalCategory::Info,
            priority,
            confidence,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ErrorWarningDetector ──────────────────────────────────────────

    #[test]
    fn test_error_detector_fatal() {
        let det = ErrorWarningDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("FATAL: kernel panic", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Error);
        assert!(sig.priority > 0.95);
    }

    #[test]
    fn test_error_detector_error_prefix() {
        let det = ErrorWarningDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("error: connection refused", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Error);
    }

    #[test]
    fn test_error_detector_error_colon_inline() {
        let det = ErrorWarningDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("  ERROR: something failed", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Error);
    }

    #[test]
    fn test_error_detector_exception() {
        let det = ErrorWarningDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("Exception: cannot convert", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Error);
    }

    #[test]
    fn test_error_detector_warning() {
        let det = ErrorWarningDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("warning: unused variable", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Warning);
    }

    #[test]
    fn test_error_detector_deprecated() {
        let det = ErrorWarningDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("This function is deprecated", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Warning);
    }

    #[test]
    fn test_error_detector_no_match() {
        let det = ErrorWarningDetector::new();
        let ctx = SignalContext::default();
        assert!(det.score("just some text", &ctx).is_none());
    }

    #[test]
    fn test_error_detector_empty_line() {
        let det = ErrorWarningDetector::new();
        let ctx = SignalContext::default();
        assert!(det.score("", &ctx).is_none());
    }

    #[test]
    fn test_error_detector_failed_pattern() {
        let det = ErrorWarningDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("FAILED: test_foo_bar", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Error);
    }

    // ── SeparatorDetector ─────────────────────────────────────────────

    #[test]
    fn test_separator_dashes() {
        let det = SeparatorDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("----------------------------", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Separator);
    }

    #[test]
    fn test_separator_equals() {
        let det = SeparatorDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("============================", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Separator);
    }

    #[test]
    fn test_separator_too_short() {
        let det = SeparatorDetector::new();
        let ctx = SignalContext::default();
        assert!(
            det.score("--", &ctx).is_none(),
            "too short to be a separator"
        );
    }

    #[test]
    fn test_separator_not_separator() {
        let det = SeparatorDetector::new();
        let ctx = SignalContext::default();
        assert!(
            det.score("This is regular text with some - dashes", &ctx)
                .is_none(),
            "regular text should not match"
        );
    }

    #[test]
    fn test_separator_asterisks() {
        let det = SeparatorDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("***********", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Separator);
    }

    #[test]
    fn test_separator_empty_line() {
        let det = SeparatorDetector::new();
        let ctx = SignalContext::default();
        assert!(det.score("", &ctx).is_none());
    }

    // ── LengthDetector ────────────────────────────────────────────────

    #[test]
    fn test_length_medium_line() {
        let det = LengthDetector::new();
        let ctx = SignalContext::default();
        let sig = det
            .score(
                "This is a medium-length line of text with some content",
                &ctx,
            )
            .unwrap();
        assert!(
            sig.priority > 0.5,
            "medium line should have decent priority"
        );
    }

    #[test]
    fn test_length_short_line() {
        let det = LengthDetector::new();
        let ctx = SignalContext::default();
        let sig = det.score("hi", &ctx).unwrap();
        assert!(sig.priority < 0.3, "short line should have low priority");
    }

    #[test]
    fn test_length_very_long_line() {
        let det = LengthDetector::new();
        let ctx = SignalContext::default();
        let long_line = "x".repeat(300);
        let sig = det.score(&long_line, &ctx).unwrap();
        assert!(
            sig.priority < 0.5,
            "very long line should have reduced priority"
        );
    }

    #[test]
    fn test_length_empty() {
        let det = LengthDetector::new();
        let ctx = SignalContext::default();
        assert!(det.score("", &ctx).is_none());
    }

    // ── Send + Sync ───────────────────────────────────────────────────

    #[test]
    fn test_detectors_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ErrorWarningDetector>();
        assert_send_sync::<SeparatorDetector>();
        assert_send_sync::<LengthDetector>();
    }
}
