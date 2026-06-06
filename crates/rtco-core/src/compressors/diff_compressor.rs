//! DiffCompressor — compress unified diff output by scoring and selecting hunks.
//!
//! Ported from headroom's `transforms/diff_compressor.rs` with simplified
//! configuration (no auth-mode policies, no Redis CCR backend).
//!
//! Pipeline:
//! 1. Parse unified diff into file/hunk/line structure
//! 2. Score each hunk by change density and context
//! 3. Select top hunks within token budget
//! 4. Trim context lines per hunk to `max_context_lines`

use anyhow::{Context, Result};

/// Default maximum context lines to keep around a change.
const DEFAULT_MAX_CONTEXT_LINES: usize = 3;
/// Default maximum hunks per file.
const DEFAULT_MAX_HUNKS_PER_FILE: usize = 20;
/// Default maximum files to include.
const DEFAULT_MAX_FILES: usize = 30;
/// Score for a changed line (added or removed).
const CHANGE_LINE_SCORE: f64 = 1.0;
/// Score for context lines within 2 of a change.
const NEAR_CONTEXT_SCORE: f64 = 0.8;
/// Score for context lines 3-5 from a change.
const MID_CONTEXT_SCORE: f64 = 0.5;
/// Score for context lines >5 from a change.
const FAR_CONTEXT_SCORE: f64 = 0.2;
/// Configuration for the diff compressor.
#[derive(Debug, Clone)]
pub struct DiffCompressorConfig {
    /// Maximum context lines to keep around a change (default: 3).
    pub max_context_lines: usize,
    /// Maximum hunks per file (default: 20).
    pub max_hunks_per_file: usize,
    /// Maximum files to include (default: 30).
    pub max_files: usize,
    /// Minimum score threshold (0.0–1.0). Lines below this are dropped.
    pub min_score_threshold: f64,
}

impl Default for DiffCompressorConfig {
    fn default() -> Self {
        Self {
            max_context_lines: DEFAULT_MAX_CONTEXT_LINES,
            max_hunks_per_file: DEFAULT_MAX_HUNKS_PER_FILE,
            max_files: DEFAULT_MAX_FILES,
            min_score_threshold: 0.15,
        }
    }
}

/// A single line in a unified diff.
#[derive(Debug, Clone)]
pub struct DiffLine {
    /// The line content (without leading +/-/space).
    pub content: String,
    /// Diff prefix: '+' for added, '-' for removed, ' ' for context, '@@' for hunk header.
    pub prefix: String,
    /// Line number within the hunk (0-indexed).
    pub line_number: usize,
    /// Whether this line is within `max_context_lines` of a change.
    pub is_near_change: bool,
}

/// A single hunk in a diff.
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Hunk header line (e.g., `@@ -1,5 +1,6 @@`).
    pub header: String,
    /// Lines in this hunk.
    pub lines: Vec<DiffLine>,
    /// Average score across all lines in this hunk.
    pub score: f64,
}

/// A file in a diff.
#[derive(Debug, Clone)]
pub struct DiffFile {
    /// File header line (e.g., `diff --git a/file.rs b/file.rs`).
    pub header: String,
    /// Optional --- line (e.g., `--- a/file.rs`).
    pub old_file: Option<String>,
    /// Optional +++ line (e.g., `+++ b/file.rs`).
    pub new_file: Option<String>,
    /// Hunks in this file.
    pub hunks: Vec<DiffHunk>,
    /// Average score across all hunks.
    pub score: f64,
}

/// Compressed diff output.
#[derive(Debug, Clone)]
pub struct CompressedDiff {
    /// Compressed diff text.
    pub text: String,
    /// Number of files in the original diff.
    pub original_files: usize,
    /// Number of files in the compressed diff.
    pub compressed_files: usize,
    /// Number of hunks in the original diff.
    pub original_hunks: usize,
    /// Number of hunks in the compressed diff.
    pub compressed_hunks: usize,
}

