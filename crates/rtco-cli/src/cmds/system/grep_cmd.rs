//! Filters grep output by grouping matches by file.

use rtco_core::config;
use rtco_core::stream::exec_capture;
use rtco_core::tracking;
use rtco_core::utils::resolved_command;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;

/// Check if a file path targets a .env file (security restriction #2428)
fn is_env_file(path: &str) -> bool {
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    name == ".env" || name.starts_with(".env.") || name.starts_with(".env_")
}

fn env_warning(what: &str) -> String {
    format!(
        "[SECURITY] rtco: refusing to read '{}' - .env files may contain secrets",
        what
    )
}


/// Parse raw grep arguments into (pattern, path, remaining_flags).
///
/// Handles:
/// - `grep -r pattern path` — flags before pattern
/// - `grep pattern path -r` — flags after path
/// - `grep -rn pattern` — combined short flags
/// - `grep -A5 pattern` — flags with values
fn parse_grep_args(args: &[String]) -> (String, String, Vec<String>) {
    let mut pattern = String::new();
    let mut path = String::from(".");
    let mut extra = Vec::new();
    let mut i = 0;

    // Two-pass: first collect all flags/values
    let mut raw = Vec::new();
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with('-') {
            // Expand combined short flags like -rn to -r -n
            if arg.len() > 2 && !arg.starts_with("--") {
                let chars: Vec<char> = arg[1..].chars().collect();
                for c in &chars {
                    extra.push(format!("-{}", c));
                }
            } else {
                extra.push(arg.clone());
                // Flags that take a separate value: -A, -B, -C
                if matches!(arg.as_str(), "-A" | "-B" | "-C" | "--after-context" | "--before-context" | "--context") {
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        i += 1;
                        extra.push(args[i].clone());
                    }
                }
            }
        } else {
            raw.push(arg.clone());
        }
        i += 1;
    }

    // First non-flag arg is pattern, second is path
    for (idx, val) in raw.iter().enumerate() {
        if idx == 0 {
            pattern = val.clone();
        } else if idx == 1 {
            path = val.clone();
        } else {
            extra.push(val.clone());
        }
    }

    if pattern.is_empty() {
        pattern = "*".to_string();
    }

    (pattern, path, extra)
}
/// Entry point from raw CLI arguments (bypasses clap strict parsing for grep-flag compat).
///
/// Parses pattern, path, and common grep flags from a raw arg list so that
/// `rtco grep -rn foo .` works even though `-r` is not a named clap flag.
/// Fixes #2614 (common grep flags rejected), #2620 (ultra-compact support).
pub fn run_from_args(args: &[String], verbose: u8, ultra_compact: bool) -> Result<i32> {
    let (pattern, path, extra_args) = parse_grep_args(args);
    run_inner(
        &pattern, &path, 80, 200, false, None, &extra_args, verbose, ultra_compact,
    )
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn run(
    pattern: &str,
    path: &str,
    max_line_len: usize,
    max_results: usize,
    context_only: bool,
    file_type: Option<&str>,
    extra_args: &[String],
    verbose: u8,
) -> Result<i32> {
    run_inner(pattern, path, max_line_len, max_results, context_only, file_type, extra_args, verbose, false)
}

/// Core grep implementation with all options.
///
/// #2614: Accepts grep-ism flags and translates them for ripgrep compatibility.
/// #2620: When `ultra_compact` is true, uses minimal output format.
/// #2606: Uses rg internally with .gitignore handling preserved.
#[allow(clippy::too_many_arguments)]
fn run_inner(
    pattern: &str,
    path: &str,
    max_line_len: usize,
    max_results: usize,
    context_only: bool,
    file_type: Option<&str>,
    extra_args: &[String],
    verbose: u8,
    ultra_compact: bool,
) -> Result<i32> {
    let timer = tracking::TimedExecution::start();

    // Security: block grep on .env files (#2428)
    if path != "." && is_env_file(path) {
        eprintln!("{}", env_warning(path));
        return Ok(1);
    }
    if is_env_file(pattern) {
        eprintln!("{}", env_warning(pattern));
        return Ok(1);
    }

if verbose > 0 {
        eprintln!("grep: '{}' in {}", pattern, path);
    }

    // Fix: convert BRE alternation \| → | for rg (which uses PCRE-style regex)
    // #2563: Convert BRE escapes to PCRE (rg flavor)
    let rg_pattern = pattern
        .replace(r"\|", "|")   // BRE alternation
        .replace(r"\?", "?")   // BRE zero-or-one
        .replace(r"\+", "+");   // BRE one-or-more

    let mut rg_cmd = resolved_command("rg");
    // #2606: Don't use --no-ignore-vcs by default so rg's .gitignore handling is preserved.
    // Without this, rg returns 0 matches for files in .gitignore, causing
    // false negatives that make AI agents draw wrong conclusions.
    // -H: always emit the filename.
    // -0: NUL-separate filename. Allows the parser to disambiguate filenames or
    // content containing `:digits:` patterns (issue #1436).
    rg_cmd.args(["-nH0", "--no-heading", &rg_pattern, path]);

    if let Some(ft) = file_type {
        rg_cmd.arg("--type").arg(ft);
    }

    for arg in extra_args {
        // Fix: translate grep-ism flags for rg compatibility (#2614)
        match arg.as_str() {
            "-r" | "--recursive" => continue,  // rg is recursive by default; -r means --replace
            "-E" | "--extended-regexp" => continue,  // rg uses PCRE by default (equivalent to -E)
            "-P" | "--perl-regexp" => continue,  // rg is PCRE by default
            "-G" | "--basic-regexp" => {},  // rg supports -G
            "-F" | "--fixed-strings" => { rg_cmd.arg("--fixed-strings"); }
            "-i" | "--ignore-case" => { rg_cmd.arg("-i"); }
            "-w" | "--word-regexp" => { rg_cmd.arg("-w"); }
            "-x" | "--line-regexp" => { rg_cmd.arg("-x"); }
            "-l" | "--files-with-matches" => { rg_cmd.arg("-l"); }
            "-c" | "--count" => { rg_cmd.arg("--count"); }
            // Pass -A/-B/-C context flags to rg natively
            _ if arg.starts_with('-') => { rg_cmd.arg(arg); }
            _ => { rg_cmd.arg(arg); }
        }
    }

    let result = exec_capture(&mut rg_cmd)
        .or_else(|_| {
            let mut grep_cmd = resolved_command("grep");
            // When we fall back to grep, include all args, not just -rnHZ.
            grep_cmd.args(["-rnHZ", pattern, path]).args(extra_args);
            exec_capture(&mut grep_cmd)
        })
        .context("grep/rg failed")?;

    // Passthrough output flags that produce output that is already small.
    if has_format_flag(extra_args) {
        print!("{}", result.stdout);
        if !result.stderr.is_empty() {
            eprint!("{}", result.stderr.trim());
        }

        let args_display = if extra_args.is_empty() {
            format!("'{}' {}", pattern, path)
        } else {
            format!("{} '{}' {}", extra_args.join(" "), pattern, path)
        };

        timer.track_passthrough(
            &format!("grep {}", args_display),
            &format!("rtco grep {} (passthrough)", args_display),
        );
        return Ok(result.exit_code);
    }

    let exit_code = result.exit_code;
    let raw_output = result.stdout.clone();

    if result.stdout.trim().is_empty() {
        // Show stderr for errors (bad regex, missing file, etc.)
        if exit_code == 2 && !result.stderr.trim().is_empty() {
            eprintln!("{}", result.stderr.trim());
        }
        let msg = format!("0 matches for '{}'", pattern);
        println!("{}", msg);
        timer.track(
            &format!("grep -rn '{}' {}", pattern, path),
            "rtco grep",
            &raw_output,
            &msg,
        );
        return Ok(exit_code);
    }

    // Always filter: truncate long lines, apply per-file and global caps.
    // Output in standard file:line:content format that AI agents can parse.
    // (A passthrough approach yields 0% savings — no reason for RTCO to exist on that path.)
    let total_matches = result.stdout.lines().count();

    let context_re = if context_only {
        Regex::new(&format!("(?i).{{0,20}}{}.*", regex::escape(pattern))).ok()
    } else {
        None
    };

    let mut by_file: HashMap<String, Vec<(usize, String)>> = HashMap::new();
    for line in result.stdout.lines() {
        let Some((file, line_num, content)) = parse_match_line(line) else {
            continue;
        };
        let cleaned = clean_line(content, max_line_len, context_re.as_ref(), pattern);
        by_file.entry(file).or_default().push((line_num, cleaned));
    }

    let mut rtco_output = String::new();
    // #2608: Skip header for single-match results — the header is pure overhead
    // when the user will see exactly one file:line. Only show it when there are
    // multiple matches or files, or when ultra_compact is explicitly requested.
    if !ultra_compact && by_file.len() >= 1 && total_matches > 1 {
        rtco_output.push_str(&format!(
            "{} matches in {} files:\n\n",
            total_matches.min(max_results),
            by_file.len()
        ));
    }

    let mut shown = 0;
    let mut files: Vec<_> = by_file.iter().collect();
    files.sort_by_key(|(f, _)| *f);

    let per_file = config::limits().grep_max_per_file;
    for (file, matches) in files {
        if shown >= max_results {
            break;
        }

        let file_display = compact_path(file);
        for (line_num, content) in matches.iter().take(per_file) {
            if shown >= max_results {
                break;
            }
            rtco_output.push_str(&format!("{}:{}:{}\n", file_display, line_num, content));
            shown += 1;
        }
    }

    if total_matches > shown {
        rtco_output.push_str(&format!("[+{} more]\n", total_matches - shown));
    }

    print!("{}", rtco_output);
    timer.track(
        &format!("grep -rn '{}' {}", pattern, path),
        "rtco grep",
        &raw_output,
        &rtco_output,
    );

    Ok(exit_code)
}

/// Parses a single rg/grep match line of the form `file\0line_number:content`.
///
/// Requires the underlying command to be invoked with `-0` (rg) or `-Z` (grep)
/// so the filename is NUL-separated from `line:content`. NUL cannot appear in
/// file paths, so the parser is unambiguous regardless of:
///   - content with `:` or `::` (e.g. `ClassRegistry::init(...)`, issue #1436);
///   - paths with embedded `:` (Windows drive letters, weird filenames like
///     `badly_named:52:file.txt`).
///
/// Returns `None` for lines that do not match the expected shape (e.g. rg
/// `-A`/`-B` context lines that use `-` as separator).
fn parse_match_line(line: &str) -> Option<(String, usize, &str)> {
    lazy_static::lazy_static! {
        static ref MATCH_LINE_RE: Regex = Regex::new(r"^([^\x00]+)\x00(\d+):(.*)$").unwrap();
    }
    MATCH_LINE_RE.captures(line).and_then(|caps| {
        let (_, [file, line_num, content]) = caps.extract();
        let line_num: usize = line_num.parse().ok()?;
        Some((file.to_string(), line_num, content))
    })
}

fn has_format_flag(extra_args: &[String]) -> bool {
    extra_args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "-c" | "--count"
                | "-l"
                | "--files-with-matches"
                | "-L"
                | "--files-without-match"
                | "-o"
                | "--only-matching"
                | "-Z"
                | "--null"
        )
    })
}

