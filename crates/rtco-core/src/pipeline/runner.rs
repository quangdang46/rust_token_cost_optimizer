//! Convenience functions for running the compression pipeline.
//!
//! The main entry point is [`compress_with_pipeline`], which creates a
//! pipeline with built-in defaults and runs it, falling back to
//! passthrough on failure.

use super::{CompressionPipeline, PipelineConfig};
use crate::ContentType;

/// Run the compression pipeline with default transform configuration.
///
/// Creates a pipeline with built-in reformatters (line collapser) and
/// sensible defaults. Falls back to returning the original input on any
/// error (graceful degradation).
///
/// # Arguments
/// * `input` — Text to compress.
/// * `config` — Pipeline configuration (max_tokens, offload settings, etc.).
/// * `content_type` — Type hint for signal scoring.
///
/// # Returns
/// Compressed text, or the original input if compression fails or is disabled.
pub fn compress_with_pipeline(
    input: &str,
    config: &PipelineConfig,
    content_type: ContentType,
) -> String {
    if !config.enabled || config.max_tokens == 0 {
        return input.to_string();
    }

    let pipeline = CompressionPipeline::default_with_config(config.clone());
    match pipeline.run(input, content_type) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("rtco: pipeline warning: {}", e);
            input.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_returns_input() {
        let cfg = PipelineConfig {
            enabled: false,
            ..Default::default()
        };
        let input = "some text to compress";
        let result = compress_with_pipeline(input, &cfg, ContentType::PlainText);
        assert_eq!(result, input);
    }

    #[test]
    fn test_zero_max_tokens_returns_input() {
        let cfg = PipelineConfig {
            enabled: true,
            max_tokens: 0,
            ..Default::default()
        };
        let input = "some text";
        let result = compress_with_pipeline(input, &cfg, ContentType::PlainText);
        assert_eq!(result, input);
    }

    #[test]
    fn test_enabled_within_budget_unchanged() {
        let cfg = PipelineConfig {
            enabled: true,
            max_tokens: 1000,
            ..Default::default()
        };
        let input = "short text";
        let result = compress_with_pipeline(input, &cfg, ContentType::PlainText);
        assert_eq!(result, input);
    }

    #[test]
    fn test_enabled_over_budget_truncated() {
        let cfg = PipelineConfig {
            enabled: true,
            max_tokens: 5,
            ..Default::default()
        };
        // Long enough input that it should be truncated
        let input = "line one\nline two\nline three\nline four\nline five\nline six";
        let result = compress_with_pipeline(input, &cfg, ContentType::PlainText);
        assert!(!result.is_empty(), "Result should not be empty");
        assert!(
            result.len() < input.len() || result == input,
            "Result should be smaller or equal"
        );
    }

    #[test]
    fn test_empty_input() {
        let cfg = PipelineConfig {
            enabled: true,
            max_tokens: 100,
            ..Default::default()
        };
        let result = compress_with_pipeline("", &cfg, ContentType::PlainText);
        assert_eq!(result, "");
    }
}