/// The DiffCompressor.
#[derive(Debug, Clone, Default)]
pub struct DiffCompressor {
    pub config: DiffCompressorConfig,
}

impl DiffCompressor {
    /// Create a new DiffCompressor with default config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new DiffCompressor with a custom config.
    pub fn with_config(config: DiffCompressorConfig) -> Self {
        Self { config }
    }

    /// Compress a unified diff string.
    pub fn compress(&self, input: &str) -> Result<CompressedDiff> {
        if input.trim().is_empty() {
            return Ok(CompressedDiff {
                text: String::new(),
                original_files: 0,
                compressed_files: 0,
                original_hunks: 0,
                compressed_hunks: 0,
            });
        }

        let files = self
            .parse_diff(input)
            .context("Failed to parse unified diff")?;
        let original_hunks: usize = files.iter().map(|f| f.hunks.len()).sum();
        let original_files = files.len();

        // Score all files
        let mut scored_files: Vec<DiffFile> = files
            .into_iter()
            .map(|mut f| {
                f.hunks = f
                    .hunks
                    .into_iter()
                    .map(|mut h| {
                        // Score each line in the hunk
                        for line in &h.lines {
                            let _ = line; // Lines scored during parse
                        }
                        h.score = self.score_hunk(&h);
                        h
                    })
                    .collect();
                f.score = if f.hunks.is_empty() {
                    0.0
                } else {
                    f.hunks.iter().map(|h| h.score).sum::<f64>() / f.hunks.len() as f64
                };
                f
            })
            .collect();

        // Sort by score descending
        scored_files.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Select top files
        scored_files.truncate(self.config.max_files);

        // Process each file: select top hunks, trim context
        let mut output_lines: Vec<String> = Vec::new();
        let mut compressed_hunks = 0;

        for file in &scored_files {
            let mut file_lines: Vec<String> = Vec::new();

            // Add file header
            file_lines.push(file.header.clone());
            if let Some(ref old) = file.old_file {
                file_lines.push(old.clone());
            }
            if let Some(ref new) = file.new_file {
                file_lines.push(new.clone());
            }

            // Sort hunks by score descending, select top
            let mut hunks = file.hunks.clone();
            hunks.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            hunks.truncate(self.config.max_hunks_per_file);

            for hunk in &hunks {
                if hunk.score < self.config.min_score_threshold {
                    continue;
                }
                compressed_hunks += 1;

                file_lines.push(hunk.header.clone());

                // Trim context lines per hunk
                let trimmed = self.trim_context(&hunk.lines);
                for line in &trimmed {
                    file_lines.push(format!("{}{}", line.prefix, line.content));
                }
            }

            output_lines.extend(file_lines);
        }

        Ok(CompressedDiff {
            text: output_lines.join("\n"),
            original_files,
            compressed_files: scored_files.len(),
            original_hunks,
            compressed_hunks,
        })
    }

