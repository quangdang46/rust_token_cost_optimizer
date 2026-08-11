//! Anchor type detection — classify lines into anchor categories.
//!
//! Each anchor type has a regex-based detector and a default priority.

use std::sync::LazyLock;

use regex::Regex;

static HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#{1,6}\s|={3,}|-{3,}|___)").unwrap());
static COMMAND_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\$ |% |> |\||`).+").unwrap());
static PATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:^|\s)(?:/[a-z0-9_./-]+|[a-zA-Z]:\\[a-z0-9_\.\\-]+|[a-z0-9_./-]+\.[a-z]+)")
        .unwrap()
});
static KEY_VALUE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*\s*[=:]\s*.+").unwrap());
static DEFINITION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:pub\s+)?(?:fn|def|class|struct|enum|trait|impl|function|function\s|async\s+fn|const|let|type|interface|typealias)\s+").unwrap()
});
static SUMMARY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:^summary|tests:|results:|total:|passed:|failed:|errors?:\s+\d|^\d+ passed|^\d+ failed)").unwrap()
});
static ARROW_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^---+$").unwrap());

/// Type of detected anchor.
#[derive(Debug, Clone, PartialEq)]
pub enum AnchorType {
    /// Section headers, Markdown H1-H6, horizontal rules
    Header,
    /// Shell commands ($, %, >, |, `)
    Command,
    /// File paths, URLs
    Path,
    /// key=value or key: value lines
    KeyValue,
    /// Code definitions (fn, def, class, struct, etc.)
    Definition,
    /// Summary/statistics lines
    Summary,
}

impl AnchorType {
    /// Default priority for this anchor type (1.0 = highest).
    pub fn default_priority(&self) -> f64 {
        match self {
            AnchorType::Header => 0.9,
            AnchorType::Command => 0.8,
            AnchorType::Path => 0.6,
            AnchorType::KeyValue => 0.5,
            AnchorType::Definition => 0.85,
            AnchorType::Summary => 0.75,
        }
    }
}

/// Detect the anchor type of a trimmed line.
pub fn detect_anchor_type(line: &str) -> Option<AnchorType> {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Headers
    if HEADER_RE.is_match(trimmed) || ARROW_RE.is_match(trimmed) {
        return Some(AnchorType::Header);
    }

    // Summary lines (check early since "1 passed" could match KeyValue)
    if SUMMARY_RE.is_match(trimmed) {
        return Some(AnchorType::Summary);
    }

    // Definitions (check before KeyValue since `fn name = ...` could match both)
    if DEFINITION_RE.is_match(trimmed) {
        return Some(AnchorType::Definition);
    }

    // Commands
    if COMMAND_RE.is_match(trimmed) {
        return Some(AnchorType::Command);
    }

    // Key=Value or Key: Value
    if KEY_VALUE_RE.is_match(trimmed) {
        return Some(AnchorType::KeyValue);
    }

    // File paths (must contain / or .ext)
    if PATH_RE.is_match(trimmed) && (trimmed.contains('/') || trimmed.contains('\\')) {
        return Some(AnchorType::Path);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_header() {
        assert_eq!(detect_anchor_type("# Title"), Some(AnchorType::Header));
        assert_eq!(detect_anchor_type("## Subtitle"), Some(AnchorType::Header));
        assert_eq!(detect_anchor_type("---"), Some(AnchorType::Header));
    }

    #[test]
    fn test_detect_definition() {
        assert_eq!(
            detect_anchor_type("fn main()"),
            Some(AnchorType::Definition)
        );
        assert_eq!(
            detect_anchor_type("pub fn helper()"),
            Some(AnchorType::Definition)
        );
        assert_eq!(
            detect_anchor_type("class Foo"),
            Some(AnchorType::Definition)
        );
        assert_eq!(
            detect_anchor_type("struct Bar"),
            Some(AnchorType::Definition)
        );
        assert_eq!(
            detect_anchor_type("  pub fn method()"),
            Some(AnchorType::Definition)
        );
    }

    #[test]
    fn test_detect_command() {
        assert_eq!(
            detect_anchor_type("$ cargo build"),
            Some(AnchorType::Command)
        );
        assert_eq!(detect_anchor_type("> output"), Some(AnchorType::Command));
    }

    #[test]
    fn test_detect_key_value() {
        assert_eq!(
            detect_anchor_type("name = value"),
            Some(AnchorType::KeyValue)
        );
        assert_eq!(detect_anchor_type("key: value"), Some(AnchorType::KeyValue));
    }

    #[test]
    fn test_detect_summary() {
        assert_eq!(detect_anchor_type("1 passed"), Some(AnchorType::Summary));
        assert_eq!(
            detect_anchor_type("Tests: 10 passed"),
            Some(AnchorType::Summary)
        );
        assert_eq!(detect_anchor_type("error: 2"), Some(AnchorType::Summary));
    }

    #[test]
    fn test_detect_path() {
        assert_eq!(detect_anchor_type("/usr/bin/env"), Some(AnchorType::Path));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(detect_anchor_type("some random text"), None);
        assert_eq!(detect_anchor_type(""), None);
    }
}
