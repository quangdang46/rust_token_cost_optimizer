//! Compares two files and shows only the changed lines.

use rtco_core::tracking;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Ultra-condensed diff - only changed lines, no context.
///
/// Exit-code contract aligned with GNU `diff` (issue rtco#1918):
/// - `Ok(0)` when files are identical.
/// - `Ok(1)` when files differ.
/// - `Ok(2)` on I/O errors such as missing files.
///
/// The "files are identical" status message is written to stderr (not stdout)
/// so scripts redirecting stdout into `patch`, `git apply`, or similar
/// consumers don't see decorative text mixed into the patch stream.
pub fn run(file1: &Path, file2: &Path, verbose: u8) -> Result<i32> {
    let timer = tracking::TimedExecution::start();

    if verbose > 0 {
        eprintln!("Comparing: {} vs {}", file1.display(), file2.display());
    }

    let content1 = match fs::read_to_string(file1) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("rtco diff: {}: {}", file1.display(), e);
            return Ok(2);
        }
    };
    let content2 = match fs::read_to_string(file2) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("rtco diff: {}: {}", file2.display(), e);
            return Ok(2);
        }
    };
    let raw = format!("{}\n---\n{}", content1, content2);

    // Use split('\n') instead of lines() to preserve \r characters at line endings.
    // str::lines() strips trailing \r, which makes CRLF vs LF files appear identical (#2627).
    let lines1: Vec<&str> = content1.split('\n').collect();
    let lines2: Vec<&str> = content2.split('\n').collect();
    let diff = compute_diff(&lines1, &lines2);

    if diff.added == 0 && diff.removed == 0 && diff.modified == 0 {
        // Match GNU diff: silent on identical files (advisory message to
        // stderr only). Exit code 0.
        eprintln!("[ok] Files are identical");  // #2446: exit 0 for identical
        timer.track(
            &format!("diff {} {}", file1.display(), file2.display()),
            "rtco diff",
            &raw,
            "",
        );
        return Ok(0);
    }

    let mut rtco = String::new();
    rtco.push_str(&format!("{} → {}\n", file1.display(), file2.display()));
    rtco.push_str(&format!(
        "   +{} added, -{} removed, ~{} modified\n\n",
        diff.added, diff.removed, diff.modified
    ));
    rtco.push_str(&format_diff_changes(&diff));

    print!("{}", rtco);
    timer.track(
        &format!("diff {} {}", file1.display(), file2.display()),
        "rtco diff",
        &raw,
        &rtco,
    );
    // GNU diff convention: exit 1 when files differ (#2446).
    Ok(1)
}

/// Run diff from stdin (piped command output)
pub fn run_stdin(_verbose: u8) -> Result<()> {
    use std::io::{self, Read};
    let timer = tracking::TimedExecution::start();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Parse unified diff format
    let condensed = condense_unified_diff(&input);
    println!("{}", condensed);

    timer.track("diff (stdin)", "rtco diff (stdin)", &input, &condensed);

    Ok(())
}

#[derive(Debug)]
enum DiffChange {
    Added(usize, String),
    Removed(usize, String),
    Modified(usize, String, String),
}

struct DiffResult {
    added: usize,
    removed: usize,
    modified: usize,
    changes: Vec<DiffChange>,
}

fn format_diff_changes(diff: &DiffResult) -> String {
    let mut out = String::new();
    for change in &diff.changes {
        match change {
            DiffChange::Added(ln, c) => out.push_str(&format!("+{:4} {}\n", ln, c)),
            DiffChange::Removed(ln, c) => out.push_str(&format!("-{:4} {}\n", ln, c)),
            DiffChange::Modified(ln, old, new) => {
                out.push_str(&format!("~{:4} {} → {}\n", ln, old, new))
            }
        }
    }
    out
}

