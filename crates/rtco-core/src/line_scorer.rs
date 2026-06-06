//! Score-based line selection for intelligent output truncation.
//!
//! Uses [`KeywordDetector`] keyword classification as a base score, then
//! applies contextual boosts for stack trace frames and summary lines.
//! The resulting scores drive a greedy selection that always preserves
//! the first and last lines while filling remaining slots by importance.

use super::keyword_detector::KeywordDetector;
use super::text_stats::is_stack_trace;

/// Scored metadata for a single line of output.
#[derive(Debug, Clone)]
pub struct LineScore {
    /// Zero-based index of the line in the original input.
    pub line_index: usize,
    /// Composite importance score (0.0 .. 2.4).
    pub score: f64,
    /// Keyword-derived level from the detector.
    pub level: super::keyword_detector::LineLevel,
}

const STACK_TRACE_BOOST: f64 = 0.3;
const SUMMARY_BOOST: f64 = 0.4;

/// Score every line in `lines` using keyword detection plus contextual boosts.
///
/// Boosts applied on top of the base keyword score:
/// - **Stack trace frames**: `+0.3` (recognised by [`is_stack_trace`])
/// - **Summary lines** (level == `Summary`): `+0.4`
///
/// Returns a `Vec<LineScore>` with one entry per input line, preserving order.
pub fn score_lines(lines: &[&str]) -> Vec<LineScore> {
    let det = KeywordDetector::new();

    lines
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            let level = det.classify_line(line);
            let mut score = level.score();

            if is_stack_trace(line) {
                score += STACK_TRACE_BOOST;
            }
            if matches!(level, super::keyword_detector::LineLevel::Summary) {
                score += SUMMARY_BOOST;
            }

            LineScore {
                line_index: idx,
                score,
                level,
            }
        })
        .collect()
}

/// Select which line indices to keep, given a budget of `max_lines`.
///
/// Always includes the first (index 0) and last (index `n-1`) lines when
/// the input is non-empty. Remaining slots are filled by descending score
/// (ties broken by original order — earlier lines win).
///
/// Returns a sorted `Vec<usize>` of kept indices.
pub fn select_by_score(scored: &[LineScore], max_lines: usize) -> Vec<usize> {
    let n = scored.len();
    if n == 0 || max_lines == 0 {
        return Vec::new();
    }

    if max_lines >= n {
        return (0..n).collect();
    }

    let mut keep: Vec<usize> = Vec::with_capacity(max_lines);

    // Always keep first and last.
    keep.push(0);
    if n > 1 {
        keep.push(n - 1);
    }

    // Build a sorted index list by descending score, breaking ties by
    // original line order (lower index first).
    let mut by_score: Vec<usize> = (0..n).collect();
    by_score.sort_by(|&a, &b| {
        scored[b]
            .score
            .partial_cmp(&scored[a].score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(&b))
    });

    for &idx in &by_score {
        if keep.len() >= max_lines {
            break;
        }
        if !keep.contains(&idx) {
            keep.push(idx);
        }
    }

    keep.sort_unstable();
    keep
}

