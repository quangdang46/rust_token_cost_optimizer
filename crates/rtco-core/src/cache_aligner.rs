//! CacheAligner — align output to token boundaries that maximize LLM
//! prefix caching.
//!
//! While primarily designed for Anthropic's prompt caching API, the concept
//! of "frozen zones" (lines that should never be truncated) is useful for
//! any LLM proxy.
//!
//! Ported from headroom's `transforms/cache_control.rs` with a focus on
//! the alignment algorithm rather than API-specific cache control headers.

/// Default target alignment boundary in tokens (Anthropic prompt cache size).
const DEFAULT_ALIGNMENT: usize = 1024;

/// Configuration for the cache aligner.
#[derive(Debug, Clone)]
pub struct CacheAlignerConfig {
    /// Target alignment boundary in tokens.
    pub target_alignment: usize,
    /// Padding token to use for alignment (e.g., "\n").
    pub pad_token: String,
    /// Whether cache alignment is enabled.
    pub enabled: bool,
    /// Whether to mark frozen zones (lines that should not be truncated).
    pub detect_frozen_zones: bool,
}

impl Default for CacheAlignerConfig {
    fn default() -> Self {
        Self {
            target_alignment: DEFAULT_ALIGNMENT,
            pad_token: "\n".to_string(),
            enabled: false,
            detect_frozen_zones: true,
        }
    }
}

/// A frozen zone: a range of lines that should be preserved together.
#[derive(Debug, Clone)]
pub struct FrozenZone {
    /// Start line index (0-indexed, inclusive).
    pub start_line: usize,
    /// End line index (0-indexed, inclusive).
    pub end_line: usize,
    /// Why this zone is frozen.
    pub reason: String,
}

/// Result of cache alignment.
#[derive(Debug, Clone)]
pub struct AlignmentResult {
    /// Aligned text.
    pub text: String,
    /// Estimated token count after alignment.
    pub estimated_tokens: usize,
    /// Target alignment boundary used.
    pub target_alignment: usize,
    /// Frozen zones detected.
    pub frozen_zones: Vec<FrozenZone>,
}

/// The CacheAligner.
#[derive(Debug, Clone, Default)]
pub struct CacheAligner {
    pub config: CacheAlignerConfig,
}

impl CacheAligner {
    /// Create a new CacheAligner with default config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a CacheAligner with a custom config.
    pub fn with_config(config: CacheAlignerConfig) -> Self {
        Self { config }
    }

    /// Align output to cache boundary.
    ///
    /// If `enabled` is false, the output is returned unchanged.
    pub fn align(&self, input: &str, estimated_tokens: usize) -> AlignmentResult {
        if !self.config.enabled {
            return AlignmentResult {
                text: input.to_string(),
                estimated_tokens,
                target_alignment: self.config.target_alignment,
                frozen_zones: Vec::new(),
            };
        }

        let frozen_zones = if self.config.detect_frozen_zones {
            self.detect_frozen_zones(input)
        } else {
            Vec::new()
        };

        // Calculate padding needed to reach the next alignment boundary
        let remainder = estimated_tokens % self.config.target_alignment;
        let tokens_to_next = if remainder == 0 {
            0
        } else {
            self.config.target_alignment - remainder
        };

        let mut aligned = input.to_string();
        if tokens_to_next > 0 && tokens_to_next < self.config.target_alignment / 2 {
            // Only pad if we're close to the next boundary (< 50%)
            let pad_count = (tokens_to_next).max(1);
            aligned.push_str(&self.config.pad_token.repeat(pad_count));
        }

        let final_tokens = estimated_tokens + tokens_to_next;

        AlignmentResult {
            text: aligned,
            estimated_tokens: final_tokens,
            target_alignment: self.config.target_alignment,
            frozen_zones,
        }
    }

    /// Detect frozen zones in the output.
    ///
    /// Frozen zones are sections that should never be split across cache
    /// boundaries: error messages, summary blocks, etc.
    fn detect_frozen_zones(&self, input: &str) -> Vec<FrozenZone> {
        let lines: Vec<&str> = input.lines().collect();
        let mut zones: Vec<FrozenZone> = Vec::new();

        // Detect error blocks (groups of consecutive error lines)
        let mut in_error_block = false;
        let mut error_start = 0;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let is_error = trimmed.starts_with("error")
                || trimmed.starts_with("Error:")
                || trimmed.starts_with("FAILED")
                || trimmed.starts_with("warning:")
                || trimmed.starts_with("Warning:");

            if is_error && !in_error_block {
                in_error_block = true;
                error_start = i;
            } else if !is_error && in_error_block {
                if i - error_start > 1 {
                    zones.push(FrozenZone {
                        start_line: error_start,
                        end_line: i - 1,
                        reason: if lines[error_start].trim().starts_with("error")
                            || lines[error_start].trim().starts_with("Error:")
                        {
                            "Error block".to_string()
                        } else {
                            "Warning block".to_string()
                        },
                    });
                }
                in_error_block = false;
            }
        }
        // Handle error block at end of input
        if in_error_block && lines.len() - error_start > 1 {
            zones.push(FrozenZone {
                start_line: error_start,
                end_line: lines.len() - 1,
                reason: "Error block at end".to_string(),
            });
        }

