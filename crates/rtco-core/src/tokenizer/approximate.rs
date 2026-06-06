//! Approximate token estimator — fast heuristic, zero dependencies.
//!
//! Uses a combination of character-count division and whitespace-aware
//! adjustment to estimate token count without any external tokenizer.
//! Based on observed ratios across multiple LLM tokenizers:
//!
//! - English/prose text: ~3.5–4 chars per token
//! - Code/text with many symbols: ~3 chars per token
//! - Dense text (minified JSON): ~2–3 chars per token
//!
//! The estimator blends a char-based baseline with a whitespace-token
//! baseline to handle both extremes reasonably.

use super::Tokenizer;

/// Fast approximate token counter using heuristic formulas.
///
/// Estimates tokens as the average of two computed values:
/// 1. `chars / 3.5` — the standard English prose heuristic
/// 2. `whitespace_tokens * 1.3` — a adjusted word count
///
/// This provides a reasonable middle-ground for mixed CLI output
/// containing code, prose, JSON, and structured text.
///
/// # Accuracy
///
/// | Content type  | Error vs tiktoken |
/// |---------------|-------------------|
/// | English prose | ±10–20%           |
/// | Code          | ±15–30%           |
/// | JSON          | ±20–40%           |
/// | Logs          | ±10–25%           |
///
/// For production use, prefer tiktoken or HF backends when available.
/// This backend exists for zero-dependency bootstrap.
#[derive(Debug, Clone, Copy)]
pub struct ApproximateEstimator;

impl ApproximateEstimator {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for ApproximateEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for ApproximateEstimator {
    /// Estimate token count using a blend of char-based and word-based heuristics.
    fn estimate(&self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let char_count = text.chars().count() as f64;
        let word_count = text.split_whitespace().count() as f64;

        // Heuristic 1: chars / 3.5 (standard English prose ratio)
        let from_chars = char_count / 3.5;

        // Heuristic 2: word_count * 1.3 (average tokens per word)
        let from_words = word_count * 1.3;

        // Blend: average of both heuristics, with a minimum of 1
        let estimated = (from_chars + from_words) / 2.0;
        (estimated.ceil() as usize).max(1)
    }

    fn name(&self) -> &str {
        "approximate"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let est = ApproximateEstimator::new();
        assert_eq!(est.estimate(""), 0);
    }

    #[test]
    fn test_single_word() {
        let est = ApproximateEstimator::new();
        let n = est.estimate("hello");
        assert!(n >= 1, "single word should have at least 1 token");
    }

    #[test]
    fn test_short_sentence() {
        let est = ApproximateEstimator::new();
        // "Hello world example text" → ~20 chars / 3.5 ≈ 5.7 → blended
        let n = est.estimate("Hello world example text");
        assert!(
            (3..=10).contains(&n),
            "short sentence: expected 3-10, got {}",
            n
        );
    }

    #[test]
    fn test_longer_text() {
        let est = ApproximateEstimator::new();
        let text = "This is a longer piece of English prose that should have a \
                     reasonable number of tokens for testing purposes";
        let n = est.estimate(text);
        assert!(
            n >= 10,
            "longer text should have at least 10 tokens, got {}",
            n
        );
    }

    #[test]
    fn test_code_snippet() {
        let est = ApproximateEstimator::new();
        let code = "fn main() {\n    println!(\"Hello\");\n}\n";
        let n = est.estimate(code);
        assert!(n >= 3, "code should have at least 3 tokens, got {}", n);
    }

    #[test]
    fn test_json_text() {
        let est = ApproximateEstimator::new();
        let json = r#"{"name":"test","values":[1,2,3,4,5]}"#;
        let n = est.estimate(json);
        assert!(n >= 5, "JSON should have at least 5 tokens, got {}", n);
    }

    #[test]
    fn test_whitespace_only() {
        let est = ApproximateEstimator::new();
        assert!(
            est.estimate("   \n   \t   ") >= 1,
            "whitespace should yield 1 token"
        );
    }

    #[test]
    fn test_name() {
        let est = ApproximateEstimator::new();
        assert_eq!(est.name(), "approximate");
    }

    #[test]
    fn test_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ApproximateEstimator>();
    }

    #[test]
    fn test_backward_compat_with_whitespace_split() {
        let est = ApproximateEstimator::new();
        let text = "hello world";
        let ws_count = text.split_whitespace().count(); // 2
        let est_count = est.estimate(text);
        // Approximate should give at least as many tokens as whitespace split
        // (since words * 1.3 and chars/3.5 both produce larger numbers for normal text)
        assert!(
            est_count >= ws_count,
            "approx should estimate >= whitespace count, got {} < {}",
            est_count,
            ws_count
        );
    }
}
