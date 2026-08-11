//! `rtco hashline` — proxy for hashline (hash-anchored file editing).
//!
//! hashline reads a file with a `[path#HASH]` header + `N:hh|content` lines,
//! and applies anchor-based patches. Its output is already maximally compact
//! (no ANSI, no verbose metadata), so rtco routes it through the shared runner
//! for tracking + tee-recovery + never_worse, with only a light filter on the
//! `read` view.
//!
//! Mutation commands (`patch`, `write`, `remove`, `rename`) are passthrough:
//! filtering must never mask an anchor/apply error (same rule as `git commit`).

use rtco_core::runner::{self, RunOptions};
use rtco_core::utils::resolved_command;
use anyhow::Result;
use std::ffi::OsString;

/// Strip the `[path#HASH]` header line from hashline read/write output.
/// The `N:hh|content` lines are the anchor format agents consume — keep them.
/// Preserves the trailing newline so passthrough output is byte-stable.
fn filter_read(output: &str) -> String {
    let had_trailing_nl = output.ends_with('\n');
    let mut body: String = output
        .lines()
        .filter(|line| !(line.starts_with('[') && line.contains('#') && line.ends_with(']')))
        .collect::<Vec<_>>()
        .join("\n");
    if had_trailing_nl {
        body.push('\n');
    }
    body
}

/// Entry point for `rtco hashline <args>`.
pub fn run(args: &[String], verbose: u8) -> Result<i32> {
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");
    match subcommand {
        // `read` is the hot path and benefits from dropping the redundant
        // `[path#HASH]` header (the anchors on each line remain).
        "read" => run_read(args, verbose),
        // Destructive / structured commands — passthrough, tracked.
        "patch" | "write" | "remove" | "rename" | "find-block" | "guide" | "serve" | "mcp" => {
            let os_args: Vec<OsString> = args.iter().map(OsString::from).collect();
            runner::run_passthrough("hashline", &os_args, verbose)
        }
        // Unknown subcommand: passthrough so hashline's own error is surfaced.
        _ => {
            let os_args: Vec<OsString> = args.iter().map(OsString::from).collect();
            runner::run_passthrough("hashline", &os_args, verbose)
        }
    }
}

fn run_read(args: &[String], verbose: u8) -> Result<i32> {
    let mut cmd = resolved_command("hashline");
    for arg in args {
        cmd.arg(arg);
    }
    let args_display = args.join(" ");
    if verbose > 0 {
        eprintln!("Running: hashline {}", args_display);
    }
    runner::run_filtered(
        cmd,
        "hashline",
        &args_display,
        filter_read,
        RunOptions::with_tee("hashline_read"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_tokens(s: &str) -> usize {
        s.split_whitespace().count()
    }

    #[test]
    fn filter_read_strips_header_keeps_anchor_lines() {
        let input = "[crates/rtco-cli/src/cmds/scala/sbt_cmd.rs#33f5]\n1:d8|use rtco_core::runner;\n2:52|use rtco_core::utils;\n";
        let out = filter_read(input);
        assert!(!out.contains("[crates/rtco-cli"));
        assert!(out.contains("1:d8|use rtco_core::runner;"));
        assert!(out.contains("2:52|use rtco_core::utils;"));
    }

    #[test]
    fn filter_read_real_fixture_strips_header() {
        let input = include_str!("../../../../../tests/fixtures/hashline/read_raw.txt");
        let out = filter_read(input);
        assert!(!out.lines().any(|l| l.starts_with('[') && l.contains('#')), "header still present");
        assert!(out.contains("1:d8|use rtco_core::runner"));
        assert!(out.contains("10:57|static TEST_SUMMARY_RE"));
    }

    #[test]
    fn filter_read_savings_non_negative() {
        // The anchor format is the value — filter must never inflate tokens.
        let input = "[f#abc]\n1:11|let x = 1;\n2:22|let y = 2;\n";
        let out = filter_read(input);
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
    fn filter_read_savings_real_fixture_non_negative() {
        let input = include_str!("../../../../../tests/fixtures/hashline/read_raw.txt");
        let out = filter_read(input);
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
    fn filter_read_empty_input() {
        assert_eq!(filter_read(""), "");
    }

    #[test]
    fn filter_read_no_header_passthrough() {
        let input = "1:11|plain line\n2:22|another\n";
        assert_eq!(filter_read(input), input);
    }
}