fn compute_diff(lines1: &[&str], lines2: &[&str]) -> DiffResult {
    let mut changes = Vec::new();
    let mut added = 0;
    let mut removed = 0;
    let mut modified = 0;

    // Strip trailing empty line from split('\n') when input ends with newline (#2627)
    let lines1 = if lines1.last() == Some(&"") { &lines1[..lines1.len()-1] } else { lines1 };
    let lines2 = if lines2.last() == Some(&"") { &lines2[..lines2.len()-1] } else { lines2 };

    // Simple line-by-line comparison (not optimal but fast)
    let max_len = lines1.len().max(lines2.len());

    for i in 0..max_len {
        let l1 = lines1.get(i).copied();
        let l2 = lines2.get(i).copied();

        match (l1, l2) {
            (Some(a), Some(b)) if a != b => {
                // Check if it's similar (modification) or completely different
                if similarity(a, b) > 0.5 {
                    changes.push(DiffChange::Modified(i + 1, a.to_string(), b.to_string()));
                    modified += 1;
                } else {
                    changes.push(DiffChange::Removed(i + 1, a.to_string()));
                    changes.push(DiffChange::Added(i + 1, b.to_string()));
                    removed += 1;
                    added += 1;
                }
            }
            (Some(a), None) => {
                changes.push(DiffChange::Removed(i + 1, a.to_string()));
                removed += 1;
            }
            (None, Some(b)) => {
                changes.push(DiffChange::Added(i + 1, b.to_string()));
                added += 1;
            }
            _ => {}
        }
    }

    DiffResult {
        added,
        removed,
        modified,
        changes,
    }
}

