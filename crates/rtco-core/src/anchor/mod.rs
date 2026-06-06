//! Anchor Selector — identify and preserve structural anchor lines in output.
//!
//! Anchors are lines that establish context, define structure, or serve as
//! reference points. They should be preserved even during aggressive truncation.
//!
//! Ported from headroom's `transforms/anchor_selector.rs` (1189 LOC) with
//! simplified heuristic-based detection instead of ML-like scoring.

pub mod selectors;

use selectors::AnchorType;

/// Configuration for the anchor selector.
#[derive(Debug, Clone)]
pub struct AnchorConfig {
    /// Whether anchor preservation is enabled.
    pub preserve_anchors: bool,
    /// Maximum number of anchors to preserve.
    pub max_anchors: usize,
    /// Priority boost multiplier for anchor lines (0.0–1.0).
    pub anchor_boost: f64,
}

impl Default for AnchorConfig {
    fn default() -> Self {
        Self {
            preserve_anchors: true,
            max_anchors: 20,
            anchor_boost: 0.5,
        }
    }
}

/// Detected anchor in a line.
#[derive(Debug, Clone)]
pub struct Anchor {
    /// The line content.
    pub line: String,
    /// The line number (0-indexed).
    pub line_number: usize,
    /// The anchor type.
    pub anchor_type: AnchorType,
    /// Computed priority (1.0 = highest).
    pub priority: f64,
}

/// Result of anchor selection.
#[derive(Debug, Clone)]
pub struct AnchorSelectionResult {
    /// Lines that are anchors and should be preserved.
    pub anchors: Vec<Anchor>,
    /// Total lines scanned.
    pub total_lines: usize,
    /// Number of anchor lines found.
    pub anchor_count: usize,
}

/// The AnchorSelector.
#[derive(Debug, Clone, Default)]
pub struct AnchorSelector {
    pub config: AnchorConfig,
}

impl AnchorSelector {
    /// Create a new AnchorSelector with default config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an AnchorSelector with a custom config.
    pub fn with_config(config: AnchorConfig) -> Self {
        Self { config }
    }

    /// Scan lines and return detected anchors.
    pub fn select_anchors(&self, input: &str) -> AnchorSelectionResult {
        let lines: Vec<&str> = input.lines().collect();
        let total_lines = lines.len();

        if !self.config.preserve_anchors || total_lines == 0 {
            return AnchorSelectionResult {
                anchors: Vec::new(),
                total_lines,
                anchor_count: 0,
            };
        }

        let mut anchors: Vec<Anchor> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(anchor_type) = selectors::detect_anchor_type(trimmed) {
                let priority = anchor_type.default_priority();
                anchors.push(Anchor {
                    line: line.to_string(),
                    line_number: i,
                    anchor_type,
                    priority,
                });
            }
        }

        // Sort by priority descending, limit to max_anchors
        anchors.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        anchors.truncate(self.config.max_anchors);

        let anchor_count = anchors.len();

        AnchorSelectionResult {
            anchors,
            total_lines,
            anchor_count,
        }
    }

    /// Apply anchor boost to a set of line scores.
    ///
    /// Given a map of line_number → score, this method boosts the scores of
    /// anchor lines by `anchor_boost`.
    pub fn apply_anchor_boost(&self, scores: &mut [(usize, f64)], input: &str) {
        let anchors = self.select_anchors(input);
        let anchor_lines: std::collections::HashSet<usize> =
            anchors.anchors.iter().map(|a| a.line_number).collect();

        for (line_number, score) in scores.iter_mut() {
            if anchor_lines.contains(line_number) {
                *score = (*score + self.config.anchor_boost).min(1.0);
            }
        }
    }
}

/// Convenience function to detect anchors in a string.
pub fn detect_anchors(input: &str) -> AnchorSelectionResult {
    let selector = AnchorSelector::new();
    selector.select_anchors(input)
}

/// Convenience function to get anchor-boosted line numbers.
pub fn get_anchor_lines(input: &str) -> Vec<usize> {
    let result = detect_anchors(input);
    result.anchors.into_iter().map(|a| a.line_number).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_selector_empty() {
        let selector = AnchorSelector::new();
        let result = selector.select_anchors("");
        assert_eq!(result.anchor_count, 0);
        assert_eq!(result.total_lines, 0);
    }

    #[test]
    fn test_anchor_selector_disabled() {
        let selector = AnchorSelector::with_config(AnchorConfig {
            preserve_anchors: false,
            ..Default::default()
        });
        let result = selector.select_anchors("fn main() {\nprintln!(\"hello\");\n}");
        assert_eq!(result.anchor_count, 0);
    }

    #[test]
    fn test_anchor_selector_detects_headers() {
        let selector = AnchorSelector::new();
        let result = selector.select_anchors("# section 1\nsome content\n## subsection\nmore");
        assert!(result.anchor_count >= 2, "Should detect Markdown headers");
        assert!(result.anchors.iter().any(|a| a.line.contains("section 1")));
        assert!(result.anchors.iter().any(|a| a.line.contains("subsection")));
    }

    #[test]
    fn test_anchor_selector_detects_definitions() {
        let selector = AnchorSelector::new();
        let result = selector.select_anchors("fn main() {\n// some code\npub fn helper() {\n}");
        assert!(result.anchor_count >= 2, "Should detect fn definitions");
    }

    #[test]
    fn test_anchor_selector_detects_paths() {
        let selector = AnchorSelector::new();
        let result = selector.select_anchors("src/main.rs:42\n/usr/bin/env\n~/projects/foo");
        assert!(result.anchor_count >= 2, "Should detect file paths");
    }

    #[test]
    fn test_anchor_selector_max_anchors() {
        let selector = AnchorSelector::with_config(AnchorConfig {
            max_anchors: 2,
            ..Default::default()
        });
        let result = selector.select_anchors("fn a()\nfn b()\nfn c()\nfn d()");
        assert_eq!(result.anchor_count, 2, "Should limit to max_anchors");
    }

    #[test]
    fn test_anchor_boost() {
        let selector = AnchorSelector::with_config(AnchorConfig {
            anchor_boost: 0.5,
            ..Default::default()
        });
        let input = "fn main() {\nlet x = 1;\n}";
        let mut scores: Vec<(usize, f64)> = vec![(0, 0.3), (1, 0.3), (2, 0.3)];
        selector.apply_anchor_boost(&mut scores, input);
        // Line 0 (fn main()) is an anchor -> boosted
        assert!(
            (scores[0].1 - 0.8).abs() < 0.01,
            "Anchor line should be boosted: got {}",
            scores[0].1
        );
    }
}
