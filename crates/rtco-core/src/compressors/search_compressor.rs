//! SearchCompressor — compress grep/rg/find output by scoring matches,
//! grouping by file, and selecting the most important results.
//!
//! Ported from headroom's `transforms/search_compressor.rs` with simplified
//! regex parsing (supports standard grep output formats).
//!
//! Pipeline:
//! 1. Parse grep/rg output into file:match structure
//! 2. Score each match by content signals
//! 3. Score files by aggregate match quality
//! 4. Select top files and matches within token budget

use anyhow::Result;

use regex::Regex;
use std::sync::LazyLock;

/// Matches standard grep output: `file:line:content` or `file:content`
static GREP_LINE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+?):(\d+):(.+)$").unwrap());

/// Matches `file:content` format (no line number — e.g., rg -l or grep -l)
static GREP_FILE_CONTENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+?):(.+)$").unwrap());

/// Matches `file:line:col:content` (e.g., rg --json or some verbose formats)
static GREP_COL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+?):(\d+):(\d+):(.+)$").unwrap());

/// Default maximum files to include.
const DEFAULT_MAX_FILES: usize = 20;
/// Default maximum matches per file.
const DEFAULT_MAX_MATCHES_PER_FILE: usize = 10;
/// Default minimum match score to keep.
const DEFAULT_MIN_MATCH_SCORE: f64 = 0.1;

/// Configuration for the search compressor.
#[derive(Debug, Clone)]
pub struct SearchCompressorConfig {
    /// Maximum files to include in output.
    pub max_files: usize,
    /// Maximum matches per file.
    pub max_matches_per_file: usize,
    /// Minimum match score to keep (0.0–1.0).
    pub min_match_score: f64,
    /// Whether to group results by file.
    pub group_by_file: bool,
}

impl Default for SearchCompressorConfig {
    fn default() -> Self {
        Self {
            max_files: DEFAULT_MAX_FILES,
            max_matches_per_file: DEFAULT_MAX_MATCHES_PER_FILE,
            min_match_score: DEFAULT_MIN_MATCH_SCORE,
            group_by_file: true,
        }
    }
}

/// A single match in grep output.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// File path where the match was found.
    pub file_path: String,
    /// Line number (0 if not available).
    pub line_number: usize,
    /// Column number (0 if not available).
    pub column: usize,
    /// The matching line content.
    pub line_content: String,
    /// Computed importance score (0.0–1.0).
    pub score: f64,
}

/// A group of matches from the same file.
#[derive(Debug, Clone)]
pub struct SearchFileGroup {
    /// File path.
    pub file_path: String,
    /// Matches in this file.
    pub matches: Vec<SearchMatch>,
    /// Aggregate score for the file (max + avg * 0.5).
    pub file_score: f64,
}

/// Result of search compression.
#[derive(Debug, Clone)]
pub struct SearchCompressionResult {
    /// Compressed output text.
    pub text: String,
    /// Number of files in original output.
    pub original_files: usize,
    /// Number of files in compressed output.
    pub compressed_files: usize,
    /// Number of matches in original output.
    pub original_matches: usize,
    /// Number of matches in compressed output.
    pub compressed_matches: usize,
}

/// The SearchCompressor.
#[derive(Debug, Clone, Default)]
pub struct SearchCompressor {
    pub config: SearchCompressorConfig,
}

impl SearchCompressor {
    /// Create a new SearchCompressor with default config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a SearchCompressor with a custom config.
    pub fn with_config(config: SearchCompressorConfig) -> Self {
        Self { config }
    }

