//! LogCompressor — detect log format, classify lines by log level, score
//! selectively, and collapse repeated patterns.
//!
//! Ported from headroom's `transforms/log_compressor.rs` with simplified
//! format detection (no magika integration, no auth-mode policies).
//!
//! Pipeline:
//! 1. Detect log format from first N lines
//! 2. Classify each line by log level
//! 3. Score lines: errors > warnings > info > debug
//! 4. Select lines to keep within budget
//! 5. Optionally collapse repeated similar lines

use anyhow::Result;

/// Supported log formats for detection.
#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
    Pytest,
    Npm,
    Cargo,
    Jest,
    Make,
    Generic,
}

impl LogFormat {
    /// Detect log format from the first few lines of input.
    pub fn detect(lines: &[&str]) -> Self {
        let sample = lines.iter().take(10).collect::<Vec<_>>();

        for line in &sample {
            let l = line.trim();
            // Pytest: "test_foo.py::test_bar PASSED", "FAILED", "ERROR"
            if l.contains("PASSED") || l.contains("FAILED") || l.contains("test session") {
                return LogFormat::Pytest;
            }
            // Jest: "PASS src/foo.test.ts", "FAIL src/bar.test.ts"
            if (l.starts_with("PASS ") || l.starts_with("FAIL ")) && l.contains(".test.") {
                return LogFormat::Jest;
            }
            // Cargo: "Compiling", "Finished", "error[E"
            if l.starts_with("Compiling ") || l.starts_with("Finished ") || l.contains("error[E") {
                return LogFormat::Cargo;
            }
            // Npm: "npm ERR!", "npm WARN"
            if l.contains("npm ERR") || l.contains("npm WARN") {
                return LogFormat::Npm;
            }
            // Make: "make[", "Entering directory", "Leaving directory"
            if l.starts_with("make[") || l.contains("Entering directory") {
                return LogFormat::Make;
            }
        }

        LogFormat::Generic
    }
}

/// Log level for a line of output.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Error,
    Fail,
    Warn,
    Info,
    Debug,
    Trace,
    Unknown,
}

impl LogLevel {
    /// Priority for selection (higher = more important to keep).
    pub fn priority(&self) -> usize {
        match self {
            LogLevel::Error => 100,
            LogLevel::Fail => 90,
            LogLevel::Warn => 70,
            LogLevel::Info => 40,
            LogLevel::Debug => 20,
            LogLevel::Trace => 10,
            LogLevel::Unknown => 30,
        }
    }
}

/// A single classified log line.
#[derive(Debug, Clone)]
pub struct LogLine {
    pub content: String,
    pub level: LogLevel,
    pub score: f64,
}

/// Configuration for the log compressor.
#[derive(Debug, Clone)]
pub struct LogCompressorConfig {
    /// Maximum lines to keep per log level.
    pub max_lines_per_level: Vec<(LogLevel, usize)>,
    /// Lines of context to preserve before/after an error.
    pub preserve_error_context: usize,
    /// Whether to collapse repeated similar lines.
    pub enable_template_detection: bool,
    /// Threshold for collapsing repeated lines (how many duplicates).
    pub collapse_threshold: usize,
}

impl Default for LogCompressorConfig {
    fn default() -> Self {
        Self {
            max_lines_per_level: vec![
                (LogLevel::Error, 100),
                (LogLevel::Fail, 80),
                (LogLevel::Warn, 50),
                (LogLevel::Info, 30),
                (LogLevel::Debug, 10),
                (LogLevel::Trace, 5),
                (LogLevel::Unknown, 20),
            ],
            preserve_error_context: 2,
            enable_template_detection: false,
            collapse_threshold: 5,
        }
    }
}

/// Compressed log output.
#[derive(Debug, Clone)]
pub struct LogCompressionResult {
    pub text: String,
    pub total_lines: usize,
    pub kept_lines: usize,
    pub collapsed_groups: usize,
    pub errors_kept: usize,
    pub warnings_kept: usize,
}

/// The LogCompressor.
#[derive(Debug, Clone, Default)]
pub struct LogCompressor {
    pub config: LogCompressorConfig,
    pub format: Option<LogFormat>,
}

impl LogCompressor {
    /// Create a new LogCompressor with default config and auto-detection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a LogCompressor with a specific config and optional format.
    pub fn with_config(config: LogCompressorConfig) -> Self {
        Self {
            config,
            format: None,
        }
    }

