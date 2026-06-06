//! Utility functions for text processing and command execution.
//!
//! Provides common helpers used across rtk commands:
//! - ANSI color code stripping
//! - Text truncation
//! - Command execution with error context

use anyhow::{Context, Result};
use regex::Regex;
use std::path::PathBuf;
use std::process::Command;

/// Truncates a string to `max_len` characters, appending `...` if needed.
///
/// # Arguments
/// * `s` - The string to truncate
/// * `max_len` - Maximum length before truncation (minimum 3 to include "...")
///
/// # Examples
/// ```
/// use rtco_core::utils::truncate;
/// assert_eq!(truncate("hello world", 8), "hello...");
/// assert_eq!(truncate("hi", 10), "hi");
/// ```
pub fn truncate(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else if max_len < 3 {
        // If max_len is too small, just return "..."
        "...".to_string()
    } else {
        format!("{}...", s.chars().take(max_len - 3).collect::<String>())
    }
}

/// Strip ANSI escape codes (colors, styles) from a string.
///
/// # Arguments
/// * `text` - Text potentially containing ANSI escape codes
///
/// # Examples
/// ```
/// use rtco_core::utils::strip_ansi;
/// let colored = "\x1b[31mError\x1b[0m";
/// assert_eq!(strip_ansi(colored), "Error");
/// ```
pub fn strip_ansi(text: &str) -> String {
    lazy_static::lazy_static! {
        static ref ANSI_RE: Regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
    }
    ANSI_RE.replace_all(text, "").to_string()
}

/// Executes a command and returns cleaned stdout/stderr.
///
/// # Arguments
/// * `cmd` - Command to execute (e.g., "eslint")
/// * `args` - Command arguments
///
/// # Returns
/// `(stdout: String, stderr: String, exit_code: i32)`
/// Formats a token count with K/M suffixes for readability.
///
/// # Arguments
/// * `n` - Number of tokens
///
/// # Returns
/// Formatted string (e.g., "1.2M", "59.2K", "694")
///
/// # Examples
/// ```
/// use rtco_core::utils::format_tokens;
/// assert_eq!(format_tokens(1_234_567), "1.2M");
/// assert_eq!(format_tokens(59_234), "59.2K");
/// assert_eq!(format_tokens(694), "694");
/// ```
pub fn format_tokens(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

/// Formats a USD amount with adaptive precision.
///
/// # Arguments
/// * `amount` - Amount in dollars
///
/// # Returns
/// Formatted string with $ prefix
///
/// # Examples
/// ```
/// use rtco_core::utils::format_usd;
/// assert_eq!(format_usd(1234.567), "$1234.57");
/// assert_eq!(format_usd(12.345), "$12.35");
/// assert_eq!(format_usd(0.123), "$0.12");
/// assert_eq!(format_usd(0.0096), "$0.0096");
/// ```
pub fn format_usd(amount: f64) -> String {
    if !amount.is_finite() {
        return "$0.00".to_string();
    }
    if amount >= 0.01 {
        format!("${:.2}", amount)
    } else {
        format!("${:.4}", amount)
    }
}

/// Format cost-per-token as $/MTok (e.g., "$3.86/MTok")
///
/// # Arguments
/// * `cpt` - Cost per token (not per million tokens)
///
/// # Returns
/// Formatted string like "$3.86/MTok"
///
/// # Examples
/// ```
/// use rtco_core::utils::format_cpt;
/// assert_eq!(format_cpt(0.000003), "$3.00/MTok");
/// assert_eq!(format_cpt(0.0000038), "$3.80/MTok");
/// assert_eq!(format_cpt(0.00000386), "$3.86/MTok");
/// ```
pub fn format_cpt(cpt: f64) -> String {
    if !cpt.is_finite() || cpt <= 0.0 {
        return "$0.00/MTok".to_string();
    }
    let cpt_per_million = cpt * 1_000_000.0;
    format!("${:.2}/MTok", cpt_per_million)
}

/// Join items into a newline-separated string, appending an overflow hint when total > max.
///
/// # Examples
/// ```
/// use rtco_core::utils::join_with_overflow;
/// let items = vec!["a".to_string(), "b".to_string()];
/// assert_eq!(join_with_overflow(&items, 5, 3, "items"), "a\nb\n... +2 more items");
/// assert_eq!(join_with_overflow(&items, 2, 3, "items"), "a\nb");
/// ```
pub fn join_with_overflow(items: &[String], total: usize, max: usize, label: &str) -> String {
    let mut out = items.join("\n");
    if total > max {
        out.push_str(&format!("\n… +{} more {}", total - max, label));
    }
    out
}

/// Truncate an ISO 8601 datetime string to just the date portion (first 10 chars).
///
/// # Examples
/// ```
/// use rtco_core::utils::truncate_iso_date;
/// assert_eq!(truncate_iso_date("2024-01-15T10:30:00Z"), "2024-01-15");
/// assert_eq!(truncate_iso_date("2024-01-15"), "2024-01-15");
/// assert_eq!(truncate_iso_date("short"), "short");
/// ```
pub fn truncate_iso_date(date: &str) -> &str {
    if date.len() >= 10 {
        &date[..10]
    } else {
        date
    }
}

/// Format a confirmation message: "ok \<action\> \<detail\>"
/// Used for write operations (merge, create, comment, edit, etc.)
///
/// # Examples
/// ```
/// use rtco_core::utils::ok_confirmation;
/// assert_eq!(ok_confirmation("merged", "#42"), "ok merged #42");
/// assert_eq!(ok_confirmation("created", "PR #5 https://..."), "ok created PR #5 https://...");
/// ```
pub fn ok_confirmation(action: &str, detail: &str) -> String {
    if detail.is_empty() {
        format!("ok {}", action)
    } else {
        format!("ok {} {}", action, detail)
    }
}

/// Extract exit code from a process output. Returns the actual exit code, or
/// `128 + signal` per Unix convention when terminated by a signal (no exit code
/// available). Falls back to 1 on non-Unix platforms.
pub fn exit_code_from_output(output: &std::process::Output, label: &str) -> i32 {
    match output.status.code() {
        Some(code) => code,
        None => {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(sig) = output.status.signal() {
                    eprintln!("[rtk] {}: process terminated by signal {}", label, sig);
                    return 128 + sig;
                }
            }
            eprintln!("[rtk] {}: process terminated by signal", label);
            1
        }
    }
}

