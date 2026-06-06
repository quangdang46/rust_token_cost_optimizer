//! Built-in [`OffloadTransform`](super::OffloadTransform) implementations.
//!
//! Offload transforms drop low-value content and store originals in CCR
//! for potential recovery.

use super::OffloadTransform;
use crate::ccr::{self, CcrStore};
use crate::signals::ImportanceSignal;
use anyhow::Result;

/// Offloads lines longer than a configurable character threshold.
///
/// Very long lines consume disproportionate token budget — store them
/// in CCR and emit a short summary marker instead.
#[derive(Debug)]
pub struct LengthOffloader {
    /// Lines longer than this many characters will be offloaded.
    pub threshold_chars: usize,
}

impl OffloadTransform for LengthOffloader {
    fn name(&self) -> &str {
        "length_offloader"
    }

    fn estimate_bloat(&self, input: &str, _signal: &Option<ImportanceSignal>) -> bool {
        input.len() > self.threshold_chars
    }

    fn apply(&self, input: &str, store: &dyn CcrStore) -> Result<String> {
        let key = ccr::compute_key(input.as_bytes());
        store.put(&key, input.as_bytes())?;
        Ok(format!("[offloaded: {} bytes]", input.len()))
    }
}

/// Offloads lines whose importance signal falls below a threshold.
///
/// Lines with very low priority (separators, noise, empty lines) are
/// dropped entirely after storing in CCR.
#[derive(Debug)]
pub struct LowPriorityOffloader {
    /// Lines with effective score below this threshold will be offloaded.
    pub threshold: f64,
}

impl OffloadTransform for LowPriorityOffloader {
    fn name(&self) -> &str {
        "low_priority_offloader"
    }

    fn estimate_bloat(&self, _input: &str, signal: &Option<ImportanceSignal>) -> bool {
        match signal {
            Some(s) => s.effective_score() < self.threshold,
            None => false,
        }
    }

    fn apply(&self, input: &str, store: &dyn CcrStore) -> Result<String> {
        let key = ccr::compute_key(input.as_bytes());
        store.put(&key, input.as_bytes())?;
        // Return empty — line is fully dropped
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ccr::InMemoryCcrStore;
    use crate::signals::{ImportanceSignal, SignalCategory};

    // ── LengthOffloader ─────────────────────────────────────────────

    #[test]
    fn test_length_offloader_estimate_short_line() {
        let offloader = LengthOffloader {
            threshold_chars: 100,
        };
        assert!(!offloader.estimate_bloat("short", &None));
    }

    #[test]
    fn test_length_offloader_estimate_long_line() {
        let offloader = LengthOffloader {
            threshold_chars: 10,
        };
        assert!(offloader.estimate_bloat("this is a long line that exceeds threshold", &None));
    }

    #[test]
    fn test_length_offloader_apply_stores_in_ccr() {
        let store = InMemoryCcrStore::new();
        let offloader = LengthOffloader {
            threshold_chars: 10,
        };
        let input = "very long line content that should be stored";
        let result = offloader.apply(input, &store).unwrap();

        assert!(
            result.contains("offloaded"),
            "Should produce marker: {}",
            result
        );
        assert!(!store.is_empty(), "CCR should have stored content");
    }

    // ── LowPriorityOffloader ────────────────────────────────────────

    #[test]
    fn test_low_priority_offloader_high_signal() {
        let offloader = LowPriorityOffloader { threshold: 0.5 };
        let signal = ImportanceSignal {
            category: SignalCategory::Error,
            priority: 1.0,
            confidence: 0.95,
        };
        assert!(!offloader.estimate_bloat("error!", &Some(signal)));
    }

    #[test]
    fn test_low_priority_offloader_low_signal() {
        let offloader = LowPriorityOffloader { threshold: 0.5 };
        let signal = ImportanceSignal {
            category: SignalCategory::Plain,
            priority: 0.1,
            confidence: 0.9,
        };
        assert!(offloader.estimate_bloat("noise", &Some(signal)));
    }

    #[test]
    fn test_low_priority_offloader_no_signal() {
        let offloader = LowPriorityOffloader { threshold: 0.5 };
        assert!(!offloader.estimate_bloat("anything", &None));
    }

    #[test]
    fn test_low_priority_offloader_apply_returns_empty() {
        let store = InMemoryCcrStore::new();
        let offloader = LowPriorityOffloader { threshold: 0.5 };
        let result = offloader.apply("dropped line", &store).unwrap();
        assert_eq!(
            result, "",
            "Low priority offload should return empty string"
        );
    }
}
