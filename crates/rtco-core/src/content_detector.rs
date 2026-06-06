//! Content type detection for CLI output.
//!
//! Classifies raw CLI output into one of several content types so that
//! downstream filters can choose the optimal compression strategy.  Detection
//! is heuristic-based and operates on the first few kilobytes of input to
//! keep latency negligible.
//!
//! # Usage
//!
//! ```no_run
//! use rtco_core::content_detector::{detect_content_type, ContentType};
//!
//! let kind = detect_content_type(r#"[{"name":"rtco","version":"0.28.2"}]"#);
//! assert_eq!(kind, ContentType::JsonArray);
//! ```

use lazy_static::lazy_static;
use regex::Regex;

// ---------------------------------------------------------------------------
// Content types
// ---------------------------------------------------------------------------

/// Classifies CLI output so that filters can pick the best strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    /// A JSON array (starts with `[`, ends with `]`).
    JsonArray,
    /// Source code in any language.
    SourceCode,
    /// Search / grep result output (file:line:match patterns).
    SearchResults,
    /// Compiler / build tool output (error[E0001], warning:, FAILED, etc.).
    BuildOutput,
    /// Unified diff / git diff output.
    GitDiff,
    /// HTML markup.
    Html,
    /// Anything that does not match a more specific type.
    PlainText,
}

// ---------------------------------------------------------------------------
// Lazily compiled regexes
// ---------------------------------------------------------------------------