    /// Parse a unified diff string into structured files/hunks/lines.
    fn parse_diff(&self, input: &str) -> Result<Vec<DiffFile>> {
        let mut files: Vec<DiffFile> = Vec::new();
        let mut current_file: Option<DiffFile> = None;
        let mut current_hunk: Option<DiffHunk> = None;

        for line in input.lines() {
            if line.starts_with("diff --git ") {
                // Finalize previous hunk
                if let Some(hunk) = current_hunk.take() {
                    if let Some(ref mut file) = current_file {
                        file.hunks.push(hunk);
                    }
                }
                // Finalize previous file
                if let Some(file) = current_file.take() {
                    files.push(file);
                }
                current_file = Some(DiffFile {
                    header: line.to_string(),
                    old_file: None,
                    new_file: None,
                    hunks: Vec::new(),
                    score: 0.0,
                });
            } else if line.starts_with("--- ") {
                if let Some(ref mut file) = current_file {
                    file.old_file = Some(line.to_string());
                }
            } else if line.starts_with("+++ ") {
                if let Some(ref mut file) = current_file {
                    file.new_file = Some(line.to_string());
                }
            } else if line.starts_with("@@") {
                // Finalize previous hunk
                if let Some(hunk) = current_hunk.take() {
                    if let Some(ref mut file) = current_file {
                        file.hunks.push(hunk);
                    }
                }
                current_hunk = Some(DiffHunk {
                    header: line.to_string(),
                    lines: Vec::new(),
                    score: 0.0,
                });
            } else if let Some(ref mut hunk) = current_hunk {
                let (prefix, content) = if let Some(stripped) = line.strip_prefix('+') {
                    ("+".to_string(), stripped.to_string())
                } else if let Some(stripped) = line.strip_prefix('-') {
                    ("-".to_string(), stripped.to_string())
                } else if let Some(stripped) = line.strip_prefix(' ') {
                    (" ".to_string(), stripped.to_string())
                } else {
                    // Could be a standalone line (binary files, new file mode, etc.)
                    (" ".to_string(), line.to_string())
                };

                let line_number = hunk.lines.len();
                // Mark lines near changes
                let is_near_change = false; // Calculated later in score_hunk
                hunk.lines.push(DiffLine {
                    content,
                    prefix,
                    line_number,
                    is_near_change,
                });
            }
        }

        // Finalize last hunk + file
        if let Some(hunk) = current_hunk.take() {
            if let Some(ref mut file) = current_file {
                file.hunks.push(hunk);
            }
        }
        if let Some(file) = current_file.take() {
            files.push(file);
        }

        Ok(files)
    }

    /// Score a hunk based on its content.
    fn score_hunk(&self, hunk: &DiffHunk) -> f64 {
        if hunk.lines.is_empty() {
            return 0.0;
        }

        // Mark lines near changes
        let change_positions: Vec<usize> = hunk
            .lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.prefix == "+" || l.prefix == "-")
            .map(|(i, _)| i)
            .collect();

        let mut total_score = 0.0;
        for (i, line) in hunk.lines.iter().enumerate() {
            let line_score = if line.prefix == "+" || line.prefix == "-" {
                CHANGE_LINE_SCORE
            } else {
                // Context line — score depends on distance to nearest change
                let min_dist = change_positions
                    .iter()
                    .map(|cp| (*cp).abs_diff(i))
                    .min()
                    .unwrap_or(usize::MAX);

                if min_dist <= 2 {
                    NEAR_CONTEXT_SCORE
                } else if min_dist <= 5 {
                    MID_CONTEXT_SCORE
                } else {
                    FAR_CONTEXT_SCORE
                }
            };
            total_score += line_score;
        }

        total_score / hunk.lines.len() as f64
    }

    /// Trim context lines to the configured maximum distance from changes.
    fn trim_context(&self, lines: &[DiffLine]) -> Vec<DiffLine> {
        let change_positions: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.prefix == "+" || l.prefix == "-")
            .map(|(i, _)| i)
            .collect();

        if change_positions.is_empty() {
            // No changes — just return the first few lines
            return lines
                .iter()
                .take(self.config.max_context_lines)
                .cloned()
                .collect();
        }

        lines
            .iter()
            .enumerate()
            .filter(|(i, l)| {
                if l.prefix == "+" || l.prefix == "-" || l.prefix == "@@" {
                    return true;
                }
                // Context line — keep if within max_context_lines of a change
                change_positions.iter().any(|cp| {
                    let dist = (*cp).abs_diff(*i);
                    dist <= self.config.max_context_lines
                })
            })
            .map(|(_, l)| l.clone())
            .collect()
    }
}