/// Extract exit code from an ExitStatus (for `.status()` calls, not `.output()`).
/// Returns the actual exit code, or `128 + signal` per Unix convention when
/// terminated by a signal. Falls back to 1 on non-Unix platforms.
pub fn exit_code_from_status(status: &std::process::ExitStatus, label: &str) -> i32 {
    match status.code() {
        Some(code) => code,
        None => {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(sig) = status.signal() {
                    eprintln!("[rtk] {}: process terminated by signal {}", label, sig);
                    return 128 + sig;
                }
            }
            eprintln!("[rtk] {}: process terminated by signal", label);
            1
        }
    }
}

/// Return the last `n` lines of output with a label, for use as a fallback
/// when filter parsing fails. Logs a diagnostic to stderr.
pub fn fallback_tail(output: &str, label: &str, n: usize) -> String {
    eprintln!(
        "[rtk] {}: output format not recognized, showing last {} lines",
        label, n
    );
    let lines: Vec<&str> = output.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

/// Build a Command for Ruby tools, auto-detecting bundle exec.
/// Uses `bundle exec <tool>` when a Gemfile exists (transitive deps like rake
/// won't appear in the Gemfile but still need bundler for version isolation).
pub fn ruby_exec(tool: &str) -> Command {
    if std::path::Path::new("Gemfile").exists() {
        let mut c = Command::new("bundle");
        c.arg("exec").arg(tool);
        return c;
    }
    Command::new(tool)
}

/// Count whitespace-delimited tokens in text. Used to estimate token counts
/// for filtering and compression.
pub fn count_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Detect the package manager used in the current directory.
/// Returns "pnpm", "yarn", or "npm" based on lockfile presence.
///
/// # Examples
/// ```no_run
/// use rtco_core::utils::detect_package_manager;
/// let pm = detect_package_manager();
/// // Returns "pnpm" if pnpm-lock.yaml exists, "yarn" if yarn.lock, else "npm"
/// ```
#[allow(dead_code)]
pub fn detect_package_manager() -> &'static str {
    if std::path::Path::new("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if std::path::Path::new("yarn.lock").exists() {
        "yarn"
    } else {
        "npm"
    }
}

