//! Traits and types for the configurable compression pipeline.
//!
//! The pipeline has two phases:
//! 1. **Reformat** (serial) — pack data denser without dropping content.
//! 2. **Offload** — score lines, select within budget, optionally store in CCR.
//!
//! Pipeline is optional — existing filters continue to work independently.
//! Filters opt in by calling [`compress_with_pipeline`] or building a
//! [`CompressionPipeline`] directly.

pub mod config;
mod offload;
mod reformat;
mod runner;

pub use config::PipelineConfig;
pub use offload::{LengthOffloader, LowPriorityOffloader};
pub use reformat::{JsonMinifier, LineCollapser};
pub use runner::compress_with_pipeline;

use crate::ccr::CcrStore;
use crate::signals::{ImportanceSignal, LineImportanceDetector, SignalContext};
use crate::tokenizer::Tokenizer;
use anyhow::Result;
use std::fmt::Debug;
use std::sync::Arc;

/// A transform that makes output more compact without dropping content.
///
/// Applied **serially** in registration order.
pub trait ReformatTransform: Send + Sync + Debug {
    /// Human-readable name for diagnostics and config.
    fn name(&self) -> &str;

    /// Apply the reformat transform.
    ///
    /// Must never drop semantic content — only compress whitespace,
    /// collapse repeated patterns, or otherwise pack denser.
    fn reformat(&self, input: &str) -> Result<String>;

    /// Estimated token savings ratio (0.0–1.0) for display / planning.
    fn estimated_savings(&self) -> f64;
}

/// A transform that drops low-value content and stores it in CCR.
///
/// Applied **after** reformatting and scoring. Lines are offloaded
/// when their bloat estimate exceeds the configured threshold.
pub trait OffloadTransform: Send + Sync + Debug {
    /// Human-readable name.
    fn name(&self) -> &str;

    /// Whether the given input should be offloaded.
    fn estimate_bloat(&self, input: &str, signal: &Option<ImportanceSignal>) -> bool;

    /// Apply the offload: store original in CCR and return a replacement string.
    ///
    /// The replacement may be a short marker, a summary, or empty (fully dropped).
    fn apply(&self, input: &str, store: &dyn CcrStore) -> Result<String>;
}

/// The compression pipeline orchestrator.
///
/// Combines reformat transforms (serial) with signal scoring and
/// budget-constrained line selection to produce compressed output.
///
/// # Example
/// ```ignore
/// use rtco_core::pipeline::{CompressionPipeline, PipelineConfig};
///
/// let config = PipelineConfig { max_tokens: 100, ..Default::default() };
/// let pipeline = CompressionPipeline::default_with_config(config);
/// let compressed = pipeline.run("some long output", ContentType::PlainText).unwrap();
/// ```
#[derive(Debug)]
pub struct CompressionPipeline {
    /// Pipeline configuration.
    pub config: PipelineConfig,
    /// Registered reformat transforms (applied in registration order).
    pub reformatters: Vec<Box<dyn ReformatTransform>>,
    /// Registered offload transforms.
    pub offloaders: Vec<Box<dyn OffloadTransform>>,
    /// Tokenizer for budget estimation.
    pub tokenizer: Option<Box<dyn Tokenizer>>,
    /// CCR store for offloaded content.
    pub ccr_store: Option<Arc<dyn CcrStore>>,
    /// Signal detector for line scoring.
    pub signal_detector: Option<Box<dyn LineImportanceDetector>>,
}

impl CompressionPipeline {
    /// Create a pipeline with the given config and default reformatters.
    ///
    /// Default reformatters:
    /// - [`LineCollapser`] (collapses consecutive repeated lines)
    ///
    /// Default offloaders:
    /// - [`LengthOffloader`] (offloads lines > 100 chars)
    ///
    /// Tokenizer, CCR store, and signal detector start as `None` —
    /// set them directly on the struct before calling [`run`](Self::run).
    pub fn default_with_config(config: PipelineConfig) -> Self {
        Self {
            reformatters: vec![Box::new(LineCollapser)],
            offloaders: vec![Box::new(LengthOffloader {
                threshold_chars: 100,
            })],
            tokenizer: None,
            ccr_store: None,
            signal_detector: None,
            config,
        }
    }