    /// Compress log output.
    pub fn compress(&mut self, input: &str) -> Result<LogCompressionResult> {
        if input.trim().is_empty() {
            return Ok(LogCompressionResult {
                text: String::new(),
                total_lines: 0,
                kept_lines: 0,
                collapsed_groups: 0,
                errors_kept: 0,
                warnings_kept: 0,
            });
        }

        // Detect format
        let lines: Vec<&str> = input.lines().collect();
        let format = self
            .format
            .clone()
            .unwrap_or_else(|| LogFormat::detect(&lines));
        self.format = Some(format.clone());

        // Classify each line
        let classified: Vec<LogLine> = lines
            .iter()
            .map(|line| {
                let level = self.classify_level(line, &format);
                let score = level.priority() as f64 / 100.0;
                LogLine {
                    content: line.to_string(),
                    level,
                    score,
                }
            })
            .collect();

        let total_lines = classified.len();

        // Collapse repeated patterns if enabled
        let (collapsed, collapsed_groups) = if self.config.enable_template_detection {
            self.collapse_repeated(&classified)
        } else {
            (classified, 0usize)
        };

        // Select lines to keep
        let selected = self.select_lines(&collapsed);

        let mut errors_kept = 0;
        let mut warnings_kept = 0;
        for line in &selected {
            match line.level {
                LogLevel::Error | LogLevel::Fail => errors_kept += 1,
                LogLevel::Warn => warnings_kept += 1,
                _ => {}
            }
        }

        let kept_lines = selected.len();
        let text: String = selected
            .into_iter()
            .map(|l| l.content)
            .collect::<Vec<_>>()
            .join("\n");

        Ok(LogCompressionResult {
            text,
            total_lines,
            kept_lines,
            collapsed_groups,
            errors_kept,
            warnings_kept,
        })
    }

    /// Classify a line's log level based on content and detected format.
    fn classify_level(&self, line: &str, format: &LogFormat) -> LogLevel {
        let trimmed = line.trim();

        // Cross-format error detection
        if trimmed.starts_with("error")
            || trimmed.starts_with("Error:")
            || trimmed.starts_with("ERROR:")
            || trimmed.contains("[E")
            || trimmed.contains("FAILED")
            || trimmed.contains("failed:")
            || trimmed.contains("Error ")
        {
            return LogLevel::Error;
        }

        // Cross-format warning detection
        if trimmed.starts_with("warning")
            || trimmed.starts_with("Warning:")
            || trimmed.starts_with("WARN:")
            || trimmed.contains("WARN ")
        {
            return LogLevel::Warn;
        }

        // Info detection
        if trimmed.starts_with("info:")
            || trimmed.starts_with("INFO:")
            || trimmed.starts_with("info ")
        {
            return LogLevel::Info;
        }

        // Debug/trace detection
        if trimmed.starts_with("debug:")
            || trimmed.starts_with("DEBUG:")
            || trimmed.starts_with("trace:")
            || trimmed.starts_with("TRACE:")
        {
            return LogLevel::Debug;
        }

        // Format-specific patterns
        match format {
            LogFormat::Pytest => {
                if trimmed.contains("PASSED") {
                    LogLevel::Info
                } else if trimmed.contains("FAILED") || trimmed.contains("ERROR") {
                    LogLevel::Error
                } else if trimmed.contains("warnings") || trimmed.contains("warning") {
                    LogLevel::Warn
                } else if trimmed.starts_with("test_") || trimmed.ends_with("::") {
                    LogLevel::Info
                } else {
                    LogLevel::Unknown
                }
            }
            LogFormat::Npm => {
                if trimmed.contains("npm ERR") {
                    LogLevel::Error
                } else if trimmed.contains("npm WARN") {
                    LogLevel::Warn
                } else if trimmed.contains("npm info") || trimmed.contains("verbose") {
                    LogLevel::Info
                } else {
                    LogLevel::Unknown
                }
            }
            LogFormat::Cargo => {
                if trimmed.contains("error[E")
                    || trimmed.starts_with("error:")
                    || trimmed.contains("aborting")
                {
                    LogLevel::Error
                } else if trimmed.starts_with("warning:") || trimmed.contains("warning[") {
                    LogLevel::Warn
                } else if trimmed.starts_with("Compiling")
                    || trimmed.starts_with("Finished")
                    || trimmed.starts_with("Checking")
                {
                    LogLevel::Info
                } else if trimmed.starts_with("Downloading") || trimmed.starts_with("Updating") {
                    LogLevel::Debug
                } else {
                    LogLevel::Unknown
                }
            }
            LogFormat::Jest => {
                if trimmed.starts_with("FAIL ") {
                    LogLevel::Error
                } else if trimmed.starts_with("PASS ")
                    || trimmed.contains("Tests:")
                    || trimmed.contains("Snapshots:")
                {
                    LogLevel::Info
                } else if trimmed.contains("console.warn") || trimmed.contains("console.error") {
                    LogLevel::Warn
                } else {
                    LogLevel::Unknown
                }
            }
            LogFormat::Make | LogFormat::Generic => {
                if trimmed.starts_with("make[") {
                    LogLevel::Info
                } else if trimmed.starts_with("Entering") || trimmed.starts_with("Leaving") {
                    LogLevel::Debug
                } else {
                    LogLevel::Unknown
                }
            }
        }
    }

