//! Line importance scoring signals for intelligent output truncation.
//!
//! Provides a trait-based system for scoring lines of command output by
//! their semantic importance.  Built-in detectors cover error/warning
//! patterns, separator lines, and line-length heuristics.
//!
//! Detectors can be composed via [`TieredDetector`] for multi-pass
//! scoring.

pub mod detectors;
pub mod tiered;

pub use detectors::{ErrorWarningDetector, LengthDetector, SeparatorDetector};
pub use tiered::TieredDetector;

/// Category of an importance signal, ordered by semantic priority.
///
/// Variants are listed from highest priority to lowest.  The ordering
/// guides truncation: lines with higher-priority categories are kept
/// before those with lower-priority categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SignalCategory {
    /// Fatal or error-level messages (must-keep).
    Error,
    /// Warning or deprecation messages.
    Warning,
    /// Summary/statistics lines (test results, build summaries).
    Summary,
    /// Stack trace frames.
    StackTrace,
    /// Informational messages.
    Info,
    /// Debug or trace-level messages (low value).
    Debug,
    /// Structural markers (separators, section headers).
    Separator,
    /// Plain text with no distinguishing features (noise).
    Plain,
}

impl SignalCategory {
    /// Return a numeric priority for sorting (higher = more important).
    ///
    /// Used by selection algorithms to decide which lines to keep first.
    pub fn priority(&self) -> f64 {
        match self {
            Self::Error => 1.0,
            Self::Warning => 0.8,
            Self::Summary => 0.7,
            Self::StackTrace => 0.6,
            Self::Info => 0.4,
            Self::Separator => 0.3,
            Self::Debug => 0.2,
            Self::Plain => 0.0,
        }
    }
}

/// A scored signal for a single line of output.
#[derive(Debug, Clone)]
pub struct ImportanceSignal {
    /// Semantic category of this line.
    pub category: SignalCategory,
    /// Composite importance score (0.0–1.0).
    ///
    /// Higher means the line is more important to preserve during
    /// truncation.
    pub priority: f64,
    /// Confidence in the signal (0.0–1.0).
    ///
    /// High confidence means the detector is certain about its
    /// classification; low confidence means it's guessing.
    pub confidence: f64,
}

impl ImportanceSignal {
    /// Create a new importance signal.
    pub fn new(category: SignalCategory, priority: f64, confidence: f64) -> Self {
        Self {
            category,
            priority: priority.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Return the effective score (priority × confidence).
    ///
    /// This is the value used for line selection comparisons.
    pub fn effective_score(&self) -> f64 {
        self.priority * self.confidence
    }

    /// Signal for an unremarkable plain-text line.
    pub fn plain() -> Self {
        Self::new(SignalCategory::Plain, 0.0, 1.0)
    }
}

/// Contextual information for line importance detection.
///
/// Provides detectors with metadata about the line's position and
/// the overall content type to improve classification accuracy.
#[derive(Debug, Clone)]
pub struct SignalContext {
    /// The content type of the overall output (e.g. GitDiff, Log, Json).
    pub content_type: crate::ContentType,
    /// Zero-based line number within the output.
    pub line_number: usize,
}

impl SignalContext {
    /// Create a new signal context.
    pub fn new(content_type: crate::ContentType, line_number: usize) -> Self {
        Self {
            content_type,
            line_number,
        }
    }
}

impl Default for SignalContext {
    fn default() -> Self {
        Self {
            content_type: crate::ContentType::PlainText,
            line_number: 0,
        }
    }
}

/// Trait for line importance detectors.
///
/// Implementations analyse a single line and return an
/// [`ImportanceSignal`] if they can classify the line, or `None`
/// if the line doesn't match their detection criteria.
///
/// Multiple detectors can be composed via [`TieredDetector`].
pub trait LineImportanceDetector: Send + Sync + std::fmt::Debug {
    /// Score a single line, returning a signal if recognised.
    fn score(&self, line: &str, context: &SignalContext) -> Option<ImportanceSignal>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_category_priority_ordering() {
        assert!(SignalCategory::Error.priority() > SignalCategory::Warning.priority());
        assert!(SignalCategory::Warning.priority() > SignalCategory::Summary.priority());
        assert!(SignalCategory::Summary.priority() > SignalCategory::Info.priority());
        assert!(SignalCategory::Info.priority() > SignalCategory::Separator.priority());
        assert!(SignalCategory::Separator.priority() > SignalCategory::Debug.priority());
        assert!(SignalCategory::Debug.priority() > SignalCategory::Plain.priority());
    }

    #[test]
    fn test_importance_signal_new_clamps_values() {
        let s = ImportanceSignal::new(SignalCategory::Error, 1.5, -0.5);
        assert!(
            (s.priority - 1.0).abs() < f64::EPSILON,
            "priority should clamp to 1.0"
        );
        assert!(
            (s.confidence - 0.0).abs() < f64::EPSILON,
            "confidence should clamp to 0.0"
        );
    }

    #[test]
    fn test_effective_score() {
        let s = ImportanceSignal::new(SignalCategory::Error, 0.8, 0.9);
        assert!((s.effective_score() - 0.72).abs() < f64::EPSILON);
    }

    #[test]
    fn test_plain_signal() {
        let s = ImportanceSignal::plain();
        assert_eq!(s.category, SignalCategory::Plain);
        assert!((s.priority - 0.0).abs() < f64::EPSILON);
        assert!((s.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_signal_context_default() {
        let ctx = SignalContext::default();
        assert_eq!(ctx.content_type, crate::ContentType::PlainText);
        assert_eq!(ctx.line_number, 0);
    }

    #[test]
    fn test_signal_context_new() {
        let ctx = SignalContext::new(crate::ContentType::GitDiff, 42);
        assert_eq!(ctx.content_type, crate::ContentType::GitDiff);
        assert_eq!(ctx.line_number, 42);
    }
}