    /// Run the compression pipeline on `input`.
    ///
    /// 1. **Reformat** — apply all reformat transforms in order.
    /// 2. **Budget check** — estimate tokens; return early if within budget.
    /// 3. **Score** — score each line via signal detector or default heuristic.
    /// 4. **Select** — greedily pick highest-score lines within token budget.
    /// 5. **Offload** — store dropped content in CCR if enabled.
    /// 6. **Render** — output selected lines in original order.
    pub fn run(&self, input: &str, content_type: crate::ContentType) -> Result<String> {
        if !self.config.enabled {
            return Ok(input.to_string());
        }

        // === Phase 1: Reformat (serial) ===
        let mut output = input.to_string();
        for t in &self.reformatters {
            output = t.reformat(&output)?;
        }

        // === Phase 2: Budget check ===
        let tokenizer = match (&self.tokenizer, self.config.max_tokens) {
            (Some(t), max) if max > 0 => t,
            _ => return Ok(output),
        };

        let current_tokens = tokenizer.estimate(&output);
        if current_tokens <= self.config.max_tokens {
            return Ok(output);
        }

        // === Phase 3: Score each line ===
        struct ScoredLine {
            index: usize,
            text: String,
            score: f64,
            signal: Option<ImportanceSignal>,
        }

        let mut scored: Vec<ScoredLine> = output
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let (score, signal) = if let Some(detector) = &self.signal_detector {
                    let ctx = SignalContext {
                        content_type,
                        line_number: i + 1,
                    };
                    let sig = detector.score(line, &ctx);
                    (
                        sig.as_ref().map(|s| s.effective_score()).unwrap_or(0.0),
                        sig,
                    )
                } else {
                    (default_score(line), None)
                };
                ScoredLine {
                    index: i,
                    text: line.to_string(),
                    score,
                    signal,
                }
            })
            .collect();

        // === Phase 4: Greedy selection within budget ===
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut keep = vec![false; scored.len()];
        let mut used: usize = 0;

        for sl in &scored {
            let line_tokens = tokenizer.estimate(&sl.text);
            if used + line_tokens <= self.config.max_tokens {
                keep[sl.index] = true;
                used += line_tokens;
            } else if self.config.enable_ccr {
                // Offload dropped content
                if let Some(store) = &self.ccr_store {
                    for offloader in &self.offloaders {
                        if offloader.estimate_bloat(&sl.text, &sl.signal) {
                            let _ = offloader.apply(&sl.text, store.as_ref());
                        }
                    }
                }
            }
        }

        // === Phase 5: Render in original order ===
        let compressed: String = scored
            .into_iter()
            .filter(|sl| keep[sl.index])
            .map(|sl| sl.text)
            .collect::<Vec<_>>()
            .join("\n");

        Ok(compressed)
    }
}

/// Default line importance score (used when no signal detector is configured).
fn default_score(line: &str) -> f64 {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        0.0
    } else if trimmed.len() < 10 {
        0.3
    } else if trimmed.starts_with("error")
        || trimmed.starts_with("Error")
        || trimmed.starts_with("FAILED")
        || trimmed.starts_with("fatal")
    {
        0.9
    } else if trimmed.starts_with("warning")
        || trimmed.starts_with("Warning")
        || trimmed.starts_with("WARN")
    {
        0.7
    } else {
        0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signals::{SignalCategory, SignalContext};
    use crate::ContentType;

    /// A mock detector that returns different scores based on line content.
    #[derive(Debug)]
    struct MockDetector;

    impl LineImportanceDetector for MockDetector {
        fn score(&self, line: &str, _ctx: &SignalContext) -> Option<ImportanceSignal> {
            let trimmed = line.trim();
            if trimmed.starts_with("error") {
                Some(ImportanceSignal {
                    category: SignalCategory::Error,
                    priority: 1.0,
                    confidence: 0.95,
                })
            } else if trimmed.starts_with("warning") {
                Some(ImportanceSignal {
                    category: SignalCategory::Warning,
                    priority: 0.7,
                    confidence: 0.85,
                })
            } else if trimmed.is_empty() {
                Some(ImportanceSignal {
                    category: SignalCategory::Plain,
                    priority: 0.0,
                    confidence: 1.0,
                })
            } else {
                None
            }
        }
    }

    fn make_pipeline(enabled: bool, max_tokens: usize) -> CompressionPipeline {
        let cfg = PipelineConfig {
            enabled,
            max_tokens,
            ..Default::default()
        };
        let mut p = CompressionPipeline::default_with_config(cfg);
        p.tokenizer = Some(Box::new(crate::tokenizer::ApproximateEstimator));
        p.signal_detector = Some(Box::new(MockDetector));
        p
    }

    #[test]
    fn test_disabled_pipeline_passthrough() {
        let p = make_pipeline(false, 10);
        let input = "line one\nline two\nline three";
        let output = p.run(input, ContentType::PlainText).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_within_budget_no_change() {
        let p = make_pipeline(true, 100);
        let input = "error: something broke\nwarning: minor issue";
        let output = p.run(input, ContentType::PlainText).unwrap();
        // Both lines should fit within 100 tokens
        assert_eq!(output, input);
    }

    #[test]
    fn test_over_budget_keeps_important() {
        // Create many lines so we exceed the token budget
        let p = make_pipeline(true, 4);
        let lines: Vec<String> = (0..20).map(|i| format!("info: line {}", i)).collect();
        let input = lines.join("\n");

        let output = p.run(&input, ContentType::PlainText).unwrap();
        assert!(!output.is_empty(), "Output should not be empty");
        assert!(
            output.len() < input.len(),
            "Output should be smaller than input"
        );
    }

    #[test]
    fn test_errors_kept_over_info() {
        let p = make_pipeline(true, 8);
        let input = "info: unimportant\nerror: critical failure\ninfo: another noise";
        let output = p.run(input, ContentType::PlainText).unwrap();
        assert!(
            output.contains("critical failure"),
            "Should keep error lines"
        );
    }

    #[test]
    fn test_default_with_config() {
        let cfg = PipelineConfig::default();
        let p = CompressionPipeline::default_with_config(cfg);
        assert_eq!(p.reformatters.len(), 1);
        assert_eq!(p.offloaders.len(), 1);
        assert!(p.tokenizer.is_none());
        assert!(p.ccr_store.is_none());
    }
}
