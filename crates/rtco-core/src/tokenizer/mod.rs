//! Token estimation for LLM context window optimization.
//!
//! Provides a pluggable [`Tokenizer`] trait with multiple backends:
//! - [`ApproximateEstimator`]: fast heuristic (chars/4), zero dependencies
//! - TikToken backend: planned for tiktoken-rs integration
//! - HuggingFace backend: planned for tokenizers crate integration
//!
//! A [`TokenizerRegistry`] manages backend selection and auto-detection.

mod approximate;
mod registry;

pub use approximate::ApproximateEstimator;
pub use registry::TokenizerRegistry;

/// Pluggable token estimation backend.
///
/// All implementations must be `Send + Sync` so the registry can be
/// shared across filters running in parallel.
pub trait Tokenizer: Send + Sync + std::fmt::Debug {
    /// Estimate the number of tokens in `text`.
    ///
    /// Implementations balance speed against accuracy.  The approximate
    /// backend runs in <1µs; more accurate backends (tiktoken, HF) may
    /// take 10–50µs.
    fn estimate(&self, text: &str) -> usize;

    /// Human-readable name of this backend (e.g. `"approximate"`).
    fn name(&self) -> &str;
}

/// Discriminant for selecting a tokenizer backend at config time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TokenizerKind {
    /// Character-count heuristic (`text.len() / 3.5`), always available.
    #[default]
    Approximate,
    /// Placeholder for future tiktoken-rs integration.
    TikToken,
    /// Placeholder for future HuggingFace tokenizers integration.
    HuggingFace,
}

impl TokenizerKind {
    /// Parse a backend name from a config string.
    ///
    /// Accepts `"approximate"`, `"tiktoken"`, or `"huggingface"` (case-insensitive).
    /// Returns `Approximate` for unknown values as a safe default.
    pub fn from_config(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "tiktoken" => Self::TikToken,
            "huggingface" | "hf" => Self::HuggingFace,
            _ => Self::Approximate,
        }
    }

    /// Return the canonical config string for this kind.
    pub fn as_config_str(&self) -> &'static str {
        match self {
            Self::Approximate => "approximate",
            Self::TikToken => "tiktoken",
            Self::HuggingFace => "huggingface",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_from_config_approximate() {
        assert_eq!(
            TokenizerKind::from_config("approximate"),
            TokenizerKind::Approximate
        );
        assert_eq!(
            TokenizerKind::from_config("APPROXIMATE"),
            TokenizerKind::Approximate
        );
        assert_eq!(
            TokenizerKind::from_config("unknown"),
            TokenizerKind::Approximate
        );
        assert_eq!(TokenizerKind::from_config(""), TokenizerKind::Approximate);
    }

    #[test]
    fn test_kind_from_config_tiktoken() {
        assert_eq!(
            TokenizerKind::from_config("tiktoken"),
            TokenizerKind::TikToken
        );
        assert_eq!(
            TokenizerKind::from_config("TikToken"),
            TokenizerKind::TikToken
        );
    }

    #[test]
    fn test_kind_from_config_huggingface() {
        assert_eq!(
            TokenizerKind::from_config("huggingface"),
            TokenizerKind::HuggingFace
        );
        assert_eq!(TokenizerKind::from_config("hf"), TokenizerKind::HuggingFace);
    }

    #[test]
    fn test_kind_as_config_str() {
        assert_eq!(TokenizerKind::Approximate.as_config_str(), "approximate");
        assert_eq!(TokenizerKind::TikToken.as_config_str(), "tiktoken");
        assert_eq!(TokenizerKind::HuggingFace.as_config_str(), "huggingface");
    }

    #[test]
    fn test_kind_default() {
        assert_eq!(TokenizerKind::default(), TokenizerKind::Approximate);
    }
}