    /// Compress grep/rg output.
    pub fn compress(&self, input: &str) -> Result<SearchCompressionResult> {
        if input.trim().is_empty() {
            return Ok(SearchCompressionResult {
                text: String::new(),
                original_files: 0,
                compressed_files: 0,
                original_matches: 0,
                compressed_matches: 0,
            });
        }

        // Parse matches
        let matches = self.parse_matches(input);
        let original_matches = matches.len();
        if matches.is_empty() {
            return Ok(SearchCompressionResult {
                text: input.to_string(),
                original_files: 0,
                compressed_files: 0,
                original_matches: 0,
                compressed_matches: 0,
            });
        }

        // Score each match
        let scored_matches: Vec<SearchMatch> = matches
            .into_iter()
            .map(|mut m| {
                m.score = self.score_match(&m);
                m
            })
            .collect();

        // Group by file
        let mut file_groups: std::collections::HashMap<String, Vec<SearchMatch>> =
            std::collections::HashMap::new();
        for m in scored_matches {
            if m.score < self.config.min_match_score {
                continue;
            }
            file_groups.entry(m.file_path.clone()).or_default().push(m);
        }

        let original_files = file_groups.len();

        // Score and sort files
        let mut scored_files: Vec<SearchFileGroup> = file_groups
            .into_iter()
            .map(|(path, mut matches)| {
                // Sort matches by score descending within file
                matches.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                matches.truncate(self.config.max_matches_per_file);

                let max_score = matches.first().map(|m| m.score).unwrap_or(0.0);
                let avg_score = if matches.is_empty() {
                    0.0
                } else {
                    matches.iter().map(|m| m.score).sum::<f64>() / matches.len() as f64
                };
                let file_score = max_score + avg_score * 0.5;

                SearchFileGroup {
                    file_path: path,
                    matches,
                    file_score,
                }
            })
            .collect();

        // Sort files by score descending
        scored_files.sort_by(|a, b| {
            b.file_score
                .partial_cmp(&a.file_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Select top files
        let compressed_files_count = scored_files.len().min(self.config.max_files);
        scored_files.truncate(self.config.max_files);

        // Render output
        let mut output_lines: Vec<String> = Vec::new();
        let mut compressed_matches = 0;

        for file in &scored_files {
            if self.config.group_by_file && !file.matches.is_empty() {
                // Render as grep-like output with file path header for first match
                output_lines.push(format!("{}:", file.file_path));
            }

            for m in &file.matches {
                compressed_matches += 1;
                if self.config.group_by_file {
                    if m.line_number > 0 {
                        output_lines.push(format!("{:>4}:{}", m.line_number, m.line_content));
                    } else {
                        output_lines.push(format!("       {}", m.line_content));
                    }
                } else {
                    if m.line_number > 0 {
                        output_lines.push(format!(
                            "{}:{}:{}",
                            m.file_path, m.line_number, m.line_content
                        ));
                    } else {
                        output_lines.push(format!("{}:{}", m.file_path, m.line_content));
                    }
                }
            }
        }

        Ok(SearchCompressionResult {
            text: output_lines.join("\n"),
            original_files,
            compressed_files: compressed_files_count,
            original_matches,
            compressed_matches,
        })
    }

    /// Parse grep/rg output into structured matches.
    fn parse_matches(&self, input: &str) -> Vec<SearchMatch> {
        let mut matches: Vec<SearchMatch> = Vec::new();

        for line in input.lines() {
            // Try column format first: file:line:col:content
            if let Some(caps) = GREP_COL_RE.captures(line) {
                matches.push(SearchMatch {
                    file_path: caps[1].to_string(),
                    line_number: caps[2].parse().unwrap_or(0),
                    column: caps[3].parse().unwrap_or(0),
                    line_content: caps[4].to_string(),
                    score: 0.0,
                });
                continue;
            }

            // Try standard format: file:line:content
            if let Some(caps) = GREP_LINE_RE.captures(line) {
                matches.push(SearchMatch {
                    file_path: caps[1].to_string(),
                    line_number: caps[2].parse().unwrap_or(0),
                    column: 0,
                    line_content: caps[3].to_string(),
                    score: 0.0,
                });
                continue;
            }

            // Try file:content format
            if let Some(caps) = GREP_FILE_CONTENT_RE.captures(line) {
                matches.push(SearchMatch {
                    file_path: caps[1].to_string(),
                    line_number: 0,
                    column: 0,
                    line_content: caps[2].to_string(),
                    score: 0.0,
                });
                continue;
            }

            // Unparseable line — skip (could be a separator or binary match message)
        }

        matches
    }

    /// Score a single search match based on content signals.
    fn score_match(&self, m: &SearchMatch) -> f64 {
        let mut score: f64 = 0.3;

        let content_lower = m.line_content.to_lowercase();

        // Error/failure signals
        if content_lower.contains("error")
            || content_lower.contains("fail")
            || content_lower.contains("exception")
            || content_lower.contains("panic")
            || content_lower.contains("crash")
        {
            score += 0.3;
        }

        // Warning signals
        if content_lower.contains("warning")
            || content_lower.contains("warn")
            || content_lower.contains("deprecated")
        {
            score += 0.15;
        }

        // Definition signals (code structure)
        if content_lower.contains("fn ")
            || content_lower.contains("def ")
            || content_lower.contains("class ")
            || content_lower.contains("struct ")
            || content_lower.contains("impl ")
            || content_lower.contains("trait ")
            || content_lower.contains("function ")
        {
            score += 0.2;
        }

        // Test signals
        if content_lower.contains("test")
            || content_lower.contains("spec")
            || content_lower.contains("it(")
            || content_lower.contains("describe(")
        {
            score += 0.1;
        }

        // Source file boost (by extension)
        let lower_path = m.file_path.to_lowercase();
        if lower_path.ends_with(".rs")
            || lower_path.ends_with(".py")
            || lower_path.ends_with(".ts")
            || lower_path.ends_with(".js")
            || lower_path.ends_with(".go")
            || lower_path.ends_with(".rs.bk")
        {
            score += 0.1;
        }

        // Very long lines are less useful
        if m.line_content.len() > 200 {
            score -= 0.1;
        }

        // Comment-only lines are less important
        let trimmed = m.line_content.trim();
        if trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            score -= 0.15;
        }

        // Clamp to [0.0, 1.0]
        score.clamp(0.0, 1.0)
    }
}

/// Compress grep/rg output with default configuration.
pub fn compress_search(input: &str) -> Result<String> {
    let compressor = SearchCompressor::new();
    let result = compressor.compress(input)?;
    Ok(result.text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_compressor_empty() {
        let compressor = SearchCompressor::new();
        let result = compressor.compress("").unwrap();
        assert!(result.text.is_empty());
        assert_eq!(result.original_matches, 0);
    }

    #[test]
    fn test_search_compressor_parse_standard() {
        let input = "src/main.rs:42:fn main() {\nsrc/lib.rs:10:pub fn helper()\n";
        let matches = SearchCompressor::new().parse_matches(input);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].file_path, "src/main.rs");
        assert_eq!(matches[0].line_number, 42);
        assert_eq!(matches[0].line_content, "fn main() {");
    }

    #[test]
    fn test_search_compressor_parse_column_format() {
        let input = "src/main.rs:42:5:fn main() {\n";
        let matches = SearchCompressor::new().parse_matches(input);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].column, 5);
    }

    #[test]
    fn test_search_compressor_score_error_boost() {
        let compressor = SearchCompressor::new();
        let error_match = SearchMatch {
            file_path: "src/main.rs".to_string(),
            line_number: 1,
            column: 0,
            line_content: "fn main() -> Result<()> {".to_string(),
            score: 0.0,
        };
        let error_score = compressor.score_match(&error_match);
        // Base 0.3 + definition boost (fn) 0.2 + .rs source boost 0.1 = 0.6
        assert!(
            (error_score - 0.6).abs() < 0.01,
            "Expected ~0.6, got {}",
            error_score
        );
    }

    #[test]
    fn test_search_compressor_score_error_keyword() {
        let compressor = SearchCompressor::new();
        let m = SearchMatch {
            file_path: "src/main.rs".to_string(),
            line_number: 1,
            column: 0,
            line_content: "error: something went wrong".to_string(),
            score: 0.0,
        };
        let score = compressor.score_match(&m);
        // Base 0.3 + error 0.3 + .rs source boost 0.1 = 0.7
        assert!((score - 0.7).abs() < 0.01, "Expected ~0.7, got {}", score);
    }

    #[test]
    fn test_search_compressor_group_and_select() {
        let input = "\
src/main.rs:1:fn main() {
src/main.rs:2:    let x = 1;
src/lib.rs:10:pub fn helper() {
src/lib.rs:11:    // comment
src/lib.rs:12:    helper()
";
        let compressor = SearchCompressor::with_config(SearchCompressorConfig {
            max_files: 2,
            max_matches_per_file: 2,
            ..Default::default()
        });
        let result = compressor.compress(input).unwrap();
        assert_eq!(result.original_files, 2);
        assert_eq!(result.original_matches, 5);
        // Should have at most 4 compressed matches (2 files × 2 matches)
        assert!(
            result.compressed_matches <= 4,
            "Should limit matches per file"
        );
    }

    #[test]
    fn test_search_compressor_min_score_filter() {
        let input = "data/notes.txt:1:// comment only line\n";
        let compressor = SearchCompressor::with_config(SearchCompressorConfig {
            min_match_score: 0.2,
            ..Default::default()
        });
        let result = compressor.compress(input).unwrap();
        // Comment-only line in non-source file: base 0.3 - comment 0.15 = 0.15, below 0.2
        assert_eq!(result.compressed_matches, 0);
    }

    #[test]
    fn test_search_compressor_non_grouped_output() {
        let input = "src/main.rs:1:fn main() {\nsrc/lib.rs:10:pub fn helper()\n";
        let compressor = SearchCompressor::with_config(SearchCompressorConfig {
            group_by_file: false,
            ..Default::default()
        });
        let result = compressor.compress(input).unwrap();
        assert_eq!(result.compressed_matches, 2);
        assert!(result.text.contains("src/main.rs:1:"));
        assert!(result.text.contains("src/lib.rs:10:"));
    }
}
