//! `rtco ffs` — proxy for ffs (fast file search).
//!
//! ffs output is already token-optimized (budget truncation, outlines,
//! `path:line:text` anchors). rtco routes it through the shared runner for
//! tracking + tee-recovery + never_worse. The grep-family commands get a
//! SearchCompressor pass to dedup/collapse repeated `file:line` hits; the rest
//! pass through unfiltered.

use rtco_core::compressors::search_compressor::SearchCompressor;
use rtco_core::runner::{self, RunOptions};
use rtco_core::utils::resolved_command;
use anyhow::Result;
use std::ffi::OsString;

/// Commands whose output is `file:line:text` style and benefits from
/// dedup/compaction via the shared SearchCompressor.
const GREP_FAMILY: &[&str] = &[
    "grep",
    "multi-grep",
    "multigrep",
    "callers",
    "callees",
    "refs",
    "flow",
    "siblings",
    "deps",
    "impact",
    "symbol",
];

fn is_grep_family(subcommand: &str) -> bool {
    GREP_FAMILY.contains(&subcommand)
}

/// Dedup/compact `file:line:text` output. Falls back to the raw input if the
/// compressor's never_worse invariant would be violated (in bytes OR tokens —
/// group-by-file headers can inflate a small match set even when bytes shrink).
fn filter_grep_output(output: &str) -> String {
    let compressor = SearchCompressor::new();
    match compressor.compress(output) {
        Ok(result)
            if result.text.len() <= output.len()
                && result.text.split_whitespace().count() <= output.split_whitespace().count() =>
        {
            result.text
        }
        _ => output.to_string(),
    }
}

/// Entry point for `rtco ffs <args>`.
pub fn run(args: &[String], verbose: u8) -> Result<i32> {
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");
    if is_grep_family(subcommand) {
        run_filtered(args, verbose)
    } else {
        // find/glob/read/outline/map/overview/index/mention/mcp/guide — compact already.
        let os_args: Vec<OsString> = args.iter().map(OsString::from).collect();
        runner::run_passthrough("ffs", &os_args, verbose)
    }
}

fn run_filtered(args: &[String], verbose: u8) -> Result<i32> {
    let mut cmd = resolved_command("ffs");
    for arg in args {
        cmd.arg(arg);
    }
    let args_display = args.join(" ");
    if verbose > 0 {
        eprintln!("Running: ffs {}", args_display);
    }
    runner::run_filtered(
        cmd,
        "ffs",
        &args_display,
        filter_grep_output,
        RunOptions::with_tee("ffs_grep"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_tokens(s: &str) -> usize {
        s.split_whitespace().count()
    }

    #[test]
    fn is_grep_family_classifies_subcommands() {
        assert!(is_grep_family("grep"));
        assert!(is_grep_family("multi-grep"));
        assert!(is_grep_family("symbol"));
        assert!(is_grep_family("callers"));
        assert!(!is_grep_family("find"));
        assert!(!is_grep_family("read"));
        assert!(!is_grep_family("map"));
        assert!(!is_grep_family(""));
    }

    #[test]
    fn filter_grep_output_dedups_repeated_paths() {
        let input = "crates/a.rs:10:fn main() {\ncrates/a.rs:11:    let x = 1;\ncrates/a.rs:12:}\ncrates/b.rs:1:mod a;\n";
        let out = filter_grep_output(input);
        // Compressed output groups by file — should be strictly smaller or equal.
        assert!(out.len() <= input.len(), "got: {}", out);
        assert!(out.contains("crates/a.rs") || out.contains("a.rs"));
        assert!(out.contains("crates/b.rs") || out.contains("b.rs"));
    }

    #[test]
    fn filter_grep_output_never_inflates() {
        let input = "a.rs:1:one\na.rs:2:two\n";
        let out = filter_grep_output(input);
        let in_tokens = count_tokens(input);
        let out_tokens = count_tokens(&out);
        assert!(
            out_tokens <= in_tokens,
            "never_worse: input {} tokens, output {} tokens",
            in_tokens,
            out_tokens
        );
    }

    #[test]
    fn filter_grep_output_real_fixture() {
        let input = include_str!("../../../../../tests/fixtures/ffs/grep_raw.txt");
        let out = filter_grep_output(input);
        // The never_worse guard returns the raw input when compression would
        // inflate it (small fixtures often group-by-file adds header bytes).
        // Either path is correct — what matters is tokens never increase and
        // the matched file is still present.
        let in_tokens = count_tokens(input);
        let out_tokens = count_tokens(&out);
        assert!(
            out_tokens <= in_tokens,
            "never_worse: input {} tokens, output {} tokens",
            in_tokens,
            out_tokens
        );
        assert!(out.contains("aws_cmd.rs") || out.contains("psql_cmd.rs"));
    }

    #[test]
    fn filter_grep_output_empty_input() {
        assert_eq!(filter_grep_output(""), "");
    }

    #[test]
    fn filter_grep_output_non_grep_input_passthrough() {
        // If the compressor can't parse it, it falls back to the raw input.
        let input = "not grep output\njust some lines\n";
        assert_eq!(filter_grep_output(input), input);
    }
}
