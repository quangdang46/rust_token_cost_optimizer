//! Built-in [`ReformatTransform`](super::ReformatTransform) implementations.
//!
//! These transforms make output more compact without dropping semantic content.

use super::ReformatTransform;
use anyhow::{Context, Result};

/// Minifies JSON by removing unnecessary whitespace.
///
/// Parses then re-serializes with `serde_json::to_string` which produces
/// compact output. Gracefully falls back to passthrough if input is not
/// valid JSON.
#[derive(Debug)]
pub struct JsonMinifier;

impl ReformatTransform for JsonMinifier {
    fn name(&self) -> &str {
        "json_minifier"
    }

    fn reformat(&self, input: &str) -> Result<String> {
        let trimmed = input.trim();
        if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
            return Ok(input.to_string());
        }
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(val) => {
                Ok(serde_json::to_string(&val).context("Failed to serialize minified JSON")?)
            }
            Err(_) => Ok(input.to_string()),
        }
    }

    fn estimated_savings(&self) -> f64 {
        0.2
    }
}

/// Collapses consecutive identical lines into a single line with a count marker.
///
/// ```text
/// Running test foo ...
/// Running test foo ...
/// Running test foo ...
/// ```
/// becomes:
/// ```text
/// Running test foo ... [×3]
/// ```
#[derive(Debug)]
pub struct LineCollapser;

impl ReformatTransform for LineCollapser {
    fn name(&self) -> &str {
        "line_collapser"
    }

    fn reformat(&self, input: &str) -> Result<String> {
        let lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            return Ok(String::new());
        }

        let mut result: Vec<String> = Vec::with_capacity(lines.len());
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            let mut count = 1;
            while i + 1 < lines.len() && lines[i + 1] == line {
                count += 1;
                i += 1;
            }
            if count > 1 {
                result.push(format!("{} [×{}]", line, count));
            } else {
                result.push(line.to_string());
            }
            i += 1;
        }

        Ok(result.join("\n"))
    }

    fn estimated_savings(&self) -> f64 {
        0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── JsonMinifier ────────────────────────────────────────────────

    #[test]
    fn test_json_minifier_valid_object() {
        let input = r#"{
            "name": "test",
            "value": 42
        }"#;
        let output = JsonMinifier.reformat(input).unwrap();
        assert!(
            !output.contains(' '),
            "Should have no whitespace: {}",
            output
        );
        assert!(output.contains("\"name\""), "Should keep keys");
        assert!(output.contains("42"), "Should keep values");
    }

    #[test]
    fn test_json_minifier_valid_array() {
        let input = "[ 1, 2, 3 ]";
        let output = JsonMinifier.reformat(input).unwrap();
        assert_eq!(output, "[1,2,3]");
    }

    #[test]
    fn test_json_minifier_non_json_passthrough() {
        let input = "not json at all";
        let output = JsonMinifier.reformat(input).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_json_minifier_empty_string() {
        let output = JsonMinifier.reformat("").unwrap();
        assert_eq!(output, "");
    }

    // ── LineCollapser ───────────────────────────────────────────────

    #[test]
    fn test_line_collapser_no_repeats() {
        let input = "line one\nline two\nline three";
        let output = LineCollapser.reformat(input).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_line_collapser_with_repeats() {
        let input = "same\nsame\nsame\ndifferent";
        let output = LineCollapser.reformat(input).unwrap();
        assert!(output.contains("[×3]"), "Should mark 3 repeats");
        assert!(output.contains("different"), "Should keep unique lines");
    }

    #[test]
    fn test_line_collapser_all_same() {
        let input = "identical\nidentical\nidentical";
        let output = LineCollapser.reformat(input).unwrap();
        assert_eq!(output, "identical [×3]");
    }

    #[test]
    fn test_line_collapser_empty() {
        let output = LineCollapser.reformat("").unwrap();
        assert_eq!(output, "");
    }

    #[test]
    fn test_line_collapser_single_line() {
        let output = LineCollapser.reformat("only one").unwrap();
        assert_eq!(output, "only one");
    }

    #[test]
    fn test_line_collapser_single_repeat() {
        let input = "pair\npair";
        let output = LineCollapser.reformat(input).unwrap();
        assert_eq!(output, "pair [×2]");
    }
}