/// Format a human-readable omission marker summarising what was dropped.
///
/// `omitted` — total number of lines removed.
/// `errors`  — how many omitted lines had an error/security level.
/// `warns`   — how many omitted lines had a warning level.
///
/// # Example output
/// ```text
/// --- 42 lines omitted (3 errors, 2 warnings) ---
/// ```
pub fn format_omission_marker(omitted: usize, errors: usize, warns: usize) -> String {
    if omitted == 0 {
        return String::new();
    }

    let mut parts = Vec::new();
    if errors > 0 {
        parts.push(format!(
            "{} error{}",
            errors,
            if errors == 1 { "" } else { "s" }
        ));
    }
    if warns > 0 {
        parts.push(format!(
            "{} warning{}",
            warns,
            if warns == 1 { "" } else { "s" }
        ));
    }

    if parts.is_empty() {
        format!("--- {} lines omitted ---", omitted)
    } else {
        format!("--- {} lines omitted ({}) ---", omitted, parts.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── score_lines ──────────────────────────────────────────────────

    #[test]
    fn score_empty_input() {
        let scored = score_lines(&[]);
        assert!(scored.is_empty());
    }

    #[test]
    fn score_error_line_high() {
        let scored = score_lines(&["ERROR: connection refused"]);
        assert_eq!(scored.len(), 1);
        assert!(scored[0].score >= 1.0);
    }

    #[test]
    fn score_plain_line_low() {
        let scored = score_lines(&["Building project..."]);
        assert_eq!(scored.len(), 1);
        assert_eq!(scored[0].score, 0.0);
    }

    #[test]
    fn score_stack_trace_boost() {
        let plain = score_lines(&["just some text"]);
        let stack = score_lines(&["   at com.example.Main(Main.java:42)"]);

        // Stack trace detector recognises Java frames.
        assert!(
            stack[0].score > plain[0].score,
            "stack trace should score higher than plain text"
        );
        assert!(
            (stack[0].score - plain[0].score - STACK_TRACE_BOOST).abs() < f64::EPSILON
                || stack[0].score > STACK_TRACE_BOOST,
            "boost should be applied"
        );
    }

    #[test]
    fn score_summary_boost() {
        let scored = score_lines(&["3 passed, 1 skipped"]);
        // Summary level base is 0.4, plus 0.4 boost = 0.8.
        assert!(
            (scored[0].score - 0.8).abs() < f64::EPSILON,
            "expected 0.8, got {}",
            scored[0].score
        );
    }

    #[test]
    fn score_preserves_order() {
        let lines = vec!["plain", "ERROR: boom", "warning: x", "plain again"];
        let scored = score_lines(&lines);
        assert_eq!(scored[0].line_index, 0);
        assert_eq!(scored[1].line_index, 1);
        assert_eq!(scored[2].line_index, 2);
        assert_eq!(scored[3].line_index, 3);
    }

    #[test]
    fn score_ordering_by_severity() {
        let lines = vec![
            "just text",
            "WARNING: something",
            "FATAL: crash",
            "hello world",
        ];
        let scored = score_lines(&lines);
        assert!(scored[2].score > scored[1].score, "fatal > warning");
        assert!(scored[1].score > scored[0].score, "warning > plain");
    }

    // ── select_by_score ──────────────────────────────────────────────

    #[test]
    fn select_empty() {
        let sel = select_by_score(&[], 10);
        assert!(sel.is_empty());
    }

    #[test]
    fn select_zero_budget() {
        let scored = score_lines(&["a", "b", "c"]);
        let sel = select_by_score(&scored, 0);
        assert!(sel.is_empty());
    }

    #[test]
    fn select_budget_exceeds_length() {
        let scored = score_lines(&["a", "b", "c"]);
        let sel = select_by_score(&scored, 100);
        assert_eq!(sel, vec![0, 1, 2]);
    }

    #[test]
    fn select_always_keeps_first_and_last() {
        let lines: Vec<&str> = (0..100).map(|_| "plain line").collect();
        let scored = score_lines(&lines);
        let sel = select_by_score(&scored, 5);

        assert!(sel.contains(&0), "must keep first line");
        assert!(sel.contains(&99), "must keep last line");
    }

    #[test]
    fn select_fills_by_score() {
        let lines = vec![
            "plain",           // idx 0 — kept (first)
            "plain",           // idx 1
            "ERROR: critical", // idx 2 — high score, should be picked
            "plain",           // idx 3
            "plain",           // idx 4 — kept (last)
        ];
        let scored = score_lines(&lines);
        let sel = select_by_score(&scored, 3);

        assert_eq!(sel, vec![0, 2, 4]);
    }

    #[test]
    fn select_result_is_sorted() {
        let lines = vec!["a", "ERROR: x", "b", "c", "FATAL: y", "d"];
        let scored = score_lines(&lines);
        let sel = select_by_score(&scored, 4);

        let mut sorted = sel.clone();
        sorted.sort_unstable();
        assert_eq!(sel, sorted, "result must be sorted");
    }

    #[test]
    fn select_no_duplicates() {
        let lines = vec!["ERROR: boom", "FATAL: crash", "plain"];
        let scored = score_lines(&lines);
        let sel = select_by_score(&scored, 2);

        let mut deduped = sel.clone();
        deduped.dedup();
        assert_eq!(sel.len(), deduped.len(), "no duplicate indices");
    }

    #[test]
    fn select_single_line() {
        let scored = score_lines(&["only one"]);
        let sel = select_by_score(&scored, 1);
        assert_eq!(sel, vec![0]);
    }

    #[test]
    fn select_two_lines() {
        let scored = score_lines(&["first", "last"]);
        let sel = select_by_score(&scored, 2);
        assert_eq!(sel, vec![0, 1]);
    }

    // ── format_omission_marker ───────────────────────────────────────

    #[test]
    fn omission_marker_zero_omitted() {
        assert_eq!(format_omission_marker(0, 0, 0), "");
    }

    #[test]
    fn omission_marker_plain() {
        assert_eq!(format_omission_marker(10, 0, 0), "--- 10 lines omitted ---");
    }

    #[test]
    fn omission_marker_with_errors() {
        assert_eq!(
            format_omission_marker(10, 3, 0),
            "--- 10 lines omitted (3 errors) ---"
        );
    }

    #[test]
    fn omission_marker_with_warnings() {
        assert_eq!(
            format_omission_marker(10, 0, 2),
            "--- 10 lines omitted (2 warnings) ---"
        );
    }

    #[test]
    fn omission_marker_with_both() {
        assert_eq!(
            format_omission_marker(42, 3, 2),
            "--- 42 lines omitted (3 errors, 2 warnings) ---"
        );
    }

    #[test]
    fn omission_marker_singular() {
        assert_eq!(
            format_omission_marker(5, 1, 1),
            "--- 5 lines omitted (1 error, 1 warning) ---"
        );
    }

    // ── integration: score + select + format ─────────────────────────

    #[test]
    fn integration_full_pipeline() {
        let lines = vec![
            "Starting build...",
            "Compiling crate v0.1.0",
            "   at src/main.rs:10:5",
            "warning: unused variable `x`",
            "ERROR: type mismatch at line 42",
            "Build summary: 1 error, 1 warning",
            "Build failed with exit code 1",
        ];

        let scored = score_lines(&lines);
        let sel = select_by_score(&scored, 4);
        assert_eq!(sel.len(), 4);

        // First and last must be kept.
        assert!(sel.contains(&0));
        assert!(sel.contains(&6));

        // Error and summary lines should be prioritised.
        assert!(sel.contains(&4), "error line kept");
        assert!(sel.contains(&5), "summary line kept");

        // Omission marker for the rest.
        let omitted = lines.len() - sel.len();
        assert!(omitted > 0);
        let marker = format_omission_marker(omitted, 0, 0);
        assert!(marker.contains("lines omitted"));
    }
}