        // Detect summary blocks (tests results, build summary)
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim().to_lowercase();
            if trimmed.starts_with("test ")
                || trimmed.starts_with("tests:")
                || trimmed.starts_with("result:")
                || trimmed.contains("passed")
            {
                // Start from this line, include next few lines
                let end = (i + 5).min(lines.len() - 1);
                zones.push(FrozenZone {
                    start_line: i,
                    end_line: end,
                    reason: "Summary block".to_string(),
                });
            }
        }

        zones
    }
}

/// Convenience function to align output with default config.
pub fn align_output(input: &str, estimated_tokens: usize) -> String {
    let aligner = CacheAligner::new();
    let result = aligner.align(input, estimated_tokens);
    result.text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_aligner_disabled() {
        let aligner = CacheAligner::new();
        let result = aligner.align("hello world", 50);
        assert_eq!(result.text, "hello world");
        assert_eq!(result.estimated_tokens, 50);
    }

    #[test]
    fn test_cache_aligner_enabled_no_padding_needed() {
        let aligner = CacheAligner::with_config(CacheAlignerConfig {
            enabled: true,
            target_alignment: 1024,
            pad_token: "\n".to_string(),
            ..Default::default()
        });
        let result = aligner.align("hello world", 1024);
        assert_eq!(result.text, "hello world");
        assert_eq!(result.estimated_tokens, 1024);
    }

    #[test]
    fn test_cache_aligner_enabled_with_padding() {
        let aligner = CacheAligner::with_config(CacheAlignerConfig {
            enabled: true,
            target_alignment: 100,
            pad_token: "\n".to_string(),
            ..Default::default()
        });
        let result = aligner.align("hello world", 95);
        assert!(
            result.text.len() > "hello world".len(),
            "Should add padding"
        );
        assert_eq!(result.estimated_tokens, 100);
    }

    #[test]
    fn test_cache_aligner_large_gap_no_padding() {
        let aligner = CacheAligner::with_config(CacheAlignerConfig {
            enabled: true,
            target_alignment: 100,
            pad_token: "\n".to_string(),
            ..Default::default()
        });
        // 30 tokens, needs 70 to next boundary which is > 50% — don't pad
        let result = aligner.align("hello world", 30);
        // 30 % 100 = 30, tokens_to_next = 70, 70 >= 100/2, so don't pad
        assert_eq!(result.text, "hello world", "Large gap should not pad");
        assert_eq!(result.estimated_tokens, 100); // still reports aligned
    }

    #[test]
    fn test_detect_frozen_zones_errors() {
        let aligner = CacheAligner::new();
        let input = "info: build starting\nerror: type mismatch\nerror: missing semicolon\ninfo: continuing\n";
        let zones = aligner.detect_frozen_zones(input);
        assert!(
            zones.iter().any(|z| z.reason == "Error block"),
            "Should detect error block"
        );
    }

    #[test]
    fn test_detect_frozen_zones_summary() {
        let aligner = CacheAligner::new();
        let zones =
            aligner.detect_frozen_zones("test result: ok. 10 passed; 0 failed;\n  1 test passed\n");
        assert!(
            zones.iter().any(|z| z.reason == "Summary block"),
            "Should detect summary block"
        );
    }

    #[test]
    fn test_detect_frozen_zones_no_false_positives() {
        let aligner = CacheAligner::new();
        let zones = aligner.detect_frozen_zones("line1\nline2\nline3\n");
        let error_zones: Vec<_> = zones
            .iter()
            .filter(|z| z.reason.contains("Error"))
            .collect();
        assert!(
            error_zones.is_empty(),
            "Should not detect error blocks in normal text"
        );
    }

    #[test]
    fn test_cache_aligner_detect_frozen_zones_disabled() {
        let aligner = CacheAligner::with_config(CacheAlignerConfig {
            enabled: true,
            detect_frozen_zones: false,
            ..Default::default()
        });
        let result = aligner.align("error: type mismatch\nerror: missing semicolon\n", 50);
        assert!(
            result.frozen_zones.is_empty(),
            "Should not detect frozen zones when disabled"
        );
    }
}
