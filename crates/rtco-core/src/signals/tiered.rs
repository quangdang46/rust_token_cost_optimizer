//! Composition-based line importance detection via tiered pipeline.
//!
//! [`TieredDetector`] runs multiple [`LineImportanceDetector`] instances
//! in order and returns the highest-confidence result.

use super::{ImportanceSignal, LineImportanceDetector, SignalContext};

/// Composes multiple detectors in a tiered pipeline.
///
/// Each detector is run in insertion order. The result with the highest
/// effective score (priority × confidence) is returned. If no detector
/// matches, falls back to a low-priority default signal.
///
/// This allows building sophisticated detection pipelines from simple
/// single-purpose detectors:
pub struct TieredDetector {
    detectors: Vec<Box<dyn LineImportanceDetector>>,
}

impl std::fmt::Debug for TieredDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TieredDetector")
            .field("detector_count", &self.detectors.len())
            .finish()
    }
}

impl Default for TieredDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl TieredDetector {
    /// Create an empty tiered detector.
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    /// Add a detector to the pipeline.
    ///
    /// Detectors are consulted in insertion order. If multiple detectors
    /// match, the one with the highest effective score wins.
    pub fn with_detector(mut self, detector: impl LineImportanceDetector + 'static) -> Self {
        self.detectors.push(Box::new(detector));
        self
    }

    /// Add a boxed detector to the pipeline.
    pub fn with_boxed_detector(mut self, detector: Box<dyn LineImportanceDetector>) -> Self {
        self.detectors.push(detector);
        self
    }

    /// Return the number of registered detectors.
    pub fn len(&self) -> usize {
        self.detectors.len()
    }

    /// Return `true` if no detectors are registered.
    pub fn is_empty(&self) -> bool {
        self.detectors.is_empty()
    }
}

impl LineImportanceDetector for TieredDetector {
    fn score(&self, line: &str, context: &SignalContext) -> Option<ImportanceSignal> {
        if self.detectors.is_empty() {
            return None;
        }

        let mut best: Option<ImportanceSignal> = None;
        let mut best_score: f64 = -1.0;

        for detector in &self.detectors {
            if let Some(signal) = detector.score(line, context) {
                let effective = signal.effective_score();
                if effective > best_score {
                    best_score = effective;
                    best = Some(signal);
                }
            }
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::detectors::{ErrorWarningDetector, SeparatorDetector};
    use crate::signals::SignalCategory;

    #[test]
    fn test_empty_detector_returns_none() {
        let detector = TieredDetector::new();
        let ctx = SignalContext::default();
        assert!(detector.score("anything", &ctx).is_none());
    }

    #[test]
    fn test_is_empty() {
        assert!(TieredDetector::new().is_empty());
        assert!(!TieredDetector::default()
            .with_detector(ErrorWarningDetector::new())
            .is_empty());
    }

    #[test]
    fn test_len() {
        let det = TieredDetector::default()
            .with_detector(ErrorWarningDetector::new())
            .with_detector(SeparatorDetector::new());
        assert_eq!(det.len(), 2);
    }

    #[test]
    fn test_single_detector() {
        let detector = TieredDetector::default().with_detector(ErrorWarningDetector::new());
        let ctx = SignalContext::default();

        let sig = detector.score("error: something broke", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Error);

        assert!(detector.score("regular text", &ctx).is_none());
    }

    #[test]
    fn test_tiered_picks_highest_score() {
        let detector = TieredDetector::default()
            .with_detector(ErrorWarningDetector::new())
            .with_detector(SeparatorDetector::new());
        let ctx = SignalContext::default();

        // Error line should get error category from ErrorWarningDetector
        let sig = detector.score("error: fatal crash", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Error);

        // Separator line should get separator category
        let sig = detector.score("---------------------------", &ctx).unwrap();
        assert_eq!(sig.category, SignalCategory::Separator);
    }

    #[test]
    fn test_tiered_no_match() {
        let detector = TieredDetector::default()
            .with_detector(ErrorWarningDetector::new())
            .with_detector(SeparatorDetector::new());
        let ctx = SignalContext::default();

        assert!(
            detector.score("just regular text content", &ctx).is_none(),
            "plain text should not match any detector"
        );
    }

    #[test]
    fn test_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TieredDetector>();
    }
}