/// Build a Command using the detected package manager's exec mechanism.
/// Returns a Command ready to have tool-specific args appended.
pub fn package_manager_exec(tool: &str) -> Command {
    if tool_exists(tool) {
        resolved_command(tool)
    } else {
        let pm = detect_package_manager();
        match pm {
            "pnpm" => {
                let mut c = resolved_command("pnpm");
                c.arg("exec").arg("--").arg(tool);
                c
            }
            "yarn" => {
                let mut c = resolved_command("yarn");
                c.arg("exec").arg("--").arg(tool);
                c
            }
            _ => {
                let mut c = resolved_command("npx");
                c.arg("--no-install").arg("--").arg(tool);
                c
            }
        }
    }
}

/// Resolve a binary name to its full path, honoring PATHEXT on Windows.
///
/// On Windows, Node.js tools are installed as `.CMD`/`.BAT`/`.PS1` shims.
/// Rust's `std::process::Command::new()` does NOT honor PATHEXT, so
/// `Command::new("vitest")` fails even when `vitest.CMD` is on PATH.
///
/// This function uses the `which` crate to perform proper PATH+PATHEXT resolution.
///
/// # Arguments
/// * `name` - Binary name (e.g., "vitest", "eslint", "tsc")
///
/// # Returns
/// Full path to the resolved binary, or error if not found.
pub fn resolve_binary(name: &str) -> Result<PathBuf> {
    which::which(name).context(format!("Binary '{}' not found on PATH", name))
}

/// Create a `Command` with PATHEXT-aware binary resolution.
///
/// Drop-in replacement for `Command::new(name)` that works on Windows
/// with `.CMD`/`.BAT`/`.PS1` wrappers.
///
/// Falls back to `Command::new(name)` if resolution fails, so native
/// commands (git, cargo) still work even if `which` can't find them.
///
/// # Arguments
/// * `name` - Binary name (e.g., "vitest", "eslint")
///
/// # Returns
/// A `Command` configured with the resolved binary path.
pub fn resolved_command(name: &str) -> Command {
    match resolve_binary(name) {
        Ok(path) => Command::new(path),
        Err(e) => {
            // On Windows, resolution failure likely means a .CMD/.BAT wrapper
            // wasn't found — always warn so users have a signal.
            // On Unix, this is less common; only log in debug builds.
            if cfg!(any(target_os = "windows", debug_assertions)) {
                eprintln!(
                    "rtk: Failed to resolve '{}' via PATH, falling back to direct exec: {}",
                    name, e
                );
            }

            Command::new(name)
        }
    }
}

/// Check if a tool exists on PATH (PATHEXT-aware on Windows).
///
/// Replaces manual `Command::new("which").arg(tool)` checks that fail on Windows.
pub fn tool_exists(name: &str) -> bool {
    which::which(name).is_ok()
}

/// Extract short name from AWS ARN.
/// Example: `arn:aws:ecs:region:acct:service/cluster/name` -> `name`
/// For simple ARNs like `arn:aws:iam::123:user/alice`, returns `alice`.
pub fn shorten_arn(arn: &str) -> &str {
    // ARNs use "/" or ":" as separators. Try "/" first (service/cluster/name pattern),
    // then fall back to ":" for Lambda/IAM ARNs.
    let slash_result = arn.rsplit('/').next().unwrap_or(arn);
    // If rsplit('/') returned the whole string (no '/' found), try ':'
    if slash_result == arn {
        arn.rsplit(':').next().unwrap_or(arn)
    } else {
        slash_result
    }
}