fn similarity(a: &str, b: &str) -> f64 {
    let a_chars: std::collections::HashSet<char> = a.chars().collect();
    let b_chars: std::collections::HashSet<char> = b.chars().collect();

    let intersection = a_chars.intersection(&b_chars).count();
    let union = a_chars.union(&b_chars).count();

    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

fn condense_unified_diff(diff: &str) -> String {
    let mut result = Vec::new();
    let mut current_file = String::new();
    let mut added = 0;
    let mut removed = 0;
    let mut changes = Vec::new();

    // Never truncate diff content — users make decisions based on this data.
    // Only strip diff metadata (headers, @@ hunks); all +/- lines shown in full.
    for line in diff.lines() {
        if line.starts_with("diff --git") || line.starts_with("--- ") || line.starts_with("+++ ") {
            if line.starts_with("+++ ") {
                if !current_file.is_empty() && (added > 0 || removed > 0) {
                    result.push(format!("[file] {} (+{} -{})", current_file, added, removed));
                    for c in &changes {
                        result.push(format!("  {}", c));
                    }
                    let total = added + removed;
                    if total > 10 {
                        result.push(format!("  ... +{} more", total - 10));
                    }
                }
                current_file = line
                    .trim_start_matches("+++ ")
                    .trim_start_matches("b/")
                    .to_string();
                added = 0;
                removed = 0;
                changes.clear();
            }
        } else if line.starts_with('+') && !line.starts_with("+++") {
            added += 1;
            changes.push(line.to_string());
        } else if line.starts_with('-') && !line.starts_with("---") {
            removed += 1;
            changes.push(line.to_string());
        }
    }

    // Last file
    if !current_file.is_empty() && (added > 0 || removed > 0) {
        result.push(format!("[file] {} (+{} -{})", current_file, added, removed));
        for c in &changes {
            result.push(format!("  {}", c));
        }
        let total = added + removed;
        if total > 10 {
            result.push(format!("  ... +{} more", total - 10));
        }
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- similarity ---

    #[test]
    fn test_similarity_identical() {
        assert_eq!(similarity("hello", "hello"), 1.0);
    }

    #[test]
    fn test_similarity_completely_different() {
        assert_eq!(similarity("abc", "xyz"), 0.0);
    }

    #[test]
    fn test_similarity_empty_strings() {
        // Both empty: union is 0, returns 1.0 by convention
        assert_eq!(similarity("", ""), 1.0);
    }

    #[test]
    fn test_similarity_partial_overlap() {
        let s = similarity("abcd", "abef");
        // Shared: a, b. Union: a, b, c, d, e, f = 6. Jaccard = 2/6
        assert!((s - 2.0 / 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_similarity_threshold_for_modified() {
        // "let x = 1;" vs "let x = 2;" should be > 0.5 (treated as modification)
        assert!(similarity("let x = 1;", "let x = 2;") > 0.5);
    }

    // --- compute_diff ---

    #[test]
    fn test_compute_diff_identical() {
        let a = vec!["line1", "line2", "line3"];
        let b = vec!["line1", "line2", "line3"];
        let result = compute_diff(&a, &b);
        assert_eq!(result.added, 0);
        assert_eq!(result.removed, 0);
        assert_eq!(result.modified, 0);
        assert!(result.changes.is_empty());
    }

    #[test]
    fn test_compute_diff_added_lines() {
        let a = vec!["line1"];
        let b = vec!["line1", "line2", "line3"];
        let result = compute_diff(&a, &b);
        assert_eq!(result.added, 2);
        assert_eq!(result.removed, 0);
    }

    #[test]
    fn test_compute_diff_removed_lines() {
        let a = vec!["line1", "line2", "line3"];
        let b = vec!["line1"];
        let result = compute_diff(&a, &b);
        assert_eq!(result.removed, 2);
        assert_eq!(result.added, 0);
    }

    #[test]
    fn test_compute_diff_modified_line() {
        // Similar lines (>0.5 similarity) are classified as modified
        let a = vec!["let x = 1;"];
        let b = vec!["let x = 2;"];
        let result = compute_diff(&a, &b);
        assert_eq!(result.modified, 1);
        assert_eq!(result.added, 0);
        assert_eq!(result.removed, 0);
    }

    #[test]
    fn test_compute_diff_completely_different_line() {
        // Dissimilar lines (<= 0.5 similarity) are added+removed, not modified
        let a = vec!["aaaa"];
        let b = vec!["zzzz"];
        let result = compute_diff(&a, &b);
        assert_eq!(result.modified, 0);
        assert_eq!(result.added, 1);
        assert_eq!(result.removed, 1);
    }

    #[test]
    fn test_compute_diff_empty_inputs() {
        let result = compute_diff(&[], &[]);
        assert_eq!(result.added, 0);
        assert_eq!(result.removed, 0);
        assert!(result.changes.is_empty());
    }

    // --- condense_unified_diff ---

    #[test]
    fn test_condense_unified_diff_single_file() {
        let diff = r#"diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("hello");
     println!("world");
 }
"#;
        let result = condense_unified_diff(diff);
        assert!(result.contains("src/main.rs"));
        assert!(result.contains("+1"));
        assert!(result.contains("println"));
    }

    #[test]
    fn test_condense_unified_diff_multiple_files() {
        let diff = r#"diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
+added line
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
-removed line
"#;
        let result = condense_unified_diff(diff);
        assert!(result.contains("a.rs"));
        assert!(result.contains("b.rs"));
    }

    #[test]
    fn test_condense_unified_diff_empty() {
        let result = condense_unified_diff("");
        assert!(result.is_empty());
    }

    // --- truncation accuracy ---

    fn make_large_unified_diff(added: usize, removed: usize) -> String {
        let mut lines = vec![
            "diff --git a/config.yaml b/config.yaml".to_string(),
            "--- a/config.yaml".to_string(),
            "+++ b/config.yaml".to_string(),
            "@@ -1,200 +1,200 @@".to_string(),
        ];
        for i in 0..removed {
            lines.push(format!("-old_value_{}", i));
        }
        for i in 0..added {
            lines.push(format!("+new_value_{}", i));
        }
        lines.join("\n")
    }

    #[test]
    fn test_condense_unified_diff_overflow_count_accuracy() {
        // 100 added + 100 removed = 200 total changes, only 10 shown
        // True overflow = 200 - 10 = 190
        // Bug: changes vec capped at 15, so old code showed "+5 more" (15-10) instead of "+190 more"
        let diff = make_large_unified_diff(100, 100);
        let result = condense_unified_diff(&diff);
        assert!(
            result.contains("+190 more"),
            "Expected '+190 more' but got:\n{}",
            result
        );
        assert!(
            !result.contains("+5 more"),
            "Bug still present: showing '+5 more' instead of true overflow"
        );
    }

    #[test]
    fn test_condense_unified_diff_no_false_overflow() {
        // 8 changes total — all fit within the 10-line display cap, no overflow message
        let diff = make_large_unified_diff(4, 4);
        let result = condense_unified_diff(&diff);
        assert!(
            !result.contains("more"),
            "No overflow message expected for 8 changes, got:\n{}",
            result
        );
    }

    #[test]
    fn test_no_truncation_large_diff() {
        // Verify compute_diff returns all changes without truncation
        let mut a = Vec::new();
        let mut b = Vec::new();
        for i in 0..500 {
            a.push(format!("line_{}", i));
            if i % 3 == 0 {
                b.push(format!("CHANGED_{}", i));
            } else {
                b.push(format!("line_{}", i));
            }
        }
        let a_refs: Vec<&str> = a.iter().map(|s| s.as_str()).collect();
        let b_refs: Vec<&str> = b.iter().map(|s| s.as_str()).collect();
        let result = compute_diff(&a_refs, &b_refs);

        assert!(
            result.changes.len() > 100,
            "Expected 100+ changes, got {}",
            result.changes.len()
        );
        assert!(!result.changes.is_empty());
    }

    #[test]
    fn test_format_diff_shows_all_changes() {
        let mut a = Vec::new();
        let mut b = Vec::new();
        for i in 0..100 {
            a.push(format!("old_line_{}", i));
            b.push(format!("new_line_{}", i));
        }
        let a_refs: Vec<&str> = a.iter().map(|s| s.as_str()).collect();
        let b_refs: Vec<&str> = b.iter().map(|s| s.as_str()).collect();
        let diff = compute_diff(&a_refs, &b_refs);
        let output = format_diff_changes(&diff);

        assert!(output.contains("old_line_0"), "should contain first change");
        assert!(output.contains("new_line_99"), "should contain last change");
    }

    #[test]
    fn test_run_returns_0_on_identical_files() {
        use std::io::Write;
        let mut a = tempfile::NamedTempFile::new().unwrap();
        let mut b = tempfile::NamedTempFile::new().unwrap();
        a.write_all(b"hello\nworld\n").unwrap();
        b.write_all(b"hello\nworld\n").unwrap();
        a.flush().unwrap();
        b.flush().unwrap();

        let exit = run(a.path(), b.path(), 0).expect("run ok");
        assert_eq!(exit, 0, "identical files must exit 0 (GNU diff convention)");
    }

    #[test]
    fn test_run_returns_1_on_different_files() {
        use std::io::Write;
        let mut a = tempfile::NamedTempFile::new().unwrap();
        let mut b = tempfile::NamedTempFile::new().unwrap();
        a.write_all(b"hello\nworld\n").unwrap();
        b.write_all(b"hello\nWORLD\n").unwrap();
        a.flush().unwrap();
        b.flush().unwrap();

        let exit = run(a.path(), b.path(), 0).expect("run ok");
        assert_eq!(exit, 1, "differing files must exit 1 (GNU diff convention)");
    }

    #[test]
    fn test_run_returns_2_on_missing_file() {
        use std::path::PathBuf;
        let missing = PathBuf::from("/nonexistent-rtco-test-path-xyz123/a.txt");
        let existing = tempfile::NamedTempFile::new().unwrap();

        let exit = run(&missing, existing.path(), 0).expect("run does not error out on missing");
        assert_eq!(exit, 2, "missing file must exit 2 (GNU diff convention)");
    }

    #[test]
    fn test_long_lines_not_truncated() {
        let long_line = "x".repeat(500);
        let a = vec![long_line.as_str()];
        let b = vec!["short"];
        let result = compute_diff(&a, &b);
        match &result.changes[0] {
            DiffChange::Removed(_, content) | DiffChange::Added(_, content) => {
                assert_eq!(content.len(), 500, "Line was truncated!");
            }
            DiffChange::Modified(_, old, _) => {
                assert_eq!(old.len(), 500, "Line was truncated!");
            }
        }
    }
}
