//! Error keywords for item preservation during compression.
//!
//! Ported from headroom's `transforms/smart_crusher/error_keywords.rs`.
//! These keywords are matched case-insensitively against JSON-serialized
//! items to identify error items that must never be dropped.

/// 12 error/failure keywords. Lowercase by construction; callers must
/// lowercase the haystack before substring-matching.
pub const ERROR_KEYWORDS: &[&str] = &[
    "error",
    "exception",
    "failed",
    "failure",
    "critical",
    "fatal",
    "crash",
    "panic",
    "abort",
    "timeout",
    "denied",
    "rejected",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_headroom_count() {
        assert_eq!(ERROR_KEYWORDS.len(), 12);
    }

    #[test]
    fn test_all_lowercase() {
        for &kw in ERROR_KEYWORDS {
            assert_eq!(kw.to_lowercase(), kw);
        }
    }

    #[test]
    fn test_pinned_keywords() {
        let expected = [
            "error",
            "exception",
            "failed",
            "failure",
            "critical",
            "fatal",
            "crash",
            "panic",
            "abort",
            "timeout",
            "denied",
            "rejected",
        ];
        let mut actual: Vec<&str> = ERROR_KEYWORDS.to_vec();
        actual.sort();
        let mut expected_sorted = expected.to_vec();
        expected_sorted.sort();
        assert_eq!(actual, expected_sorted);
    }
}
