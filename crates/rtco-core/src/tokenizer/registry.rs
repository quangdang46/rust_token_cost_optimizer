//! Tokenizer registry — backend selection and initialization.
//!
//! [`TokenizerRegistry`] holds the configured tokenizer and provides
//! a factory method to create it from a [`TokenizerKind`].

use super::approximate::ApproximateEstimator;
use super::{Tokenizer, TokenizerKind};

/// Registry that manages tokenizer backend selection.
///
/// Created once at startup, then shared across filter modules.
/// The registry holds a concrete [`Box<dyn Tokenizer>`] and delegates
/// `estimate()` calls to it.
#[derive(Debug)]
pub struct TokenizerRegistry {
    backend: Box<dyn Tokenizer>,
}

impl TokenizerRegistry {
    /// Create a registry with the specified backend.
    ///
    /// Falls back to [`ApproximateEstimator`] if the requested backend
    /// is not yet implemented (e.g. `TikToken` or `HuggingFace`).
    pub fn new(kind: TokenizerKind) -> Self {
        let backend: Box<dyn Tokenizer> = match kind {
            TokenizerKind::Approximate => Box::new(ApproximateEstimator::new()),
            // Placeholder: future tiktoken backend
            TokenizerKind::TikToken => {
                eprintln!(
                    "[rtco] tiktoken backend requested but not yet implemented; \
                     falling back to approximate"
                );
                Box::new(ApproximateEstimator::new())
            }
            // Placeholder: future HuggingFace backend
            TokenizerKind::HuggingFace => {
                eprintln!(
                    "[rtco] huggingface backend requested but not yet implemented; \
                     falling back to approximate"
                );
                Box::new(ApproximateEstimator::new())
            }
        };
        Self { backend }
    }

    /// Create a registry using the default backend (`Approximate`).
    pub fn with_default_backend() -> Self {
        Self::new(TokenizerKind::Approximate)
    }

    /// Estimate token count using the configured backend.
    pub fn estimate(&self, text: &str) -> usize {
        self.backend.estimate(text)
    }

    /// Return the name of the active backend.
    pub fn backend_name(&self) -> &str {
        self.backend.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::TokenizerKind;

    #[test]
    fn test_registry_default_approximate() {
        let reg = TokenizerRegistry::with_default_backend();
        assert_eq!(reg.backend_name(), "approximate");
    }

    #[test]
    fn test_registry_estimate_empty() {
        let reg = TokenizerRegistry::with_default_backend();
        assert_eq!(reg.estimate(""), 0);
    }

    #[test]
    fn test_registry_estimate_non_empty() {
        let reg = TokenizerRegistry::with_default_backend();
        let n = reg.estimate("hello world");
        assert!(n > 0, "should estimate >0 tokens for non-empty text");
    }

    #[test]
    fn test_registry_with_approximate_kind() {
        let reg = TokenizerRegistry::new(TokenizerKind::Approximate);
        assert_eq!(reg.backend_name(), "approximate");
    }

    #[test]
    fn test_registry_with_tiktoken_fallback() {
        let reg = TokenizerRegistry::new(TokenizerKind::TikToken);
        // Falls back to approximate
        assert_eq!(reg.backend_name(), "approximate");
    }

    #[test]
    fn test_registry_with_huggingface_fallback() {
        let reg = TokenizerRegistry::new(TokenizerKind::HuggingFace);
        assert_eq!(reg.backend_name(), "approximate");
    }

    #[test]
    fn test_registry_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // Box<dyn Tokenizer> requires Tokenizer: Send + Sync
        assert_send_sync::<TokenizerRegistry>();
    }

    #[test]
    fn test_registry_multiple_calls_consistent() {
        let reg = TokenizerRegistry::with_default_backend();
        let text = "The quick brown fox jumps over the lazy dog";
        let a = reg.estimate(text);
        let b = reg.estimate(text);
        assert_eq!(a, b, "estimate() should be deterministic");
    }
}