    /// Collapse consecutive repeated lines into compact form.
    fn collapse_repeated(&self, lines: &[LogLine]) -> (Vec<LogLine>, usize) {
        if lines.is_empty() {
            return (Vec::new(), 0);
        }

        let mut collapsed: Vec<LogLine> = Vec::new();
        let mut groups_collapsed = 0;

        let mut i = 0;
        while i < lines.len() {
            let mut count = 1;
            while i + count < lines.len()
                && lines[i + count].content == lines[i].content
                && lines[i + count].level == lines[i].level
            {
                count += 1;
            }

            if count >= self.config.collapse_threshold {
                groups_collapsed += 1;
                collapsed.push(LogLine {
                    content: format!("{} \u{00d7}{}", lines[i].content.trim(), count),
                    level: lines[i].level.clone(),
                    score: lines[i].score,
                });
            } else {
                for j in 0..count {
                    collapsed.push(lines[i + j].clone());
                }
            }

            i += count;
        }

        (collapsed, groups_collapsed)
    }

    /// Select lines to keep based on level priority and per-level limits.
    fn select_lines(&self, lines: &[LogLine]) -> Vec<LogLine> {
        if lines.is_empty() {
            return Vec::new();
        }

        // Build level limit map
        let mut limits: std::collections::HashMap<&LogLevel, usize> =
            std::collections::HashMap::new();
        for (level, limit) in &self.config.max_lines_per_level {
            limits.insert(level, *limit);
        }

        // Track error positions for context preservation
        let error_positions: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.level == LogLevel::Error || l.level == LogLevel::Fail)
            .map(|(i, _)| i)
            .collect();

        let mut selected: Vec<(usize, &LogLine)> = Vec::new();
        let mut level_counts: std::collections::HashMap<&LogLevel, usize> =
            std::collections::HashMap::new();

        // First pass: select by priority, respecting per-level limits
        for (i, line) in lines.iter().enumerate() {
            let level_key = &line.level;
            let max_for_level = limits.get(level_key).copied().unwrap_or(usize::MAX);
            let count = level_counts.get(level_key).copied().unwrap_or(0);

            if count < max_for_level {
                selected.push((i, line));
                *level_counts.entry(level_key).or_insert(0) += 1;
            }
        }

        // Add error context lines (lines near errors that might have been dropped)
        let kept_indices: std::collections::HashSet<usize> =
            selected.iter().map(|(i, _)| *i).collect();
        let ctx = self.config.preserve_error_context;

        for &err_pos in &error_positions {
            for offset in 1..=ctx {
                if err_pos >= offset {
                    let before = err_pos - offset;
                    if !kept_indices.contains(&before) {
                        selected.push((before, &lines[before]));
                    }
                }
                let after = err_pos + offset;
                if after < lines.len() && !kept_indices.contains(&after) {
                    selected.push((after, &lines[after]));
                }
            }
        }

        // Deduplicate by original index
        selected.sort_by_key(|(i, _)| *i);
        selected.dedup_by_key(|(i, _)| *i);

        // Re-sort by original position to maintain order
        selected.sort_by_key(|(i, _)| *i);
        selected.into_iter().map(|(_, l)| l.clone()).collect()
    }
}