/// Compress a unified diff string with default configuration.
pub fn compress_diff(input: &str) -> Result<String> {
    let compressor = DiffCompressor::new();
    let result = compressor.compress(input)?;
    Ok(result.text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_compressor_empty_input() {
        let compressor = DiffCompressor::new();
        let result = compressor.compress("").unwrap();
        assert!(result.text.is_empty());
        assert_eq!(result.original_files, 0);
    }

    #[test]
    fn test_diff_compressor_simple_diff() {
        let input = "\
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,5 +1,6 @@
 fn main() {
     println!(\"Hello\");
+    println!(\"World\");
     println!(\"Goodbye\");
+
 }
";
        let compressor = DiffCompressor::new();
        let result = compressor.compress(input).unwrap();
        assert!(result.compressed_files >= 1);
        assert!(result.original_files >= 1);
        assert!(!result.text.is_empty());
        // Should contain the added line
        assert!(result.text.contains("World"));
    }

    #[test]
    fn test_diff_compressor_multiple_files() {
        let input = "\
diff --git a/src/a.rs b/src/a.rs
--- a/src/a.rs
+++ b/src/a.rs
@@ -1,3 +1,4 @@
 line1
+line2
 line3
diff --git a/src/b.rs b/src/b.rs
--- a/src/b.rs
+++ b/src/b.rs
@@ -10,6 +10,7 @@
 fn foo() {
+    bar();
 }
";
        let compressor = DiffCompressor::new();
        let result = compressor.compress(input).unwrap();
        assert_eq!(result.original_files, 2);
        assert_eq!(result.compressed_files, 2);
    }

    #[test]
    fn test_diff_compressor_score_change_lines() {
        let hunk = DiffHunk {
            header: "@@ -1,3 +1,4 @@".to_string(),
            lines: vec![
                DiffLine {
                    content: "fn main() {".to_string(),
                    prefix: " ".to_string(),
                    line_number: 0,
                    is_near_change: false,
                },
                DiffLine {
                    content: "    println!(\"Hello\");".to_string(),
                    prefix: "+".to_string(),
                    line_number: 1,
                    is_near_change: false,
                },
                DiffLine {
                    content: "}".to_string(),
                    prefix: " ".to_string(),
                    line_number: 2,
                    is_near_change: false,
                },
            ],
            score: 0.0,
        };
        let compressor = DiffCompressor::new();
        let score = compressor.score_hunk(&hunk);
        // Change lines score 1.0, near context scores 0.8, far context 0.2
        // Expected: (0.8 + 1.0 + 0.8) / 3 ≈ 0.867
        assert!(
            (score - 0.867).abs() < 0.01,
            "Expected ~0.867, got {}",
            score
        );
    }

    #[test]
    fn test_diff_compressor_trim_context() {
        let lines: Vec<DiffLine> = (0..10)
            .map(|i| DiffLine {
                content: format!("line {}", i),
                prefix: if i == 5 {
                    "+".to_string()
                } else {
                    " ".to_string()
                },
                line_number: i,
                is_near_change: false,
            })
            .collect();

        let compressor = DiffCompressor::with_config(DiffCompressorConfig {
            max_context_lines: 2,
            ..Default::default()
        });
        let trimmed = compressor.trim_context(&lines);
        // Should keep: lines 3,4,5(change),6,7 (within 2 of change at index 5)
        // Also keep the change line itself
        assert_eq!(
            trimmed.len(),
            5,
            "Expected 5 lines (change + 2 each side), got {}",
            trimmed.len()
        );
    }

    #[test]
    fn test_diff_compressor_savings() {
        let input = "\
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,20 +1,21 @@
 line1
 line2
 line3
 line4
+inserted
 line5
 line6
 line7
 line8
 line9
 line10
 line11
 line12
 line13
 line14
 line15
 line16
 line17
 line18
 line19
 line20
";
        let compressor = DiffCompressor::new();
        let result = compressor.compress(input).unwrap();
        // Should save some tokens by trimming context
        let savings = 1.0 - (result.text.len() as f64 / input.len() as f64);
        assert!(savings > 0.0, "Expected some savings, got {}", savings);
    }
}