lazy_static! {
    /// JSON array: leading whitespace then `[` at the start, `]` at the end
    /// (after trimming).  We only look at the first/last non-space char.
    static ref JSON_ARRAY_OPEN: Regex = Regex::new(r"^\s*\[").unwrap();
    static ref JSON_ARRAY_CLOSE: Regex = Regex::new(r"\]\s*$").unwrap();

    /// JSON object -- useful as a secondary signal for `SourceCode` false
    /// positives (a `{` ... `}` block that looks like code may actually be
    /// JSON).
    static ref JSON_OBJECT_OPEN: Regex = Regex::new(r"^\s*\{").unwrap();
    static ref JSON_OBJECT_CLOSE: Regex = Regex::new(r"\}\s*$").unwrap();

    /// Diff hunk header: `--- a/file`, `+++ b/file`, `@@ -1,3 +1,5 @@`,
    /// and also handles `--- original_file` / `+++ modified_file` without `a/` prefix.
    static ref DIFF_HEADER: Regex =
        Regex::new(r"^diff --git |^--- (?:[ab]/)?\S|^\+\+\+ (?:[ab]/)?\S|^@@\s+-?\d+").unwrap();
    static ref DIFF_HUNK: Regex =
        Regex::new(r"^@@\s+-?\d+").unwrap();

    /// Search / grep result: `file.ext:NN:...` or `file.ext-NN-...`.
    static ref SEARCH_RESULT: Regex =
        Regex::new(r"^\S+\.(?:rs|py|js|ts|go|java|kt|swift|c|cpp|cxx|h|hpp|hh|rb|toml|yaml|yml|json|md|txt|sh|bash|zsh|fish|ps1|css|scss|less|php|sql|xml|conf|cfg|ini|env|log|r|lua|hs|ex|exs|vue|svelte|tex|svg|gradle|sbt|makefile|dockerfile):\d+[:\-]").unwrap();

    /// HTML tag at the start of input.
    static ref HTML_TAG: Regex = Regex::new(r"(?i)^\s*<!DOCTYPE\s+html|^\s*<html[\s>]").unwrap();

    /// Build/compiler output indicators.  Each alternative is tested per-line
    /// (the function does per-line matching, not multi-line, so `^` means
    /// start-of-string which equals start-of-line for each trimmed line).
    static ref BUILD_ERROR: Regex =
        Regex::new(r"(?i)((error|warning|fatal)(\[|:))|\berror\[E\d+\]|FAILED|Build failed|Compilation failed|cannot find|undefined reference").unwrap();

    /// Source code indicator keywords that appear at the start of a line
    /// (after optional leading whitespace).
    /// Handles both `pub fn` and `pub(crate) fn` / `pub(super) fn` etc.
    static ref SOURCE_CODE_LINE: Regex = Regex::new(
        r"^(pub(?:\s+|\([^)]*\)\s*)?)?(fn |func |def |class |import |from |package |module |namespace |struct |enum |trait |impl |type |const |let |var |async |use |extern |static |#\[|//|/\*|\*/)"
    ).unwrap();
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

/// Detect the [`ContentType`] of a block of CLI output.
///
/// The function examines structural markers (first/last characters, common
/// prefixes) and uses a scoring heuristic so that the most specific type wins.
/// When no specific pattern matches, [`ContentType::PlainText`] is returned.
pub fn detect_content_type(input: &str) -> ContentType {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return ContentType::PlainText;
    }

    // -- 1. Diff (very distinctive prefix) --
    if DIFF_HEADER.is_match(trimmed) {
        return ContentType::GitDiff;
    }

    // -- 2. HTML (check before code, because HTML looks like code) --
    if HTML_TAG.is_match(trimmed) {
        return ContentType::Html;
    }

    // -- 3. JSON array --
    if JSON_ARRAY_OPEN.is_match(trimmed) && JSON_ARRAY_CLOSE.is_match(trimmed) {
        return ContentType::JsonArray;
    }

    // -- 4. JSON object (not in the enum, but treat as PlainText to avoid
    //      misclassifying it as SourceCode) --
    if JSON_OBJECT_OPEN.is_match(trimmed) && JSON_OBJECT_CLOSE.is_match(trimmed) {
        // Could be JSON object or a code block.  Heuristic: if the content
        // contains `"key":` patterns it is likely JSON, not code.
        let quote_colon_count = trimmed.matches("\":").count();
        if quote_colon_count >= 2 {
            return ContentType::PlainText;
        }
    }

    // -- 5. Build output --
    let has_build = input.lines().any(|l| BUILD_ERROR.is_match(l.trim()));
    if has_build {
        return ContentType::BuildOutput;
    }

    // -- 6. Search results --
    let search_lines = input
        .lines()
        .filter(|l| SEARCH_RESULT.is_match(l.trim()))
        .count();
    let total_lines = input.lines().count().max(1);
    // If more than half the lines match grep-style output, classify as search
    if search_lines > 0 && (search_lines as f64 / total_lines as f64) >= 0.3 {
        return ContentType::SearchResults;
    }

    // -- 7. Source code (per-line match after trimming leading whitespace) --
    let code_lines = input
        .lines()
        .filter(|l| SOURCE_CODE_LINE.is_match(l.trim_start()))
        .take(2)
        .count();
    if code_lines >= 2 {
        return ContentType::SourceCode;
    }

    // -- 8. Fallback --
    ContentType::PlainText
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- JSON Array --

    #[test]
    fn json_array_basic() {
        let input = r#"[{"name":"rtco","version":"0.28.2"},{"name":"rtk","version":"0.1.0"}]"#;
        assert_eq!(detect_content_type(input), ContentType::JsonArray);
    }

    #[test]
    fn json_array_with_whitespace() {
        let input = r#"
        [
          {"key": "value"},
          {"key": "other"}
        ]
        "#;
        assert_eq!(detect_content_type(input), ContentType::JsonArray);
    }

    #[test]
    fn json_array_empty() {
        let input = "[]";
        assert_eq!(detect_content_type(input), ContentType::JsonArray);
    }

    // -- Git Diff --

    #[test]
    fn git_diff_header() {
        let input = "\
diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdef0 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,6 +10,8 @@ fn main() {
     println!(\"hello\");
+    println!(\"world\");
 }";
        assert_eq!(detect_content_type(input), ContentType::GitDiff);
    }

    #[test]
    fn git_diff_hunk_only() {
        let input = "@@ -1,3 +1,5 @@\n+added\n-removed";
        assert_eq!(detect_content_type(input), ContentType::GitDiff);
    }

    // -- HTML --

    #[test]
    fn html_doctype() {
        let input = "<!DOCTYPE html><html><head><title>T</title></head><body></body></html>";
        assert_eq!(detect_content_type(input), ContentType::Html);
    }

    #[test]
    fn html_tag() {
        let input = "<html lang=\"en\"><body><p>Hello</p></body></html>";
        assert_eq!(detect_content_type(input), ContentType::Html);
    }

    // -- Build Output --

    #[test]
    fn build_output_rust_error() {
        let input = "error[E0308]: mismatched types\n  --> src/main.rs:10:5";
        assert_eq!(detect_content_type(input), ContentType::BuildOutput);
    }

    #[test]
    fn build_output_cargo_failed() {
        let input = "   Compiling rtco v0.28.2\nerror: build failed\nFAILED to compile";
        assert_eq!(detect_content_type(input), ContentType::BuildOutput);
    }

    #[test]
    fn build_output_warning() {
        let input = "warning: unused variable `x`";
        assert_eq!(detect_content_type(input), ContentType::BuildOutput);
    }

    // -- Search Results --

    #[test]
    fn search_results_grep_output() {
        let input = "\
src/main.rs:10:fn main() {
src/main.rs:11:    println!(\"hello\");
src/lib.rs:5:pub fn helper() -> i32 {";
        assert_eq!(detect_content_type(input), ContentType::SearchResults);
    }

    #[test]
    fn search_results_with_line_numbers() {
        let input = "README.md:1:# Project\nREADME.md:2:Some description";
        assert_eq!(detect_content_type(input), ContentType::SearchResults);
    }

    // -- Source Code --

    #[test]
    fn source_code_rust() {
        let input = "\
pub fn main() {
    let x = 42;
    println!(\"{}\", x);
}";
        assert_eq!(detect_content_type(input), ContentType::SourceCode);
    }

    #[test]
    fn source_code_python() {
        let input = "\
import os
from pathlib import Path

def main():
    print(\"hello\")";
        assert_eq!(detect_content_type(input), ContentType::SourceCode);
    }

    // -- PlainText --

    #[test]
    fn plain_text_simple() {
        let input = "Hello, world!";
        assert_eq!(detect_content_type(input), ContentType::PlainText);
    }

    #[test]
    fn plain_text_empty() {
        assert_eq!(detect_content_type(""), ContentType::PlainText);
    }

    #[test]
    fn plain_text_whitespace_only() {
        assert_eq!(detect_content_type("   \n  \n  "), ContentType::PlainText);
    }

    #[test]
    fn plain_text_json_object_not_array() {
        let input = r#"{"name":"rtco","version":"0.28.2"}"#;
        assert_eq!(detect_content_type(input), ContentType::PlainText);
    }

    // -- Edge cases --

    #[test]
    fn unicode_input() {
        let input = "commit 日本語メッセージ";
        // Should not panic
        let kind = detect_content_type(input);
        assert_eq!(kind, ContentType::PlainText);
    }

    #[test]
    fn ansi_colored_input() {
        let input = "\x1b[32mSuccess\x1b[0m: build completed";
        // Should not panic, and classify as PlainText (no specific markers)
        let kind = detect_content_type(input);
        assert_eq!(kind, ContentType::PlainText);
    }

    #[test]
    fn source_code_rust_pub_crate_fn() {
        let input = "\
pub(crate) fn helper() -> i32 {
    42
}
pub(super) fn internal() {}";
        assert_eq!(detect_content_type(input), ContentType::SourceCode);
    }

    #[test]
    fn source_code_rust_module() {
        let input = "\
mod foo;
use bar::Baz;
const X: i32 = 1;";
        assert_eq!(detect_content_type(input), ContentType::SourceCode);
    }

    #[test]
    fn source_code_wins_over_build_heuristic() {
        // Source code with multiple code-like lines should be SourceCode
        let input = "\
// TODO: fix this later
fn main() {
    let x = 1;
}";
        assert_eq!(detect_content_type(input), ContentType::SourceCode);
    }

    #[test]
    fn diff_without_ab_prefix() {
        let input = "\
--- original_file
+++ modified_file
@@ -1,3 +1,5 @@
+added
-removed";
        assert_eq!(detect_content_type(input), ContentType::GitDiff);
    }

    #[test]
    fn build_output_priority_over_code() {
        // A file with both code-like lines and error lines should prefer BuildOutput
        let input = "\
fn main() {
    let x = 42;
}
error[E0308]: mismatched types";
        assert_eq!(detect_content_type(input), ContentType::BuildOutput);
    }
}