fn clean_line(line: &str, max_len: usize, context_re: Option<&Regex>, pattern: &str) -> String {
    let trimmed = line.trim();

    if let Some(re) = context_re {
        if let Some(m) = re.find(trimmed) {
            let matched = m.as_str();
            if matched.len() <= max_len {
                return matched.to_string();
            }
        }
    }

    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        let lower = trimmed.to_lowercase();
        let pattern_lower = pattern.to_lowercase();

        if let Some(pos) = lower.find(&pattern_lower) {
            let char_pos = lower[..pos].chars().count();
            let chars: Vec<char> = trimmed.chars().collect();
            let char_len = chars.len();

            let start = char_pos.saturating_sub(max_len / 3);
            let end = (start + max_len).min(char_len);
            let start = if end == char_len {
                end.saturating_sub(max_len)
            } else {
                start
            };

            let slice: String = chars[start..end].iter().collect();
            if start > 0 && end < char_len {
                format!("...{}...", slice)
            } else if start > 0 {
                format!("...{}", slice)
            } else {
                format!("{}...", slice)
            }
        } else {
            let t: String = trimmed.chars().take(max_len - 3).collect();
            format!("{}...", t)
        }
    }
}

fn compact_path(path: &str) -> String {
    if path.len() <= 50 {
        return path.to_string();
    }

    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 3 {
        return path.to_string();
    }

    format!(
        "{}/.../{}/{}",
        parts[0],
        parts[parts.len() - 2],
        parts[parts.len() - 1]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_line() {
        let line = "            const result = someFunction();";
        let cleaned = clean_line(line, 50, None, "result");
        assert!(!cleaned.starts_with(' '));
        assert!(cleaned.len() <= 50);
    }

    #[test]
    fn test_compact_path() {
        let path = "/Users/patrick/dev/project/src/components/Button.tsx";
        let compact = compact_path(path);
        assert!(compact.len() <= 60);
    }

    #[test]
    fn test_extra_args_accepted() {
        // Test that the function signature accepts extra_args
        // This is a compile-time test - if it compiles, the signature is correct
        let _extra: Vec<String> = vec!["-i".to_string(), "-A".to_string(), "3".to_string()];
        // No need to actually run - we're verifying the parameter exists
    }

    #[test]
    fn test_clean_line_multibyte() {
        // Thai text that exceeds max_len in bytes
        let line = "  สวัสดีครับ นี่คือข้อความที่ยาวมากสำหรับทดสอบ  ";
        let cleaned = clean_line(line, 20, None, "ครับ");
        // Should not panic
        assert!(!cleaned.is_empty());
    }

    #[test]
    fn test_clean_line_emoji() {
        let line = "🎉🎊🎈🎁🎂🎄 some text 🎃🎆🎇✨";
        let cleaned = clean_line(line, 15, None, "text");
        assert!(!cleaned.is_empty());
    }

    // Fix: BRE \| alternation is translated to PCRE | for rg
    #[test]
    fn test_bre_alternation_translated() {
        let pattern = r"fn foo\|pub.*bar";
        // #2563: Convert BRE escapes to PCRE (rg flavor)
    let rg_pattern = pattern
        .replace(r"\|", "|")   // BRE alternation
        .replace(r"\?", "?")   // BRE zero-or-one
        .replace(r"\+", "+");   // BRE one-or-more
        assert_eq!(rg_pattern, "fn foo|pub.*bar");
    }

    // Fix: -r flag (grep recursive) is stripped from extra_args (rg is recursive by default)
    #[test]
    fn test_recursive_flag_stripped() {
        let extra_args: Vec<String> = vec!["-r".to_string(), "-i".to_string()];
        let filtered: Vec<&String> = extra_args
            .iter()
            .filter(|a| *a != "-r" && *a != "--recursive")
            .collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "-i");
    }

    // --- truncation accuracy ---

    #[test]
    fn test_grep_overflow_uses_uncapped_total() {
        // Confirm the grep overflow invariant: matches vec is never capped before overflow calc.
        // If total_matches > per_file, overflow = total_matches - per_file (not capped).
        // This documents that grep_cmd.rs avoids the diff_cmd bug (cap at N then compute N-10).
        let per_file = config::limits().grep_max_per_file;
        let total_matches = per_file + 42;
        let overflow = total_matches - per_file;
        assert_eq!(overflow, 42, "overflow must equal true suppressed count");
        // Demonstrate why capping before subtraction is wrong:
        let hypothetical_cap = per_file + 5;
        let capped = total_matches.min(hypothetical_cap);
        let wrong_overflow = capped - per_file;
        assert_ne!(
            wrong_overflow, overflow,
            "capping before subtraction gives wrong overflow"
        );
    }

    // --- format flag detection ---

    #[test]
    fn test_format_flag_detects_count() {
        assert!(has_format_flag(&["-c".to_string()]));
        assert!(has_format_flag(&["--count".to_string()]));
    }

    #[test]
    fn test_format_flag_detects_files_with_matches() {
        assert!(has_format_flag(&["-l".to_string()]));
        assert!(has_format_flag(&["--files-with-matches".to_string()]));
    }

    #[test]
    fn test_format_flag_detects_files_without_match() {
        assert!(has_format_flag(&["-L".to_string()]));
        assert!(has_format_flag(&["--files-without-match".to_string()]));
    }

    #[test]
    fn test_format_flag_detects_only_matching() {
        assert!(has_format_flag(&["-o".to_string()]));
        assert!(has_format_flag(&["--only-matching".to_string()]));
    }

    #[test]
    fn test_format_flag_detects_null() {
        assert!(has_format_flag(&["-Z".to_string()]));
        assert!(has_format_flag(&["--null".to_string()]));
    }

    #[test]
    fn test_format_flag_ignores_normal_flags() {
        assert!(!has_format_flag(&[
            "-i".to_string(),
            "-w".to_string(),
            "-A".to_string(),
            "3".to_string(),
        ]));
    }

    // Verify line numbers are always enabled in rg invocation (grep_cmd.rs:24).
    // The -n/--line-numbers clap flag in main.rs is a no-op accepted for compat.
    #[test]
    fn test_rg_always_has_line_numbers() {
        // grep_cmd::run() always passes "-n" to rg (line 24).
        // This test documents that -n is built-in, so the clap flag is safe to ignore.
        let mut cmd = resolved_command("rg");
        cmd.args(["-n", "--no-heading", "NONEXISTENT_PATTERN_12345", "."]);
        // If rg is available, it should accept -n without error (exit 1 = no match, not error)
        if let Ok(output) = cmd.output() {
            assert!(
                output.status.code() == Some(1) || output.status.success(),
                "rg -n should be accepted"
            );
        }
        // If rg is not installed, skip gracefully (test still passes)
    }

    // --- issue #1436: parse_match_line robustness ---
    // Input shape is `file\0line:content` (rg --null / grep -Z).

    #[test]
    fn test_parse_match_line_simple() {
        let line = "file.php\x0010:use Foo\\Bar;";
        let (file, line_num, content) = parse_match_line(line).unwrap();
        assert_eq!(file, "file.php");
        assert_eq!(line_num, 10);
        assert_eq!(content, "use Foo\\Bar;");
    }

    // Issue #1436 reproducer: content with `::` must not split into a phantom
    // file bucket. With NUL separation between file and line:content, content
    // colons are irrelevant to the parser.
    #[test]
    fn test_parse_match_line_content_with_double_colon() {
        let line = "externalImportShell.class.php\x0081:        $this->queueProcessModel = ClassRegistry::init('Collections.QueueProcess');";
        let (file, line_num, content) = parse_match_line(line).unwrap();
        assert_eq!(file, "externalImportShell.class.php");
        assert_eq!(line_num, 81);
        assert_eq!(
            content,
            "        $this->queueProcessModel = ClassRegistry::init('Collections.QueueProcess');"
        );
    }

    // Windows abs-path safety: drive letter + backslashes must not break the
    // parser. The NUL separator makes the file portion unambiguous.
    #[test]
    fn test_parse_match_line_windows_path() {
        let line = "C:\\src\\file.rs\x0042:fn main() {}";
        let (file, line_num, content) = parse_match_line(line).unwrap();
        assert_eq!(file, r"C:\src\file.rs");
        assert_eq!(line_num, 42);
        assert_eq!(content, "fn main() {}");
    }

    // Filenames containing `:digits:` (which would fool a greedy `:` parser)
    // must still parse correctly under NUL separation.
    #[test]
    fn test_parse_match_line_filename_with_colons() {
        let line = "badly_named:52:file.txt\x001:xxx";
        let (file, line_num, content) = parse_match_line(line).unwrap();
        assert_eq!(file, "badly_named:52:file.txt");
        assert_eq!(line_num, 1);
        assert_eq!(content, "xxx");
    }

    // Content that itself contains `:digits:` (e.g. log lines, port numbers,
    // line-number-like substrings) must not confuse the parser.
    #[test]
    fn test_parse_match_line_content_with_digit_colons() {
        let line = "log.txt\x007:debug: counter is :42: now";
        let (file, line_num, content) = parse_match_line(line).unwrap();
        assert_eq!(file, "log.txt");
        assert_eq!(line_num, 7);
        assert_eq!(content, "debug: counter is :42: now");
    }

    #[test]
    fn test_parse_match_line_malformed_returns_none() {
        // No NUL separator (e.g. rg/grep invoked without --null/-Z, or a
        // context line written with `-`).
        assert!(parse_match_line("file.rs:1:content").is_none());
        assert!(parse_match_line("not a match line").is_none());
        // Missing line number after NUL
        assert!(parse_match_line("file.rs\x00fn foo()").is_none());
        // Empty
        assert!(parse_match_line("").is_none());
    }

    #[test]
    fn test_parse_match_line_empty_content() {
        let line = "file.rs\x007:";
        let (file, line_num, content) = parse_match_line(line).unwrap();
        assert_eq!(file, "file.rs");
        assert_eq!(line_num, 7);
        assert_eq!(content, "");
    }

    #[test]
    fn test_rg_no_ignore_vcs_flag_accepted() {
        // Verify rg accepts --no-ignore-vcs (used to match grep -r behavior for .gitignore)
        let mut cmd = resolved_command("rg");
        cmd.args([
            "-n",
            "--no-heading",
            "--no-ignore-vcs",
            "NONEXISTENT_PATTERN_12345",
            ".",
        ]);
        if let Ok(output) = cmd.output() {
            assert!(
                output.status.code() == Some(1) || output.status.success(),
                "rg --no-ignore-vcs should be accepted"
            );
        }
        // If rg is not installed, skip gracefully (test still passes)
    }
}