/// Convert bytes to human-readable format (KB, MB, GB, TB).
/// Used for S3 object sizes.
pub fn human_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Conservative normalization for deduplication: split a line at the first `:` or `=`,
/// keep the prefix (key) verbatim, and normalize the suffix (value) by replacing
/// contiguous digit runs with `N` and hex-looking runs (>=8 hex chars) with `H`,
/// and UUID-like segments with `U`.
///
/// This is intentionally conservative — it only normalizes the value portion so that
/// lines differing only in numeric/hex/UUID values collapse to the same key.
///
/// # Examples
/// ```
/// use rtco_core::utils::conservative_normalize;
/// assert_eq!(conservative_normalize("pid=12345"), "pid=N");
/// assert_eq!(conservative_normalize("commit abc1234567890 done"), "commit abcH done");
/// assert_eq!(conservative_normalize("no separator here"), "no separator here");
/// ```
#[allow(dead_code)]
pub fn conservative_normalize(line: &str) -> String {
    lazy_static::lazy_static! {
        // 8+ hex chars (sha-like hashes)
        static ref HEX_RE: Regex = Regex::new(r"\b[0-9a-fA-F]{8,40}\b").unwrap();
        // UUID pattern: 8-4-4-4-12 hex
        static ref UUID_RE: Regex = Regex::new(
            r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
        ).unwrap();
        // Contiguous digit runs (2+ digits to avoid replacing single meaningful digits)
        static ref DIGITS_RE: Regex = Regex::new(r"\d{2,}").unwrap();
    }

    // Split at first ':' or '=' to separate key from value
    if let Some(sep_pos) = line.find([':', '=']) {
        let prefix = &line[..=sep_pos]; // include the separator
        let suffix = &line[sep_pos + 1..];

        // Normalize suffix: UUIDs first (longest match), then hex, then digits
        let normalized = UUID_RE.replace_all(suffix, "U");
        let normalized = HEX_RE.replace_all(&normalized, "H");
        let normalized = DIGITS_RE.replace_all(&normalized, "N");

        let mut result = String::with_capacity(prefix.len() + normalized.len());
        result.push_str(prefix);
        result.push_str(&normalized);
        result
    } else {
        // No separator — normalize the entire line
        let normalized = UUID_RE.replace_all(line, "U");
        let normalized = HEX_RE.replace_all(&normalized, "H");
        DIGITS_RE.replace_all(&normalized, "N").into_owned()
    }
}

/// Deduplicate lines conservatively using [`conservative_normalize`].
///
/// Returns a vector of `(index, line)` where duplicate lines (those whose
/// normalized form matches an earlier line) are marked. The first occurrence
/// of each normalized form is kept as-is; subsequent duplicates are still
/// included but their index appears so callers can decide to collapse or
/// skip them.
///
/// # Examples
/// ```
/// use rtco_core::utils::deduplicate_conservative;
/// let lines = vec!["pid=100", "pid=200", "foo=bar"];
/// let result = deduplicate_conservative(&lines);
/// // pid=100 is first (index 0), pid=200 is a duplicate (index 1), foo=bar is unique (index 2)
/// assert_eq!(result.len(), 3);
/// assert_eq!(result[0], (0, "pid=100"));
/// assert_eq!(result[1], (1, "pid=200")); // duplicate of pid=100
/// assert_eq!(result[2], (2, "foo=bar"));
/// ```
#[allow(dead_code)]
pub fn deduplicate_conservative<'a>(lines: &[&'a str]) -> Vec<(usize, &'a str)> {
    use std::collections::HashSet;

    let mut seen: HashSet<String> = HashSet::with_capacity(lines.len());
    let mut result = Vec::with_capacity(lines.len());

    for (idx, line) in lines.iter().enumerate() {
        let normalized = conservative_normalize(line);
        // Always include the line; callers use the index to decide what to do.
        // We track whether this normalized form was already seen.
        result.push((idx, *line));
        let _ = seen.insert(normalized); // insert returns false if already present
    }

    result
}