/// Compress log output with auto-detection and default config.
pub fn compress_logs(input: &str) -> Result<String> {
    let mut compressor = LogCompressor::new();
    let result = compressor.compress(input)?;
    Ok(result.text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_detect_pytest() {
        let lines = vec![
            "test_foo.py::test_bar PASSED",
            "test_baz.py::test_qux FAILED",
        ];
        assert_eq!(LogFormat::detect(&lines), LogFormat::Pytest);
    }

    #[test]
    fn test_log_format_detect_cargo() {
        let lines = vec!["Compiling foo v1.0.0", "error[E0308]: mismatched types"];
        assert_eq!(LogFormat::detect(&lines), LogFormat::Cargo);
    }

    #[test]
    fn test_log_format_detect_npm() {
        let lines = vec!["npm ERR! code 1", "npm WARN deprecated foo@1.0.0"];
        assert_eq!(LogFormat::detect(&lines), LogFormat::Npm);
    }

    #[test]
    fn test_log_format_detect_jest() {
        let lines = vec!["PASS src/foo.test.ts", "FAIL src/bar.test.ts"];
        assert_eq!(LogFormat::detect(&lines), LogFormat::Jest);
    }

    #[test]
    fn test_log_format_detect_generic() {
        let lines = vec!["some random output", "more output"];
        assert_eq!(LogFormat::detect(&lines), LogFormat::Generic);
    }

    #[test]
    fn test_log_compressor_empty() {
        let mut compressor = LogCompressor::new();
        let result = compressor.compress("").unwrap();
        assert!(result.text.is_empty());
        assert_eq!(result.total_lines, 0);
    }

    #[test]
    fn test_log_compressor_classify_error() {
        let line = "error[E0308]: mismatched types";
        let compressor = LogCompressor::new();
        assert_eq!(
            compressor.classify_level(line, &LogFormat::Generic),
            LogLevel::Error
        );
    }

    #[test]
    fn test_log_compressor_classify_warning() {
        let line = "warning: unused variable";
        let compressor = LogCompressor::new();
        assert_eq!(
            compressor.classify_level(line, &LogFormat::Generic),
            LogLevel::Warn
        );
    }

    #[test]
    fn test_log_compressor_keeps_errors() {
        let input = "\
info: starting build
error[E0308]: mismatched types
info: build failed
";
        let mut compressor = LogCompressor::new();
        let result = compressor.compress(input).unwrap();
        assert!(result.text.contains("E0308"), "Should keep error lines");
        assert!(result.errors_kept >= 1, "Should count errors");
    }

    #[test]
    fn test_log_compressor_select_errors_over_info() {
        let input = "\
info: first
info: second
error: critical failure
info: third
info: fourth
";
        let mut compressor = LogCompressor::with_config(LogCompressorConfig {
            max_lines_per_level: vec![
                (LogLevel::Error, 10),
                (LogLevel::Info, 2),
                (LogLevel::Unknown, 5),
            ],
            preserve_error_context: 0,
            enable_template_detection: false,
            collapse_threshold: 5,
        });
        let result = compressor.compress(input).unwrap();
        assert!(
            result.text.contains("critical failure"),
            "Error must be kept"
        );
        // With max_lines_per_level Info=2, we keep at most 2 info lines
        let info_count = result
            .text
            .lines()
            .filter(|l| l.starts_with("info:"))
            .count();
        assert!(
            info_count <= 2,
            "Should limit info lines to 2, got {}",
            info_count
        );
    }

    #[test]
    fn test_log_compressor_collapse_repeated() {
        let input = "\
error: timeout
error: timeout
error: timeout
error: timeout
error: timeout
done
";
        let mut compressor = LogCompressor::with_config(LogCompressorConfig {
            enable_template_detection: true,
            collapse_threshold: 3,
            ..Default::default()
        });
        let result = compressor.compress(input).unwrap();
        // Should have collapsed at least one group
        assert!(
            result.collapsed_groups >= 1,
            "Should collapse repeated lines"
        );
        // The collapsed line should have ×5 marker
        assert!(
            result.text.contains('\u{00d7}'),
            "Collapsed line should have × marker"
        );
    }

    #[test]
    fn test_log_compressor_context_around_errors() {
        let mut compressor = LogCompressor::with_config(LogCompressorConfig {
            preserve_error_context: 1,
            ..Default::default()
        });
        let result = compressor
            .compress("info: before\nerror: boom\ninfo: after\ninfo: far")
            .unwrap();
        assert!(result.text.contains("before"), "Context before error kept");
        assert!(result.text.contains("after"), "Context after error kept");
    }
}