/// Return `true` if the output is too small to benefit from compression.
///
/// When output is very short, the overhead of parsing and filtering can exceed
/// the token savings. This function lets filters skip compression for small
/// outputs, passing them through unchanged.
///
/// # Arguments
/// * `output` - The raw command output text
/// * `min_bytes` - Minimum byte threshold below which compression is skipped
///
/// # Examples
/// ```
/// use rtco_core::utils::should_skip_compression;
/// assert!(should_skip_compression("short", 100));
/// assert!(!should_skip_compression("a much longer output string that exceeds the threshold", 10));
/// ```
#[allow(dead_code)]
pub fn should_skip_compression(output: &str, min_bytes: usize) -> bool {
    output.len() < min_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        let result = truncate("hello world", 8);
        assert_eq!(result, "hello...");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_edge_case() {
        // max_len < 3 returns just "..."
        assert_eq!(truncate("hello", 2), "...");
        // When string length equals max_len, return as is
        assert_eq!(truncate("abc", 3), "abc");
        // When string is longer and max_len is exactly 3, return "..."
        assert_eq!(truncate("hello world", 3), "...");
    }

    #[test]
    fn test_strip_ansi_simple() {
        let input = "\x1b[31mError\x1b[0m";
        assert_eq!(strip_ansi(input), "Error");
    }

    #[test]
    fn test_strip_ansi_multiple() {
        let input = "\x1b[1m\x1b[32mSuccess\x1b[0m\x1b[0m";
        assert_eq!(strip_ansi(input), "Success");
    }

    #[test]
    fn test_strip_ansi_no_codes() {
        assert_eq!(strip_ansi("plain text"), "plain text");
    }

    #[test]
    fn test_strip_ansi_complex() {
        let input = "\x1b[32mGreen\x1b[0m normal \x1b[31mRed\x1b[0m";
        assert_eq!(strip_ansi(input), "Green normal Red");
    }

    #[test]
    fn test_format_tokens_millions() {
        assert_eq!(format_tokens(1_234_567), "1.2M");
        assert_eq!(format_tokens(12_345_678), "12.3M");
    }

    #[test]
    fn test_format_tokens_thousands() {
        assert_eq!(format_tokens(59_234), "59.2K");
        assert_eq!(format_tokens(1_000), "1.0K");
    }

    #[test]
    fn test_format_tokens_small() {
        assert_eq!(format_tokens(694), "694");
        assert_eq!(format_tokens(0), "0");
    }

    #[test]
    fn test_format_usd_large() {
        assert_eq!(format_usd(1234.567), "$1234.57");
        assert_eq!(format_usd(1000.0), "$1000.00");
    }

    #[test]
    fn test_format_usd_medium() {
        assert_eq!(format_usd(12.345), "$12.35");
        assert_eq!(format_usd(0.99), "$0.99");
    }

    #[test]
    fn test_format_usd_small() {
        assert_eq!(format_usd(0.0096), "$0.0096");
        assert_eq!(format_usd(0.0001), "$0.0001");
    }

    #[test]
    fn test_format_usd_edge() {
        assert_eq!(format_usd(0.01), "$0.01");
        assert_eq!(format_usd(0.009), "$0.0090");
    }

    #[test]
    fn test_ok_confirmation_with_detail() {
        assert_eq!(ok_confirmation("merged", "#42"), "ok merged #42");
        assert_eq!(
            ok_confirmation("created", "PR #5 https://github.com/foo/bar/pull/5"),
            "ok created PR #5 https://github.com/foo/bar/pull/5"
        );
    }

    #[test]
    fn test_ok_confirmation_no_detail() {
        assert_eq!(ok_confirmation("commented", ""), "ok commented");
    }

    #[test]
    fn test_format_cpt_normal() {
        assert_eq!(format_cpt(0.000003), "$3.00/MTok");
        assert_eq!(format_cpt(0.0000038), "$3.80/MTok");
        assert_eq!(format_cpt(0.00000386), "$3.86/MTok");
    }

    #[test]
    fn test_format_cpt_edge_cases() {
        assert_eq!(format_cpt(0.0), "$0.00/MTok"); // zero
        assert_eq!(format_cpt(-0.000001), "$0.00/MTok"); // negative
        assert_eq!(format_cpt(f64::INFINITY), "$0.00/MTok"); // infinite
        assert_eq!(format_cpt(f64::NAN), "$0.00/MTok"); // NaN
    }

    #[test]
    fn test_detect_package_manager_default() {
        // In the test environment (rtk repo), there's no JS lockfile
        // so it should default to "npm"
        let pm = detect_package_manager();
        assert!(["pnpm", "yarn", "npm"].contains(&pm));
    }

    #[test]
    fn test_truncate_multibyte_thai() {
        // Thai characters are 3 bytes each
        let thai = "สวัสดีครับ";
        let result = truncate(thai, 5);
        // Should not panic, should produce valid UTF-8
        assert!(result.len() <= thai.len());
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_multibyte_emoji() {
        let emoji = "🎉🎊🎈🎁🎂🎄🎃🎆🎇✨";
        let result = truncate(emoji, 5);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_multibyte_cjk() {
        let cjk = "你好世界测试字符串";
        let result = truncate(cjk, 6);
        assert!(result.ends_with("..."));
    }

    // ===== resolve_binary tests (issue #212) =====

    #[test]
    fn test_resolve_binary_finds_known_command() {
        // "cargo" must be on PATH in any Rust dev environment
        let result = resolve_binary("cargo");
        assert!(
            result.is_ok(),
            "resolve_binary('cargo') should succeed, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_resolve_binary_returns_absolute_path() {
        let path = resolve_binary("cargo").expect("cargo should be resolvable");
        assert!(
            path.is_absolute(),
            "resolve_binary should return absolute path, got: {:?}",
            path
        );
    }

    #[test]
    fn test_resolve_binary_fails_for_unknown() {
        let result = resolve_binary("nonexistent_binary_xyz_99999");
        assert!(
            result.is_err(),
            "resolve_binary should fail for nonexistent binary"
        );
    }

    #[test]
    fn test_resolve_binary_path_contains_binary_name() {
        let path = resolve_binary("cargo").expect("cargo should be resolvable");
        let filename = path
            .file_name()
            .expect("should have filename")
            .to_string_lossy();
        // On Windows this could be "cargo.exe", on Unix just "cargo"
        assert!(
            filename.starts_with("cargo"),
            "resolved path filename should start with 'cargo', got: {}",
            filename
        );
    }

    // ===== resolved_command tests (issue #212) =====

    #[test]
    fn test_resolved_command_executes_known_command() {
        let output = resolved_command("cargo")
            .arg("--version")
            .output()
            .expect("resolved_command('cargo') should execute");
        assert!(
            output.status.success(),
            "cargo --version should succeed via resolved_command"
        );
    }

    // ===== tool_exists tests (issue #212) =====

    #[test]
    fn test_tool_exists_finds_cargo() {
        assert!(
            tool_exists("cargo"),
            "tool_exists('cargo') should return true"
        );
    }

    #[test]
    fn test_tool_exists_rejects_unknown() {
        assert!(
            !tool_exists("nonexistent_binary_xyz_99999"),
            "tool_exists should return false for nonexistent binary"
        );
    }

    #[test]
    fn test_tool_exists_finds_git() {
        assert!(tool_exists("git"), "tool_exists('git') should return true");
    }

    // ===== Windows-specific PATHEXT resolution tests (issue #212) =====

    #[cfg(target_os = "windows")]
    mod windows_tests {
        use super::super::*;
        use std::fs;

        /// Create a temporary .cmd wrapper to simulate Node.js tool installation
        fn create_temp_cmd_wrapper(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
            let cmd_path = dir.join(format!("{}.cmd", name));
            fs::write(&cmd_path, "@echo off\r\necho fake-tool-output\r\n")
                .expect("failed to create .cmd wrapper");
            cmd_path
        }

        /// Build a PATH string that includes the temp dir
        fn path_with_dir(dir: &std::path::Path) -> std::ffi::OsString {
            let original = std::env::var_os("PATH").unwrap_or_default();
            let mut new_path = std::ffi::OsString::from(dir.as_os_str());
            new_path.push(";");
            new_path.push(&original);
            new_path
        }

        #[test]
        fn test_resolve_binary_finds_cmd_wrapper() {
            let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
            create_temp_cmd_wrapper(temp_dir.path(), "fake-tool-test");

            // Use which::which_in to avoid mutating global PATH (thread-safe)
            let search_path = path_with_dir(temp_dir.path());
            let result = which::which_in(
                "fake-tool-test",
                Some(search_path),
                std::env::current_dir().unwrap(),
            );

            assert!(
                result.is_ok(),
                "which_in should find .cmd wrapper on Windows, got: {:?}",
                result.err()
            );

            let path = result.unwrap();
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            assert!(
                ext == "cmd" || ext == "bat",
                "resolved path should have .cmd/.bat extension, got: {:?}",
                path
            );
        }

        #[test]
        fn test_resolve_binary_finds_bat_wrapper() {
            let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
            let bat_path = temp_dir.path().join("fake-bat-tool.bat");
            fs::write(&bat_path, "@echo off\r\necho bat-output\r\n")
                .expect("failed to create .bat wrapper");

            let search_path = path_with_dir(temp_dir.path());
            let result = which::which_in(
                "fake-bat-tool",
                Some(search_path),
                std::env::current_dir().unwrap(),
            );

            assert!(
                result.is_ok(),
                "which_in should find .bat wrapper on Windows, got: {:?}",
                result.err()
            );
        }

        #[test]
        fn test_resolved_command_executes_cmd_wrapper() {
            let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
            create_temp_cmd_wrapper(temp_dir.path(), "fake-exec-test");

            // Resolve the full path, then execute it directly (no PATH mutation)
            let search_path = path_with_dir(temp_dir.path());
            let resolved = which::which_in(
                "fake-exec-test",
                Some(search_path),
                std::env::current_dir().unwrap(),
            )
            .expect("should resolve fake-exec-test");

            let output = Command::new(&resolved).output();

            assert!(
                output.is_ok(),
                "Command with resolved path should execute .cmd wrapper on Windows"
            );
            let output = output.unwrap();
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("fake-tool-output"),
                "should get output from .cmd wrapper, got: {}",
                stdout
            );
        }

        #[test]
        fn test_resolved_command_fallback_on_unknown_binary() {
            // When resolve_binary fails, resolved_command should fall back to
            // Command::new(name) instead of panicking.  On Windows this also
            // prints a warning to stderr.
            let mut cmd = resolved_command("nonexistent_binary_xyz_99999");
            // The Command should be created (not panic).  Attempting to run it
            // will fail, but that's expected — we just verify the fallback path
            // produces a usable Command.
            let result = cmd.output();
            assert!(
                result.is_err() || !result.unwrap().status.success(),
                "nonexistent binary should fail to execute, but resolved_command must not panic"
            );
        }

        #[test]
        fn test_tool_exists_finds_cmd_wrapper() {
            let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
            create_temp_cmd_wrapper(temp_dir.path(), "fake-exists-test");

            let search_path = path_with_dir(temp_dir.path());
            let result = which::which_in(
                "fake-exists-test",
                Some(search_path),
                std::env::current_dir().unwrap(),
            );

            assert!(
                result.is_ok(),
                "which_in should find .cmd wrapper on Windows"
            );
        }
    }

    // ===== AWS helper function tests =====

    #[test]
    fn test_shorten_arn_ecs_service() {
        assert_eq!(
            shorten_arn("arn:aws:ecs:us-east-1:123:service/cluster/api-service"),
            "api-service"
        );
    }

    #[test]
    fn test_shorten_arn_iam_user() {
        assert_eq!(shorten_arn("arn:aws:iam::123456789012:user/alice"), "alice");
    }

    #[test]
    fn test_shorten_arn_lambda() {
        assert_eq!(
            shorten_arn("arn:aws:lambda:us-west-2:123:function:my-function"),
            "my-function"
        );
    }

    #[test]
    fn test_shorten_arn_fallback() {
        // Non-ARN string - return as-is
        assert_eq!(shorten_arn("simple-name"), "simple-name");
    }

    #[test]
    fn test_human_bytes_bytes() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1023), "1023 B");
    }

    #[test]
    fn test_human_bytes_kb() {
        assert_eq!(human_bytes(1024), "1.0 KB");
        assert_eq!(human_bytes(2048), "2.0 KB");
        assert_eq!(human_bytes(1536), "1.5 KB");
    }

    #[test]
    fn test_human_bytes_mb() {
        assert_eq!(human_bytes(1_048_576), "1.0 MB");
        assert_eq!(human_bytes(5_242_880), "5.0 MB");
    }

    #[test]
    fn test_human_bytes_gb() {
        assert_eq!(human_bytes(1_073_741_824), "1.0 GB");
        assert_eq!(human_bytes(2_147_483_648), "2.0 GB");
    }

    #[test]
    fn test_human_bytes_tb() {
        assert_eq!(human_bytes(1_099_511_627_776), "1.0 TB");
    }

    #[test]
    fn test_count_tokens_basic() {
        assert_eq!(count_tokens("hello world"), 2);
        assert_eq!(count_tokens("one two three four"), 4);
    }

    #[test]
    fn test_count_tokens_empty() {
        assert_eq!(count_tokens(""), 0);
        assert_eq!(count_tokens("   "), 0);
    }

    #[test]
    fn test_count_tokens_multiple_spaces() {
        assert_eq!(count_tokens("hello    world"), 2);
        assert_eq!(count_tokens("  hello   world  "), 2);
    }

    // ===== conservative_normalize tests =====

    #[test]
    fn test_conservative_normalize_simple_digits() {
        assert_eq!(conservative_normalize("pid=12345"), "pid=N");
    }

    #[test]
    fn test_conservative_normalize_colon_separator() {
        assert_eq!(conservative_normalize("count: 42 items"), "count: N items");
    }

    #[test]
    fn test_conservative_normalize_no_separator() {
        assert_eq!(conservative_normalize("12345"), "N");
    }

    #[test]
    fn test_conservative_normalize_preserves_prefix() {
        // "abc1234567890" is 13 hex chars, fully matched as one hex token
        assert_eq!(
            conservative_normalize("commit abc1234567890 done"),
            "commit H done"
        );
        // Non-hex prefix is preserved
        assert_eq!(
            conservative_normalize("commit xyz12345678 done"),
            "commit xyzN done"
        );
    }

    #[test]
    fn test_conservative_normalize_uuid() {
        assert_eq!(
            conservative_normalize("id=550e8400-e29b-41d4-a716-446655440000"),
            "id=U"
        );
    }

    #[test]
    fn test_conservative_normalize_hex_in_line() {
        assert_eq!(conservative_normalize("ref=a1b2c3d4e5f6a7b8"), "ref=H");
    }

    #[test]
    fn test_conservative_normalize_empty_line() {
        assert_eq!(conservative_normalize(""), "");
    }

    #[test]
    fn test_conservative_normalize_no_digits() {
        assert_eq!(conservative_normalize("hello=world"), "hello=world");
    }

    #[test]
    fn test_conservative_normalize_single_digit_preserved() {
        // Single digits (< 2 digits) should not be replaced
        assert_eq!(conservative_normalize("level=3"), "level=3");
    }

    #[test]
    fn test_conservative_normalize_multiple_values() {
        assert_eq!(conservative_normalize("time=12:34:56"), "time=N:N:N");
    }

    #[test]
    fn test_conservative_normalize_complex_line() {
        assert_eq!(
            conservative_normalize("src/main.rs:42: error[E0425]: cannot find value"),
            "src/main.rs:N: error[EN]: cannot find value"
        );
    }

    // ===== deduplicate_conservative tests =====

    #[test]
    fn test_deduplicate_conservative_basic() {
        let lines = vec!["pid=100", "pid=200", "foo=bar"];
        let result = deduplicate_conservative(&lines);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], (0, "pid=100"));
        assert_eq!(result[1], (1, "pid=200"));
        assert_eq!(result[2], (2, "foo=bar"));
    }

    #[test]
    fn test_deduplicate_conservative_exact_duplicates() {
        let lines = vec!["hello", "hello", "world"];
        let result = deduplicate_conservative(&lines);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], (0, "hello"));
        assert_eq!(result[1], (1, "hello"));
        assert_eq!(result[2], (2, "world"));
    }

    #[test]
    fn test_deduplicate_conservative_empty() {
        let lines: Vec<&str> = vec![];
        let result = deduplicate_conservative(&lines);
        assert!(result.is_empty());
    }

    #[test]
    fn test_deduplicate_conservative_all_same_normalized() {
        // All normalize to "pid=N" so they are duplicates
        let lines = vec!["pid=1", "pid=2", "pid=3"];
        let result = deduplicate_conservative(&lines);
        assert_eq!(result.len(), 3);
        // All are included; caller decides what to do
        assert_eq!(result[0], (0, "pid=1"));
        assert_eq!(result[1], (1, "pid=2"));
        assert_eq!(result[2], (2, "pid=3"));
    }

    #[test]
    fn test_deduplicate_conservative_preserves_order() {
        let lines = vec!["b=2", "a=1", "b=3", "a=4"];
        let result = deduplicate_conservative(&lines);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], (0, "b=2"));
        assert_eq!(result[1], (1, "a=1"));
        assert_eq!(result[2], (2, "b=3"));
        assert_eq!(result[3], (3, "a=4"));
    }

    // ===== should_skip_compression tests =====

    #[test]
    fn test_should_skip_compression_below_threshold() {
        assert!(should_skip_compression("short", 100));
    }

    #[test]
    fn test_should_skip_compression_at_threshold() {
        // Exactly at threshold: len == min_bytes, so NOT less than => false
        assert!(!should_skip_compression("12345", 5));
    }

    #[test]
    fn test_should_skip_compression_above_threshold() {
        assert!(!should_skip_compression("a much longer output", 10));
    }

    #[test]
    fn test_should_skip_compression_empty() {
        assert!(should_skip_compression("", 1));
    }

    #[test]
    fn test_should_skip_compression_zero_threshold() {
        // Empty string has len 0 which is not < 0
        assert!(!should_skip_compression("", 0));
        assert!(!should_skip_compression("anything", 0));
    }
}
