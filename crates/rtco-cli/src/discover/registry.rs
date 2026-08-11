//! Matches shell commands against known RTCO rewrite rules to decide how to handle them.

use regex::{Regex, RegexSet};
use rtco_core::utils::composer_bin_dirs;
use std::path::Path;
use std::sync::LazyLock;

use super::lexer::{
    shell_split, split_on_operators, tokenize, tokenize_with_newlines, ParsedToken, TokenKind,
};
use super::rules::{IGNORED_EXACT, IGNORED_PREFIXES, RULES};

const PHP_TOOL_NAMES: [&str; 6] = ["phpunit", "phpstan", "ecs", "pest", "paratest", "pint"];

/// Result of classifying a command.
#[derive(Debug, PartialEq)]
pub enum Classification {
    Supported {
        rtco_equivalent: &'static str,
        category: &'static str,
        estimated_savings_pct: f64,
        status: super::report::RtcoStatus,
    },
    Unsupported {
        base_command: String,
    },
    Ignored,
}

/// Average token counts per category for estimation when no output_len available.
pub fn category_avg_tokens(category: &str, subcmd: &str) -> usize {
    match category {
        "Git" => match subcmd {
            "log" | "diff" | "show" => 200,
            _ => 40,
        },
        "Cargo" => match subcmd {
            "test" => 500,
            _ => 150,
        },
        "Tests" => 800,
        "Files" => 100,
        "Build" => 300,
        "Infra" => 120,
        "Network" => 150,
        "GitHub" => 200,
        "GitLab" => 200,
        "PackageManager" => 150,
        "NodeVersionManager" => 40,
        _ => 150,
    }
}

static REGEX_SET: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new(RULES.iter().map(|r| r.pattern)).expect("invalid regex patterns")
});
static COMPILED: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    RULES
        .iter()
        .map(|r| Regex::new(r.pattern).expect("invalid regex"))
        .collect()
});
static ENV_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    let double_quoted = r#""(?:[^"\\]|\\.)*""#;
    let single_quoted = r#"'(?:[^'\\]|\\.)*'"#;
    let unquoted = r#"[^\s]*"#;
    let env_value = format!("(?:{}|{}|{})", double_quoted, single_quoted, unquoted);
    let env_assign = format!(r#"[A-Z_][A-Z0-9_]*={}"#, env_value);
    Regex::new(&format!(r#"^(?:sudo\s+|env\s+|{}\s+)+"#, env_assign)).unwrap()
});
// Git global options that appear before the subcommand: -C <path>, -c <key=val>,
// --git-dir <dir>, --work-tree <dir>, and flag-only options (#163)
static GIT_GLOBAL_OPT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:(?:-C\s+\S+|-c\s+\S+|--git-dir(?:=\S+|\s+\S+)|--work-tree(?:=\S+|\s+\S+)|--no-pager|--no-optional-locks|--bare|--literal-pathspecs)\s+)+").unwrap()
});
// Issue #1362: each capture expects a SINGLE file argument (`\S+$`). Multi-file
// invocations like `head -3 a b c` fail to match so the segment is passed through
// to the native `head`/`tail` binary — which already handles multi-file with
// `==> name <==` banners that `rtco read --max-lines` cannot reproduce.
static HEAD_N: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^head\s+-(\d+)\s+(\S+)$").unwrap());
static HEAD_LINES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^head\s+--lines=(\d+)\s+(\S+)$").unwrap());
static TAIL_N: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^tail\s+-(\d+)\s+(\S+)$").unwrap());
static TAIL_N_SPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^tail\s+-n\s+(\d+)\s+(\S+)$").unwrap());
static TAIL_LINES_EQ: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^tail\s+--lines=(\d+)\s+(\S+)$").unwrap());
static TAIL_LINES_SPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^tail\s+--lines\s+(\d+)\s+(\S+)$").unwrap());

const GOLANGCI_GLOBAL_OPT_WITH_VALUE: &[&str] = &[
    "-c",
    "--color",
    "--config",
    "--cpu-profile-path",
    "--mem-profile-path",
    "--trace-path",
];

#[derive(Debug, Clone, Copy)]
struct GolangciRunParts<'a> {
    global_segment: &'a str,
    run_segment: &'a str,
}

/// Classify a single (already-split) command.
pub fn classify_command(cmd: &str) -> Classification {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return Classification::Ignored;
    }

    // Check ignored
    for exact in IGNORED_EXACT {
        if trimmed == *exact {
            return Classification::Ignored;
        }
    }
    for prefix in IGNORED_PREFIXES {
        if trimmed.starts_with(prefix) {
            return Classification::Ignored;
        }
    }

    // Strip env prefixes (sudo, env VAR=val, VAR=val)
    let stripped = ENV_PREFIX.replace(trimmed, "");
    let cmd_clean = stripped.trim();
    if cmd_clean.is_empty() {
        return Classification::Ignored;
    }

    // Normalize absolute binary paths: /usr/bin/grep → grep (#485)
    let cmd_normalized = strip_absolute_path(cmd_clean);
    // Strip git global options: git -C /tmp status → git status (#163)
    let cmd_normalized = strip_git_global_opts(&cmd_normalized);
    // Normalize PHP tool paths: vendor/bin/phpunit, bin/phpunit, or composer
    // custom bin-dir → phpunit (so one rule matches every Composer layout).
    let cmd_normalized = normalize_php_tool_command(&cmd_normalized);
    // Strip golangci-lint global options before `run` so classify/rewrite stays
    // aligned with the runtime wrapper behavior.
    let cmd_normalized = strip_golangci_global_opts(&cmd_normalized);
    let cmd_clean = cmd_normalized.as_str();

    // Exclude cat/head/tail with redirect operators — these are writes, not reads (#315)
    if cmd_clean.starts_with("cat ")
        || cmd_clean.starts_with("head ")
        || cmd_clean.starts_with("tail ")
    {
        let has_redirect = cmd_clean
            .split_whitespace()
            .skip(1)
            .any(|t| t.starts_with('>') || t == "<" || t.starts_with(">>"));
        if has_redirect {
            return Classification::Unsupported {
                base_command: cmd_clean
                    .split_whitespace()
                    .next()
                    .unwrap_or("cat")
                    .to_string(),
            };
        }
    }

    // Fast check with RegexSet — take the last (most specific) match
    let matches: Vec<usize> = REGEX_SET.matches(cmd_clean).into_iter().collect();
    if let Some(&idx) = matches.last() {
        let rule = &RULES[idx];

        // Extract subcommand for savings override and status detection
        let (savings, status) = if let Some(caps) = COMPILED[idx].captures(cmd_clean) {
            if let Some(sub) = caps.get(1) {
                let subcmd = sub.as_str();
                // Check if this subcommand has a special status
                let status = rule
                    .subcmd_status
                    .iter()
                    .find(|(s, _)| *s == subcmd)
                    .map(|(_, st)| *st)
                    .unwrap_or(super::report::RtcoStatus::Existing);

                // Check if this subcommand has custom savings
                let savings = rule
                    .subcmd_savings
                    .iter()
                    .find(|(s, _)| *s == subcmd)
                    .map(|(_, pct)| *pct)
                    .unwrap_or(rule.savings_pct);

                (savings, status)
            } else {
                (rule.savings_pct, super::report::RtcoStatus::Existing)
            }
        } else {
            (rule.savings_pct, super::report::RtcoStatus::Existing)
        };

        Classification::Supported {
            rtco_equivalent: rule.rtco_cmd,
            category: rule.category,
            estimated_savings_pct: savings,
            status,
        }
    } else {
        // Extract base command for unsupported
        let base = extract_base_command(cmd_clean);
        if base.is_empty() {
            Classification::Ignored
        } else {
            Classification::Unsupported {
                base_command: base.to_string(),
            }
        }
    }
}

/// Extract the base command (first word, or first two if it looks like a subcommand pattern).
fn extract_base_command(cmd: &str) -> &str {
    let parts: Vec<&str> = cmd.splitn(3, char::is_whitespace).collect();
    match parts.len() {
        0 => "",
        1 => parts[0],
        _ => {
            let second = parts[1];
            // If the second token looks like a subcommand (no leading -)
            if !second.starts_with('-') && !second.contains('/') && !second.contains('.') {
                // Return "cmd subcmd"
                let end = cmd
                    .find(char::is_whitespace)
                    .and_then(|i| {
                        let rest = &cmd[i..];
                        let trimmed = rest.trim_start();
                        trimmed
                            .find(char::is_whitespace)
                            .map(|j| i + (rest.len() - trimmed.len()) + j)
                    })
                    .unwrap_or(cmd.len());
                &cmd[..end]
            } else {
                parts[0]
            }
        }
    }
}

/// Quote-aware heredoc detection — `<<` inside quotes is not a heredoc.
pub fn has_heredoc(cmd: &str) -> bool {
    tokenize(cmd)
        .iter()
        .any(|t| t.kind == TokenKind::Redirect && t.value.starts_with("<<"))
}

pub fn split_command_chain(cmd: &str) -> Vec<&str> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    // Lexer-based for `<<`; string-based for `$((` (lexer splits it across tokens).
    if has_heredoc(trimmed) || trimmed.contains("$((") {
        return vec![trimmed];
    }

    split_on_operators(trimmed, true)
}

/// Strip git global options before the subcommand (#163).
/// `git -C /tmp status` → `git status`, preserving the rest.
/// Returns the original string unchanged if not a git command.
fn strip_git_global_opts(cmd: &str) -> String {
    // Only applies to commands starting with "git "
    if !cmd.starts_with("git ") {
        return cmd.to_string();
    }
    let after_git = &cmd[4..]; // skip "git "
    let stripped = GIT_GLOBAL_OPT.replace(after_git, "");
    format!("git {}", stripped.trim())
}

/// Strip golangci-lint global options before the `run` subcommand.
/// `golangci-lint --color never run ./...` → `golangci-lint run ./...`
/// Returns the original string unchanged if this is not a supported compact `run` invocation.
fn strip_golangci_global_opts(cmd: &str) -> String {
    match parse_golangci_run_parts(cmd) {
        Some(parts) => format!("golangci-lint {}", parts.run_segment),
        None => cmd.to_string(),
    }
}

/// Parse supported golangci-lint invocations with optional global flags before `run`.
fn parse_golangci_run_parts(cmd: &str) -> Option<GolangciRunParts<'_>> {
    let tokens = split_token_spans(cmd);
    let first = tokens.first()?;
    if first.0 != "golangci-lint" && first.0 != "golangci" {
        return None;
    }

    let mut i = 1;
    while i < tokens.len() {
        let token = tokens[i].0;

        if token == "--" {
            return None;
        }

        if !token.starts_with('-') {
            if token == "run" {
                let global_segment = if i > 1 {
                    cmd[tokens[1].1..tokens[i].1].trim()
                } else {
                    ""
                };
                let run_segment = cmd[tokens[i].1..].trim();
                return Some(GolangciRunParts {
                    global_segment,
                    run_segment,
                });
            }
            return None;
        }

        if let Some(flag) = split_golangci_flag_name(token) {
            if golangci_flag_takes_separate_value(token, flag) {
                i += 1;
            }
        }

        i += 1;
    }

    None
}

fn split_golangci_flag_name(arg: &str) -> Option<&str> {
    if arg.starts_with("--") {
        return Some(arg.split_once('=').map(|(flag, _)| flag).unwrap_or(arg));
    }

    if arg.starts_with('-') {
        return Some(arg);
    }

    None
}

fn golangci_flag_takes_separate_value(arg: &str, flag: &str) -> bool {
    if !GOLANGCI_GLOBAL_OPT_WITH_VALUE.contains(&flag) {
        return false;
    }

    if arg.starts_with("--") && arg.contains('=') {
        return false;
    }

    true
}

fn split_token_spans(cmd: &str) -> Vec<(&str, usize, usize)> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (idx, ch) in cmd.char_indices() {
        if ch.is_whitespace() {
            if let Some(token_start) = start.take() {
                tokens.push((&cmd[token_start..idx], token_start, idx));
            }
        } else if start.is_none() {
            start = Some(idx);
        }
    }

    if let Some(token_start) = start {
        tokens.push((&cmd[token_start..], token_start, cmd.len()));
    }

    tokens
}

fn normalize_php_tool_command(cmd: &str) -> String {
    // Peel `php ` then normalize so `php vendor/bin/phpunit` and
    // `vendor/bin/phpunit` both collapse to the bare tool name.
    let unwrapped = strip_php_wrapper(cmd);
    normalize_php_tool_command_with_dirs(unwrapped, &composer_bin_dirs())
}

/// Peel a leading `php` interpreter wrapper off a Composer-tool invocation
/// (`php vendor/bin/phpunit …` → `vendor/bin/phpunit …`).
fn strip_php_wrapper(cmd: &str) -> &str {
    cmd.strip_prefix("php ").map_or(cmd, str::trim_start)
}

fn normalize_php_tool_command_with_dirs(cmd: &str, bin_dirs: &[std::path::PathBuf]) -> String {
    let first_space = cmd.find(char::is_whitespace);
    let first_word = match first_space {
        Some(pos) => &cmd[..pos],
        None => cmd,
    };

    let Some(tool) = normalize_php_tool_word(first_word, bin_dirs) else {
        return cmd.to_string();
    };

    match first_space {
        Some(pos) => format!("{}{}", tool, &cmd[pos..]),
        None => tool.to_string(),
    }
}

fn normalize_php_tool_word<'a>(word: &str, bin_dirs: &'a [std::path::PathBuf]) -> Option<&'a str> {
    let normalized_word = normalize_php_tool_path(word);

    for tool in PHP_TOOL_NAMES {
        if normalized_word == tool {
            return Some(tool);
        }

        if bin_dirs
            .iter()
            .any(|bin_dir| matches_php_tool_path(&normalized_word, bin_dir, tool))
        {
            return Some(tool);
        }
    }

    None
}

fn matches_php_tool_path(word: &str, bin_dir: &Path, tool: &str) -> bool {
    let normalized_dir = normalize_php_tool_path(&bin_dir.to_string_lossy());
    let candidate = format!("{normalized_dir}/{tool}");
    word == candidate || word.ends_with(&format!("/{candidate}"))
}

fn normalize_php_tool_path(path: &str) -> String {
    let mut normalized = path.trim().replace('\\', "/");
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.to_string();
    }

    if let Some((stem, ext)) = normalized.rsplit_once('.') {
        if ["bat", "cmd", "exe", "ps1"]
            .iter()
            .any(|candidate| ext.eq_ignore_ascii_case(candidate))
        {
            normalized = stem.to_string();
        }
    }

    normalized
}

/// Normalize absolute binary paths: `/usr/bin/grep -rn foo` → `grep -rn foo` (#485)
/// Only strips if the first word contains a `/` (Unix path).
fn strip_absolute_path(cmd: &str) -> String {
    let first_space = cmd.find(' ');
    let first_word = match first_space {
        Some(pos) => &cmd[..pos],
        None => cmd,
    };
    if first_word.contains('/') {
        // Extract basename
        let basename = first_word.rsplit('/').next().unwrap_or(first_word);
        if basename.is_empty() {
            return cmd.to_string();
        }
        match first_space {
            Some(pos) => format!("{}{}", basename, &cmd[pos..]),
            None => basename.to_string(),
        }
    } else {
        cmd.to_string()
    }
}

pub fn prefix_contains_rtco_disabled(prefix_part: &str) -> bool {
    prefix_part.contains("RTCO_DISABLED=") || prefix_part.contains("RTK_DISABLED=")
}

/// Check if a command has RTCO_DISABLED= prefix (or legacy RTK_DISABLED=) in its env prefix portion.
pub fn cmd_has_rtco_disabled_prefix(cmd: &str) -> bool {
    let (prefix_part, _) = strip_disabled_prefix(cmd);
    prefix_contains_rtco_disabled(prefix_part)
}

/// Strip RTCO_DISABLED=X (or legacy RTK_DISABLED=X) and other env prefixes, returns `(env_prefix, actual_command)`.
pub fn strip_disabled_prefix(cmd: &str) -> (&str, &str) {
    let trimmed = cmd.trim();
    let stripped = ENV_PREFIX.replace(trimmed, "");
    // stripped is a Cow<str> that borrows from trimmed when no replacement happens.
    // We need to return a &str into the original, so compute the offset.
    let prefix_len = trimmed.len() - stripped.len();
    let prefix_part = &trimmed[..prefix_len];
    let rest = trimmed[prefix_len..].trim();
    (prefix_part, rest)
}

fn strip_trailing_redirects(cmd: &str) -> (&str, &str) {
    let tokens = tokenize(cmd);
    if tokens.is_empty() {
        return (cmd, "");
    }

    let mut redir_boundary = tokens.len();
    let mut i = tokens.len();
    while i > 0 {
        i -= 1;
        match tokens[i].kind {
            TokenKind::Redirect => {
                redir_boundary = i;
            }
            TokenKind::Arg => {
                if i > 0 && tokens[i - 1].kind == TokenKind::Redirect {
                    redir_boundary = i - 1;
                    i -= 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }

    if redir_boundary >= tokens.len() {
        return (cmd, "");
    }

    let cut = tokens[redir_boundary].offset;
    let cmd_part = cmd[..cut].trim_end();
    let redir_part = &cmd[cmd_part.len()..];
    (cmd_part, redir_part)
}

/// Matches a bash line-continuation: a backslash immediately followed by
/// `\n` or `\r\n`, *plus* any horizontal whitespace on the line before AND
/// after the break. This is what bash already collapses to a single space
/// before executing the command — rtco's hook matcher needs to do the same
/// so commands authored across multiple lines still hit the rewrite rules.
/// Consuming the trailing whitespace prevents double spaces in cases like
/// `git diff \<NL>HEAD~1`.
static LINE_CONTINUATION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)[ \t\x0B\x0C]*\\\r?\n[ \t\x0B\x0C]*").unwrap());

/// Bare `\<NL>` join, used by the heredoc/$(( guard before
/// `collapse_line_continuations` (which consumes surrounding whitespace).
static BASH_JOIN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\\\r?\n").unwrap());

/// Replace every bash line continuation with a single space, mirroring what
/// bash does before dispatching the command. Returns a borrowed `&str` when the
/// input contains no continuations, so the common fast path allocates nothing.
fn collapse_line_continuations(s: &str) -> std::borrow::Cow<'_, str> {
    LINE_CONTINUATION_RE.replace_all(s, " ")
}

/// Returns `None` if the command is unsupported or ignored (hook should pass through).
///
/// Handles compound commands (`&&`, `||`, `;`) by rewriting each segment independently.
/// For pipes (`|`), only rewrites the left-hand command (pipe targets stay raw),
/// but continues rewriting segments after subsequent `&&`/`||`/`;` operators.
/// Also strips user-configured transparent wrapper prefixes
/// (`[hooks].transparent_prefixes` in `config.toml`) before routing.
///
/// A transparent prefix is a wrapper command that doesn't change *what* is
/// being run, only *how* it's run — e.g. `docker exec mycontainer`,
/// `direnv exec .`, `poetry run`, or `bundle exec`. Stripping it lets the inner
/// command match a filter; the prefix is then re-prepended to the rewrite. The
/// built-in [`SHELL_PREFIX_BUILTINS`] (`noglob`, `command`, `builtin`, `exec`,
/// `nocorrect`) are always applied in addition to user-configured prefixes.
///
/// Matching is strict: a configured prefix `"foo bar"` matches a command that
/// starts with `"foo bar "` (or strictly equals `"foo bar"`), not anything
/// else. Matching is literal, not pattern-based: configure the exact concrete
/// prefix you use.
/// Check if a command is a shell builtin that must run in-process (#2508).
/// Builtins like cd, export, source, etc. have no effect when run as a subprocess.
fn is_shell_builtin(cmd: &str) -> bool {
    let first_word = cmd.split_whitespace().next().unwrap_or("");
    matches!(
        first_word,
        "cd" | "pushd"
            | "popd"
            | "export"
            | "source"
            | "alias"
            | "unalias"
            | "set"
            | "unset"
            | "ulimit"
            | "umask"
            | "trap"
            | "exec"
            | "exit"
            | "return"
            | "logout"
            | "type"
            | "builtin"
            | "enable"
            | "let"
            | "read"
            | "readonly"
            | "shift"
            | "shopt"
            | "declare"
            | "local"
            | "eval"
            | "."
            | "bg"
            | "fg"
            | "jobs"
            | "disown"
            | "wait"
            | "suspend"
            | "times"
    )
}

pub fn rewrite_command(
    cmd: &str,
    excluded: &[String],
    transparent_prefixes: &[String],
) -> Option<String> {
    // Bash joins `\<NL>` with nothing, so `<<` or `$((` can arrive split across
    // a continuation; the space-join below would erase them (#3188 review).
    if cmd.contains('\\') {
        let joined = BASH_JOIN_RE.replace_all(cmd, "");
        if has_heredoc(&joined) || joined.contains("$((") {
            return None;
        }
    }

    // Bash line continuations (`\<NL>`, `\<CRLF>`) and the leading whitespace that
    // follows are syntactically equivalent to a single space, but `cmd.trim()` does
    // not unwrap them so a leading backslash-newline used to defeat the whole matcher.
    // Normalize first, then trim. See issue #1564.
    let normalized = collapse_line_continuations(cmd);
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return None;
    }

    if has_heredoc(trimmed) || trimmed.contains("$((") {
        return None;
    }

    let compiled = compile_exclude_patterns(excluded);
    let normalized_prefixes = normalize_transparent_prefixes(transparent_prefixes);

    // Multi-line blocks (bash scripts, heredoc-free pipelines across lines):
    // rewrite each line independently through the single-line path. Ported
    // from upstream rtk (#1243, #3319).
    if trimmed.contains('\n') {
        return rewrite_multiline_block(trimmed, &compiled, &normalized_prefixes);
    }

    // Simple (non-compound) already-RTCO command — return as-is.
    // For compound commands that start with "rtco" (e.g. "rtco git add . && cargo test"),
    // fall through to rewrite_compound so the remaining segments get rewritten.
    let has_compound = trimmed.contains("&&")
        || trimmed.contains("||")
        || trimmed.contains(';')
        || trimmed.contains('|')
        || trimmed.contains(" & ");
    let is_rtco_style = trimmed.starts_with("rtco ");
    let is_bare_rtco = trimmed == "rtco";
    if !has_compound && (is_rtco_style || is_bare_rtco) {
        // #2508: If the inner command is a shell builtin (cd, export, source, etc.),
        // return passthrough so the shell runs the real builtin instead of rtco as
        // a subprocess (which would lose the side-effect).
        let inner = trimmed.strip_prefix("rtco ").unwrap_or("");
        let inner_trimmed = inner.trim();
        if is_shell_builtin(inner_trimmed) {
            return None; // passthrough — let the shell execute the builtin directly
        }
        return Some(trimmed.to_string());
    }

    rewrite_compound(trimmed, &compiled, &normalized_prefixes)
}

/// Rewrite a single (newline-free) line, dispatching compound lines
/// (`&&`, `||`, `;`, `|`) to `rewrite_compound` and simple lines to
/// `rewrite_segment`. Mirrors upstream rtk's `rewrite_single`, used by the
/// multi-line block rewriter to rewrite each line independently.
fn rewrite_single_line(
    line: &str,
    excluded: &[ExcludePattern],
    transparent_prefixes: &[String],
) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let has_compound = trimmed.contains("&&")
        || trimmed.contains("||")
        || trimmed.contains(';')
        || trimmed.contains('|')
        || trimmed.contains(" & ");
    if !has_compound && (trimmed.starts_with("rtco ") || trimmed == "rtco") {
        return Some(trimmed.to_string());
    }
    if has_compound {
        rewrite_compound(trimmed, excluded, transparent_prefixes)
    } else {
        rewrite_segment(trimmed, excluded, transparent_prefixes)
    }
}

// ── Multi-line block rewriting (port from upstream rtk #1243, #3319) ────────

/// Byte-offset iterator over a string that tracks single/double-quote state,
/// so newline split-point detection and balance checks ignore quoted text.
struct QuoteScan<'a> {
    bytes: &'a [u8],
    i: usize,
    in_single: bool,
    in_double: bool,
}

impl<'a> QuoteScan<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            bytes: s.as_bytes(),
            i: 0,
            in_single: false,
            in_double: false,
        }
    }

    fn balanced(&self) -> bool {
        !self.in_single && !self.in_double
    }
}

impl Iterator for QuoteScan<'_> {
    type Item = (usize, u8, bool, bool);

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.bytes.len() {
            let i = self.i;
            let b = self.bytes[i];
            if b == b'\\' && !self.in_single {
                self.i += 2;
                continue;
            }
            let item = (i, b, self.in_single, self.in_double);
            match b {
                b'\'' if !self.in_double => self.in_single = !self.in_single,
                b'"' if !self.in_single => self.in_double = !self.in_double,
                _ => {}
            }
            self.i += 1;
            return Some(item);
        }
        None
    }
}

const BLOCK_KEYWORDS: &[&str] = &[
    "for", "while", "until", "if", "then", "else", "elif", "fi", "do", "done", "case", "esac",
    "select", "function", "coproc", "{", "}", "(", ")",
];

/// Byte offset where an unquoted `#` at the start of a word begins a trailing
/// comment, if any. The lexer has no comment state, so the independence checks
/// must ignore comment text themselves: `git log | # keep pipeline` continues
/// the pipeline across the newline even though the line ends in comment text.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    // `#` starts a comment at any word start, incl. after an operator
    // byte — but not after `{`: `${#var}` is an expansion (#3188 review).
    QuoteScan::new(line).find_map(|(i, b, in_single, in_double)| {
        (b == b'#'
            && !in_single
            && !in_double
            && (i == 0
                || bytes[i - 1].is_ascii_whitespace()
                || matches!(bytes[i - 1], b'|' | b'&' | b';' | b'(' | b')')))
        .then_some(i)
    })
}

/// Unquoted `(`/`)` or `{`/`}` that don't balance within the line: an array
/// literal (`arr=(one`), function body (`foo() {`), or group spans lines, so
/// the lines around it are not independent commands.
fn line_has_unbalanced_grouping(code: &str) -> bool {
    let mut paren = 0i32;
    let mut brace = 0i32;
    for (_, b, in_single, in_double) in QuoteScan::new(code) {
        if in_single || in_double {
            continue;
        }
        match b {
            b'(' => paren += 1,
            b')' => paren -= 1,
            b'{' => brace += 1,
            b'}' => brace -= 1,
            _ => {}
        }
        if paren < 0 || brace < 0 {
            return true;
        }
    }
    paren != 0 || brace != 0
}

/// Unquoted `[[` / `]]` words that don't balance within the line: bash allows
/// a conditional expression to span lines (`[[ -f a &&` / `-f b ]]`), so the
/// surrounding lines are not independent commands.
fn line_has_unbalanced_test_brackets(code: &str) -> bool {
    let bytes = code.as_bytes();
    let mut depth = 0i32;
    for (i, b, in_single, in_double) in QuoteScan::new(code) {
        if in_single || in_double || !matches!(b, b'[' | b']') {
            continue;
        }
        let word_start = i == 0 || bytes[i - 1].is_ascii_whitespace();
        let word_end = bytes.get(i + 2).is_none_or(|c| c.is_ascii_whitespace());
        if bytes.get(i + 1) == Some(&b) && word_start && word_end {
            depth += if b == b'[' { 1 } else { -1 };
            if depth < 0 {
                return true;
            }
        }
    }
    depth != 0
}

/// Only `\'` inside `$'…'` diverges: bash keeps the string open, the lexer
/// closes it — an extra split point the newline-count check can't see (#3188).
fn ansi_c_quote_defeats_lexer(cmd: &str) -> bool {
    let bytes = cmd.as_bytes();
    let mut ansi_span = false;
    let mut backslash_run = 0u32;
    for (i, b, in_single, in_double) in QuoteScan::new(cmd) {
        if b == b'\'' && !in_double {
            if !in_single {
                ansi_span = i > 0 && bytes[i - 1] == b'$';
                backslash_run = 0;
            } else if ansi_span && backslash_run % 2 == 1 {
                return true;
            }
        } else if in_single {
            if b == b'\\' {
                backslash_run += 1;
            } else {
                backslash_run = 0;
            }
        }
    }
    false
}

fn quotes_balanced(cmd: &str) -> bool {
    let mut scan = QuoteScan::new(cmd);
    scan.by_ref().for_each(drop);
    scan.balanced()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineRole {
    Passive,
    Independent,
    ContinuesNext,
    Unsafe,
}

fn classify_line(line: &str) -> LineRole {
    if line.is_empty() || line.starts_with('#') {
        return LineRole::Passive;
    }
    let comment = comment_start(line);
    let code = comment.map_or(line, |i| line[..i].trim_end());
    let first = code.split_whitespace().next().unwrap_or("");
    if BLOCK_KEYWORDS.contains(&first) {
        return LineRole::Unsafe;
    }
    const CONTINUATION_OPS: [&str; 4] = ["&&", "||", "|&", "|"];
    if CONTINUATION_OPS.iter().any(|op| code.starts_with(op))
        || code.starts_with("((")
        || code.ends_with("))")
        || line_has_unbalanced_grouping(code)
        || line_has_unbalanced_test_brackets(code)
    {
        return LineRole::Unsafe;
    }
    if CONTINUATION_OPS.iter().any(|op| code.ends_with(op)) {
        // An operator behind a trailing comment can't be joined textually:
        // the comment-blind tokenizer would read the comment as command words.
        return if comment.is_some() {
            LineRole::Unsafe
        } else {
            LineRole::ContinuesNext
        };
    }
    LineRole::Independent
}

/// Rewrite each line of a multi-line block independently (issue #1243).
///
/// Split points are the newline tokens the quote-aware lexer emits, so a
/// newline inside a quoted string (e.g. a multi-line commit message) never
/// becomes a boundary. Lines continued by a trailing `&&`/`||`/`|`/`|&` are
/// joined and rewritten as one logical command through the single-line path —
/// joining is not byte-preserving: separators inside a joined unit collapse
/// to single spaces; any line [`classify_line`] marks unsafe passes the whole
/// block through. Blank lines and comment lines are preserved verbatim, as is
/// indentation and the original separator bytes (`\n` vs `\r\n`).
///
/// If any newline byte was swallowed by quote state, the block passes through
/// untouched.
fn rewrite_multiline_block(
    cmd: &str,
    excluded: &[ExcludePattern],
    transparent_prefixes: &[String],
) -> Option<String> {
    let newline_offsets: Vec<usize> = tokenize_with_newlines(cmd)
        .iter()
        .filter(|t| t.kind == TokenKind::Operator && t.value == "\n")
        .map(|t| t.offset)
        .collect();

    if ansi_c_quote_defeats_lexer(cmd) {
        return None;
    }

    // The lexer emits one newline token per `\r` and per `\n` (CRLF = two
    // tokens), so the parity check must count both bytes individually.
    let raw_breaks = cmd.chars().filter(|c| matches!(c, '\n' | '\r')).count();
    if raw_breaks != newline_offsets.len() {
        // Every newline swallowed by quote state with quotes balanced at EOF
        // is one logical command (a multi-line commit message), not a hidden
        // extra line; rewrite it whole.
        if newline_offsets.is_empty() && quotes_balanced(cmd) {
            return rewrite_single_line(cmd, excluded, transparent_prefixes);
        }
        return None;
    }

    let mut segments = Vec::with_capacity(newline_offsets.len() + 1);
    let mut start = 0;
    for &off in &newline_offsets {
        segments.push((start, &cmd[start..off]));
        start = off + 1;
    }
    segments.push((start, &cmd[start..]));

    let roles: Vec<LineRole> = segments
        .iter()
        .map(|(_, seg)| classify_line(seg.trim()))
        .collect();
    if roles.contains(&LineRole::Unsafe) {
        return None;
    }

    let mut any_changed = false;
    let mut result = String::with_capacity(cmd.len() + 32);
    let mut i = 0;
    while i < segments.len() {
        if i > 0 {
            let off = newline_offsets[i - 1];
            result.push_str(&cmd[off..off + 1]);
        }
        let (seg_off, seg) = segments[i];

        if roles[i] == LineRole::Passive {
            result.push_str(seg);
            i += 1;
            continue;
        }

        let mut end = i;
        while roles[end] == LineRole::ContinuesNext {
            let mut next = end + 1;
            while next < segments.len() && segments[next].1.trim().is_empty() {
                next += 1;
            }
            if next >= segments.len() {
                break;
            }
            if roles[next] == LineRole::Passive {
                // Comment line inside a continuation: the comment-blind
                // tokenizer would join it as command words (#3188 review).
                return None;
            }
            end = next;
        }

        // A joined unit is rebuilt through the single-line path: interior
        // newlines and blank lines collapse to single spaces, not preserved.
        let unit = if end == i {
            seg
        } else {
            let (last_off, last_seg) = segments[end];
            &cmd[seg_off..last_off + last_seg.len()]
        };
        let line = unit.trim();
        match rewrite_single_line(line, excluded, transparent_prefixes) {
            Some(rewritten) if rewritten != line => {
                any_changed = true;
                let indent = &seg[..seg.len() - seg.trim_start().len()];
                result.push_str(indent);
                result.push_str(&rewritten);
            }
            _ => result.push_str(unit),
        }
        i = end + 1;
    }

    if any_changed {
        Some(result)
    } else {
        None
    }
}

/// Where a pipeline starts/stops, so `rewrite_pipeline_final_stage` can
/// isolate the final `|` stage. Ported from upstream rtk (523c803).
struct PipelineAnalysis {
    end_offset: usize,
    next_clause_offset: Option<usize>,
    final_stage_start: Option<usize>,
}

fn analyze_pipeline(
    cmd: &str,
    tokens: &[ParsedToken],
    segment_start: usize,
    first_pipe_offset: usize,
) -> PipelineAnalysis {
    let next_clause_offset = tokens
        .iter()
        .find(|token| {
            token.offset > first_pipe_offset
                && (token.kind == TokenKind::Operator
                    || (token.kind == TokenKind::Shellism && token.value == "&"))
        })
        .map(|token| token.offset);
    let end_offset = next_clause_offset.unwrap_or(cmd.len());

    let mut stage_start = segment_start;
    let mut final_stage_start = None;
    let mut has_supported_structure = true;

    for token in tokens {
        if token.offset >= end_offset {
            break;
        }
        if token.offset < first_pipe_offset {
            continue;
        }
        if token.kind != TokenKind::Pipe {
            continue;
        }

        if cmd[stage_start..token.offset].trim().is_empty() {
            has_supported_structure = false;
        }

        stage_start = token.offset + token.value.len();
        final_stage_start = Some(stage_start);
    }

    if cmd[stage_start..end_offset].trim().is_empty() {
        has_supported_structure = false;
    }

    PipelineAnalysis {
        end_offset,
        next_clause_offset,
        final_stage_start: if has_supported_structure {
            final_stage_start
        } else {
            None
        },
    }
}

/// Rewrite the final stage of a pipeline (e.g. `git log | grep foo` →
/// `rtco git log | rtco grep foo`) when the rule is `pipeline_final_safe`.
/// Ported from upstream rtk (523c803).
fn rewrite_pipeline_final_stage(
    cmd: &str,
    segment_start: usize,
    analysis: PipelineAnalysis,
    excluded: &[ExcludePattern],
    transparent_prefixes: &[String],
) -> Option<String> {
    let final_stage_start = analysis.final_stage_start?;
    let final_stage = cmd[final_stage_start..analysis.end_offset].trim();

    rewrite_segment_inner(
        final_stage,
        excluded,
        transparent_prefixes,
        RewriteContext::PipelineFinal,
        0,
    )
    .filter(|rewritten| rewritten != final_stage)
    .map(|rewritten| {
        format!(
            "{} {}",
            cmd[segment_start..final_stage_start].trim(),
            rewritten
        )
    })
}

/// Rewrite a compound command (with `&&`, `||`, `;`, `|`) by rewriting each segment.
fn rewrite_compound(
    cmd: &str,
    excluded: &[ExcludePattern],
    transparent_prefixes: &[String],
) -> Option<String> {
    let tokens = tokenize(cmd);
    let mut result = String::with_capacity(cmd.len() + 32);
    let mut any_changed = false;
    let mut seg_start: usize = 0;

    for tok in &tokens {
        if tok.offset < seg_start {
            continue;
        }
        match tok.kind {
            TokenKind::Operator => {
                let seg = cmd[seg_start..tok.offset].trim();
                let rewritten = rewrite_segment(seg, excluded, transparent_prefixes)
                    .unwrap_or_else(|| seg.to_string());
                if rewritten != seg {
                    any_changed = true;
                }
                result.push_str(&rewritten);
                if tok.value == ";" {
                    result.push(';');
                    let after = tok.offset + tok.value.len();
                    if after < cmd.len() {
                        result.push(' ');
                    }
                } else {
                    result.push(' ');
                    result.push_str(&tok.value);
                    result.push(' ');
                }
                seg_start = tok.offset + tok.value.len();
                while seg_start < cmd.len() && cmd.as_bytes().get(seg_start) == Some(&b' ') {
                    seg_start += 1;
                }
            }
            TokenKind::Pipe => {
                let analysis = analyze_pipeline(cmd, &tokens, seg_start, tok.offset);
                let next_clause_offset = analysis.next_clause_offset;
                let pipeline = cmd[seg_start..analysis.end_offset].trim();
                let rewritten_pipeline = rewrite_pipeline_final_stage(
                    cmd,
                    seg_start,
                    analysis,
                    excluded,
                    transparent_prefixes,
                );

                if let Some(rewritten) = rewritten_pipeline {
                    any_changed = true;
                    result.push_str(&rewritten);
                } else {
                    result.push_str(pipeline);
                }

                match next_clause_offset {
                    Some(next_clause_offset) => {
                        seg_start = next_clause_offset;
                        continue;
                    }
                    None => {
                        return if any_changed { Some(result) } else { None };
                    }
                }
            }
            TokenKind::Shellism if tok.value == "&" => {
                let seg = cmd[seg_start..tok.offset].trim();
                let rewritten = rewrite_segment(seg, excluded, transparent_prefixes)
                    .unwrap_or_else(|| seg.to_string());
                if rewritten != seg {
                    any_changed = true;
                }
                result.push_str(&rewritten);
                result.push_str(" & ");
                seg_start = tok.offset + tok.value.len();
                while seg_start < cmd.len() && cmd.as_bytes().get(seg_start) == Some(&b' ') {
                    seg_start += 1;
                }
            }
            _ => {}
        }
    }

    let seg = cmd[seg_start..].trim();
    let rewritten =
        rewrite_segment(seg, excluded, transparent_prefixes).unwrap_or_else(|| seg.to_string());
    if rewritten != seg {
        any_changed = true;
    }
    result.push_str(&rewritten);

    if any_changed {
        Some(result)
    } else {
        None
    }
}

fn rewrite_line_range(cmd: &str) -> Option<String> {
    for re in [&*HEAD_N, &*HEAD_LINES] {
        if let Some(caps) = re.captures(cmd) {
            let n = caps.get(1)?.as_str();
            let file = caps.get(2)?.as_str();
            return Some(format!("rtco read {} --max-lines {}", file, n));
        }
    }
    if cmd.starts_with("head -") {
        return None;
    }
    for re in [
        &*TAIL_N,
        &*TAIL_N_SPACE,
        &*TAIL_LINES_EQ,
        &*TAIL_LINES_SPACE,
    ] {
        if let Some(caps) = re.captures(cmd) {
            let n = caps.get(1)?.as_str();
            let file = caps.get(2)?.as_str();
            return Some(format!("rtco read {} --tail-lines {}", file, n));
        }
    }
    None
}

/// Shell prefix builtins that modify how the shell runs a command
/// but don't change which command runs. Strip before routing, re-prepend after.
const SHELL_PREFIX_BUILTINS: &[&str] = &["noglob", "command", "builtin", "exec", "nocorrect"];

const MAX_PREFIX_DEPTH: usize = 10;

enum ExcludePattern {
    Regex(Regex),
    Prefix(String),
}

fn compile_exclude_patterns(patterns: &[String]) -> Vec<ExcludePattern> {
    patterns
        .iter()
        .filter_map(|pattern| {
            let trimmed = pattern.trim();
            if trimmed.is_empty() || trimmed == "^" {
                eprintln!(
                    "rtco: warning: ignoring trivial exclude_commands pattern '{}'",
                    pattern
                );
                return None;
            }
            let anchored = if trimmed.starts_with('^') {
                trimmed.to_string()
            } else {
                format!(r"^{}($|\s)", regex::escape(trimmed))
            };
            Some(match Regex::new(&anchored) {
                Ok(re) => ExcludePattern::Regex(re),
                Err(e) => {
                    eprintln!(
                        "rtco: warning: invalid exclude_commands pattern '{}': {}",
                        pattern, e
                    );
                    ExcludePattern::Prefix(trimmed.to_string())
                }
            })
        })
        .collect()
}

fn normalize_transparent_prefixes(prefixes: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = prefixes
        .iter()
        .map(|prefix| prefix.trim())
        .filter(|prefix| !prefix.is_empty())
        .map(str::to_string)
        .collect();

    // Match longer wrappers first so `docker exec mycontainer` wins over `docker`.
    normalized.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    normalized.dedup();
    normalized
}

/// Rewrite context — distinguishes a normal command from the final stage of
/// a pipeline, where only `pipeline_final_safe` rules may be rewritten.
/// Ported from upstream rtk (523c803).
#[derive(Clone, Copy, PartialEq, Eq)]
enum RewriteContext {
    Normal,
    PipelineFinal,
}

/// Whether grep/rg reads patterns from a file (`-f`/`--file`), in which case
/// rewriting the final pipeline stage would be unsafe.
fn search_uses_pattern_file(cmd: &str) -> bool {
    shell_split(cmd)
        .into_iter()
        .skip(1)
        .take_while(|arg| arg != "--")
        .any(|arg| {
            arg == "--file"
                || arg.starts_with("--file=")
                || arg
                    .strip_prefix('-')
                    .filter(|flags| !flags.starts_with('-'))
                    .is_some_and(|flags| flags.contains('f'))
        })
}

fn pipeline_final_command_is_safe(rtco_cmd: &str, cmd: &str) -> bool {
    !matches!(rtco_cmd, "rtco grep" | "rtco rg") || !search_uses_pattern_file(cmd)
}

fn rewrite_segment(
    seg: &str,
    excluded: &[ExcludePattern],
    transparent_prefixes: &[String],
) -> Option<String> {
    rewrite_segment_inner(
        seg,
        excluded,
        transparent_prefixes,
        RewriteContext::Normal,
        0,
    )
}

fn is_excluded(cmd: &str, excluded: &[ExcludePattern]) -> bool {
    excluded.iter().any(|pat| match pat {
        ExcludePattern::Regex(re) => re.is_match(cmd),
        ExcludePattern::Prefix(prefix) => cmd.starts_with(prefix.as_str()),
    })
}

fn rewrite_segment_inner(
    seg: &str,
    excluded: &[ExcludePattern],
    transparent_prefixes: &[String],
    context: RewriteContext,
    depth: usize,
) -> Option<String> {
    let trimmed = seg.trim();
    if trimmed.is_empty() {
        return None;
    }

    if depth >= MAX_PREFIX_DEPTH {
        return None;
    }

    let (env_prefix, rest_after_env) = strip_disabled_prefix(trimmed);
    if !env_prefix.is_empty() {
        // #345: RTK_DISABLED=1 in env prefix → skip rewrite entirely (legacy compat)
        // #508: warn on stderr so agents learn to stop overusing it
        if env_prefix.contains("RTK_DISABLED=") {
            eprintln!(
                "[rtco] RTK_DISABLED=1 detected — skipping filter. \
                 Use RTCO_DISABLED=1 for the new env var."
            );
            return None;
        }
        if env_prefix.contains("RTCO_DISABLED=") {
            eprintln!(
                "[rtco] RTCO_DISABLED=1 detected — skipping filter for this command. \
                 Remove RTCO_DISABLED=1 to restore token savings."
            );
            return None;
        }
        let rewritten = rewrite_segment_inner(
            rest_after_env,
            excluded,
            transparent_prefixes,
            context,
            depth + 1,
        )?;
        return Some(format!("{}{}", env_prefix, rewritten));
    }

    for &prefix in SHELL_PREFIX_BUILTINS {
        if let Some(rest) = strip_word_prefix(trimmed, prefix) {
            if rest.is_empty() {
                return None;
            }
            return rewrite_segment_inner(rest, excluded, transparent_prefixes, context, depth + 1)
                .map(|rewritten| format!("{} {}", prefix, rewritten));
        }
    }

    // User-configured wrapper prefixes (e.g. `docker exec mycontainer`). Same
    // strip-recurse-reprepend contract as the builtin list above.
    for prefix in transparent_prefixes {
        if let Some(rest) = strip_word_prefix(trimmed, prefix) {
            if rest.is_empty() {
                return None;
            }
            return rewrite_segment_inner(rest, excluded, transparent_prefixes, context, depth + 1)
                .map(|rewritten| format!("{} {}", prefix, rewritten));
        }
    }

    // Strip trailing stderr/stdout redirects before matching (#530)
    // e.g. "git status 2>&1" → match "git status", re-append " 2>&1"
    let (cmd_part, redirect_suffix) = strip_trailing_redirects(trimmed);

    // Already RTCO (or the legacy `rtk` binary) — pass through unchanged.
    // The `rtk` check is intentional: keeps backward compat with shell history
    // and aliases that still invoke the old binary name.
    if cmd_part.starts_with("rtk ") || cmd_part.starts_with("rtco ") || cmd_part == "rtk" {
        return None;
    }

    // #2363: Apply exclude_commands check to head/tail rewrites
    if cmd_part.starts_with("head -") || cmd_part.starts_with("tail ") {
        if !is_excluded(cmd_part, excluded) {
            return rewrite_line_range(cmd_part).map(|r| format!("{}{}", r, redirect_suffix));
        }
        return None;
    }

    // Most cat flags (-v, -A, -e, -t, -s, -b, --show-all, etc.) have different
    // semantics than rtco read or no equivalent at all. Only `-n` (line numbers)
    // maps correctly to `rtco read -n`. Skip rewrite for any other flag.
    if let Some(cmd_args) = cmd_part.strip_prefix("cat ") {
        let args = cmd_args.trim_start();
        if args.starts_with('-') && !args.starts_with("-n ") && !args.starts_with("-n\t") {
            return None;
        }
    }

    // Use classify_command for correct ignore/prefix handling
    let rtco_equivalent = match classify_command(cmd_part) {
        Classification::Supported {
            rtco_equivalent, ..
        } => {
            let stripped = ENV_PREFIX.replace(cmd_part, "");
            let cmd_clean = stripped.trim();
            if is_excluded(cmd_clean, excluded) {
                return None;
            }
            rtco_equivalent
        }
        _ => return None,
    };

    // Find the matching rule (rtco_cmd values are unique across all rules)
    let rule = RULES.iter().find(|r| r.rtco_cmd == rtco_equivalent)?;
    if context == RewriteContext::PipelineFinal
        && (!rule.pipeline_final_safe || !pipeline_final_command_is_safe(rule.rtco_cmd, cmd_part))
    {
        return None;
    }

    if let Some(parts) = parse_golangci_run_parts(cmd_part) {
        let rewritten = if parts.global_segment.is_empty() {
            format!("rtco golangci-lint {}", parts.run_segment)
        } else {
            format!(
                "rtco golangci-lint {} {}",
                parts.global_segment, parts.run_segment
            )
        };
        return Some(rewritten);
    }

    // #196: gh with --json/--jq/--template produces structured output that
    // rtco gh would corrupt — skip rewrite so the caller gets raw JSON.
    if rule.rtco_cmd == "rtco gh" {
        let args_lower = cmd_part.to_lowercase();
        if args_lower.contains("--json")
            || args_lower.contains("--jq")
            || args_lower.contains("--template")
        {
            return None;
        }
    }

    // #664: RTCO `find` supports only a small subset of native `find` semantics.
    // Outside that subset, `rtco find` either errors loudly (-not/-exec/-delete)
    // or silently returns wrong results (multiple start paths, duplicate
    // predicates, -mindepth/-path, -type l, ...). Default-deny: only rewrite
    // invocations that fit RTCO's compact-find grammar. Otherwise let native
    // `find` run unchanged. This is a hook-layer transparency guard, not a
    // safety sandbox — destructive actions the user typed (-exec rm, -delete)
    // still execute via native find.
    if rule.rtco_cmd == "rtco find" && !is_supported_simple_find(cmd_part) {
        return None;
    }

    // For Composer-resolved PHP tools, normalize the leading invocation
    // (php wrapper, ./, vendor/bin, composer bin-dir) exactly as
    // classify_command does, so a small canonical prefix list matches every
    // invocation form instead of enumerating each literal spelling.
    let php_normalized;
    let strip_target: &str = if rule
        .rtco_cmd
        .strip_prefix("rtco ")
        .is_some_and(|t| PHP_TOOL_NAMES.contains(&t))
    {
        let unwrapped = strip_php_wrapper(cmd_part);
        let unwrapped = unwrapped.strip_prefix("./").unwrap_or(unwrapped);
        php_normalized = normalize_php_tool_command(unwrapped);
        &php_normalized
    } else {
        cmd_part
    };

    // Try each rewrite prefix (longest first) with word-boundary check
    for &prefix in rule.rewrite_prefixes {
        if let Some(rest) = strip_word_prefix(strip_target, prefix) {
            let rewritten = if rest.is_empty() {
                format!("{}{}", rule.rtco_cmd, redirect_suffix)
            } else {
                format!("{} {}{}", rule.rtco_cmd, rest, redirect_suffix)
            };
            return Some(rewritten);
        }
    }

    None
}

fn contains_glob_metachar(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

// #664: RTCO `find` only reproduces a narrow slice of native `find` semantics.
// Outside that slice it either errors loudly (-not/-exec/-delete/...) or
// silently returns wrong output (multiple start paths, duplicate predicates,
// -mindepth/-path, -type l, -maxdepth alone, file/missing start paths, ...).
// See `find_cmd::parse_native_find_args` for the divergences this guard
// prevents and the plan in PR #664-fix for the full taxonomy.
fn is_supported_simple_find(cmd_part: &str) -> bool {
    use crate::discover::lexer::shell_split;

    let Some(rest) = strip_word_prefix(cmd_part, "find") else {
        return false;
    };
    let args = shell_split(rest);
    if args.is_empty() {
        return false;
    }

    // Disambiguate by glob in args[0]:
    //   glob → Shape B (RTCO alias `find PATTERN [PATH] [-m N] [-t f|d]`)
    //   else → Shape A (native simple `find [PATH] (FLAG VALUE)+`)
    if contains_glob_metachar(&args[0]) {
        is_supported_rtco_alias(&args)
    } else {
        is_supported_native_simple(&args)
    }
}

fn is_supported_native_simple(args: &[String]) -> bool {
    use std::path::Path;

    let mut i = 0;

    // Optional single start path (non-flag, non-grouping).
    // Must be an existing directory at rewrite time — file roots, missing
    // paths, and unexpanded `~`/`$VAR` are declined to prevent silent-wrong
    // outputs (rtco strips the file root to empty; missing path → "0 for ...").
    if !args[i].starts_with('-') {
        if matches!(args[i].as_str(), "!" | "(" | ")") {
            return false;
        }
        if !Path::new(&args[i]).is_dir() {
            return false;
        }
        i += 1;
    }

    let mut seen_name_or_iname = false;
    let mut seen_type = false;
    let mut seen_maxdepth = false;
    // -name/-iname/-type only. -maxdepth alone is NOT a selector because
    // FindArgs::default() pins file_type="f" — rtco would drop directories
    // while native `find . -maxdepth 2` returns files AND directories.
    let mut seen_selector = false;

    while i < args.len() {
        match args[i].as_str() {
            "-name" | "-iname" => {
                if seen_name_or_iname {
                    return false;
                }
                let Some(v) = args.get(i + 1) else {
                    return false;
                };
                if v.starts_with('-') {
                    return false;
                }
                seen_name_or_iname = true;
                seen_selector = true;
                i += 2;
            }
            "-type" => {
                if seen_type {
                    return false;
                }
                let Some(v) = args.get(i + 1) else {
                    return false;
                };
                if !matches!(v.as_str(), "f" | "d") {
                    return false;
                }
                seen_type = true;
                seen_selector = true;
                i += 2;
            }
            "-maxdepth" => {
                if seen_maxdepth {
                    return false;
                }
                let Some(v) = args.get(i + 1) else {
                    return false;
                };
                match v.parse::<usize>() {
                    // Native `-maxdepth 0` prints the start path; rtco strips
                    // the search-root prefix to empty and skips it → "0 for *".
                    Ok(0) => return false,
                    Ok(_) => {}
                    Err(_) => return false,
                }
                seen_maxdepth = true;
                i += 2;
            }
            // Anything else (-not/-exec/-delete/-path/-mindepth/-printf/-ls,
            // future flags, extra positional args, `!`/`(`/`)`, `-o`/`-a`)
            // disqualifies.
            _ => return false,
        }
    }

    seen_selector
}

fn is_supported_rtco_alias(args: &[String]) -> bool {
    use std::path::Path;

    // args[0] is a glob pattern (checked by caller).
    let mut i = 1;

    // Optional second positional path: must be non-glob, non-grouping, and
    // resolve to an existing directory (same is_dir guard as Shape A).
    if i < args.len() && !args[i].starts_with('-') {
        if matches!(args[i].as_str(), "!" | "(" | ")") || contains_glob_metachar(&args[i]) {
            return false;
        }
        if !Path::new(&args[i]).is_dir() {
            return false;
        }
        i += 1;
    }

    let mut seen_max = false;
    let mut seen_type = false;

    while i < args.len() {
        match args[i].as_str() {
            "-m" | "--max" => {
                if seen_max {
                    return false;
                }
                let Some(v) = args.get(i + 1) else {
                    return false;
                };
                if v.parse::<usize>().is_err() {
                    return false;
                }
                seen_max = true;
                i += 2;
            }
            "-t" | "--file-type" => {
                if seen_type {
                    return false;
                }
                let Some(v) = args.get(i + 1) else {
                    return false;
                };
                if !matches!(v.as_str(), "f" | "d") {
                    return false;
                }
                seen_type = true;
                i += 2;
            }
            _ => return false,
        }
    }

    true
}

/// Strip a command prefix with word-boundary check.
/// Returns the remainder of the command after the prefix, or `None` if no match.
fn strip_word_prefix<'a>(cmd: &'a str, prefix: &str) -> Option<&'a str> {
    if cmd == prefix {
        Some("")
    } else if cmd.len() > prefix.len()
        && cmd.starts_with(prefix)
        && cmd.as_bytes()[prefix.len()] == b' '
    {
        Some(cmd[prefix.len() + 1..].trim_start())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::report::RtcoStatus;
    use super::*;

    fn rewrite_command_no_prefixes(cmd: &str, excluded: &[String]) -> Option<String> {
        super::rewrite_command(cmd, excluded, &[])
    }

    #[test]
    fn test_rewrite_vendor_bin_phpunit() {
        assert_eq!(
            rewrite_command_no_prefixes("vendor/bin/phpunit tests/", &[]),
            Some("rtco phpunit tests/".into())
        );
    }

    #[test]
    fn test_rewrite_php_vendor_bin_phpunit() {
        assert_eq!(
            rewrite_command_no_prefixes("php vendor/bin/phpunit tests/", &[]),
            Some("rtco phpunit tests/".into())
        );
    }

    #[test]
    fn test_normalize_php_tool_command_custom_bin_dir() {
        let dirs = vec![std::path::PathBuf::from("tools/bin")];
        assert_eq!(
            normalize_php_tool_command_with_dirs("tools/bin/phpunit tests/", &dirs),
            "phpunit tests/"
        );
        assert_eq!(
            normalize_php_tool_command_with_dirs("./tools/bin/pest", &dirs),
            "pest"
        );
    }

    #[test]
    fn test_classify_git_status() {
        assert_eq!(
            classify_command("git status"),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_yadm_status() {
        assert_eq!(
            classify_command("yadm status"),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_yadm_diff() {
        assert_eq!(
            classify_command("yadm diff"),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 80.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_rewrite_yadm_status() {
        assert_eq!(
            rewrite_command_no_prefixes("yadm status", &[]),
            Some("rtco git status".to_string())
        );
    }

    #[test]
    fn test_classify_git_diff_cached() {
        assert_eq!(
            classify_command("git diff --cached"),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 80.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_cargo_test_filter() {
        assert_eq!(
            classify_command("cargo test filter::"),
            Classification::Supported {
                rtco_equivalent: "rtco cargo",
                category: "Cargo",
                estimated_savings_pct: 90.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_npx_tsc() {
        assert_eq!(
            classify_command("npx tsc --noEmit"),
            Classification::Supported {
                rtco_equivalent: "rtco tsc",
                category: "Build",
                estimated_savings_pct: 83.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_cat_file() {
        assert_eq!(
            classify_command("cat src/main.rs"),
            Classification::Supported {
                rtco_equivalent: "rtco read",
                category: "Files",
                estimated_savings_pct: 60.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_cat_redirect_not_supported() {
        // cat > file and cat >> file are writes, not reads — should not be classified as supported
        let write_commands = [
            "cat > /tmp/output.txt",
            "cat >> /tmp/output.txt",
            "cat file.txt > output.txt",
            "cat -n file.txt >> log.txt",
            "head -10 README.md > output.txt",
            "tail -f app.log > /dev/null",
        ];
        for cmd in &write_commands {
            if let Classification::Supported { .. } = classify_command(cmd) {
                panic!("{} should NOT be classified as Supported", cmd)
            }
            // Unsupported or Ignored is fine
        }
    }

    #[test]
    fn test_classify_cd_ignored() {
        assert_eq!(classify_command("cd /tmp"), Classification::Ignored);
    }

    #[test]
    fn test_classify_rtk_already() {
        assert_eq!(classify_command("rtco git status"), Classification::Ignored);
    }

    #[test]
    fn test_classify_echo_ignored() {
        assert_eq!(
            classify_command("echo hello world"),
            Classification::Ignored
        );
    }

    #[test]
    fn test_classify_htop_unsupported() {
        match classify_command("htop -d 10") {
            Classification::Unsupported { base_command } => {
                assert_eq!(base_command, "htop");
            }
            other => panic!("expected Unsupported, got {:?}", other),
        }
    }

    #[test]
    fn test_classify_env_prefix_stripped() {
        assert_eq!(
            classify_command("GIT_SSH_COMMAND=ssh git push"),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_sudo_stripped() {
        assert_eq!(
            classify_command("sudo docker ps"),
            Classification::Supported {
                rtco_equivalent: "rtco docker",
                category: "Infra",
                estimated_savings_pct: 85.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_cargo_check() {
        assert_eq!(
            classify_command("cargo check"),
            Classification::Supported {
                rtco_equivalent: "rtco cargo",
                category: "Cargo",
                estimated_savings_pct: 80.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_cargo_check_all_targets() {
        assert_eq!(
            classify_command("cargo check --all-targets"),
            Classification::Supported {
                rtco_equivalent: "rtco cargo",
                category: "Cargo",
                estimated_savings_pct: 80.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_cargo_fmt_passthrough() {
        assert_eq!(
            classify_command("cargo fmt"),
            Classification::Supported {
                rtco_equivalent: "rtco cargo",
                category: "Cargo",
                estimated_savings_pct: 80.0,
                status: RtcoStatus::Passthrough,
            }
        );
    }

    #[test]
    fn test_classify_cargo_clippy_savings() {
        assert_eq!(
            classify_command("cargo clippy --all-targets"),
            Classification::Supported {
                rtco_equivalent: "rtco cargo",
                category: "Cargo",
                estimated_savings_pct: 80.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_registry_covers_all_cargo_subcommands() {
        // Verify that every CargoCommand variant (Build, Test, Clippy, Check, Fmt)
        // except Other has a matching pattern in the registry
        for subcmd in ["build", "test", "clippy", "check", "fmt"] {
            let cmd = format!("cargo {subcmd}");
            match classify_command(&cmd) {
                Classification::Supported { .. } => {}
                other => panic!("cargo {subcmd} should be Supported, got {other:?}"),
            }
        }
    }

    #[test]
    fn test_registry_covers_all_git_subcommands() {
        // Verify that every GitCommand subcommand has a matching pattern
        for subcmd in [
            "status", "log", "diff", "show", "add", "commit", "push", "pull", "branch", "fetch",
            "stash", "worktree",
        ] {
            let cmd = format!("git {subcmd}");
            match classify_command(&cmd) {
                Classification::Supported { .. } => {}
                other => panic!("git {subcmd} should be Supported, got {other:?}"),
            }
        }
    }

    #[test]
    fn test_classify_find_not_blocked_by_fi() {
        // Regression: "fi" in IGNORED_PREFIXES used to shadow "find" commands
        // because "find".starts_with("fi") is true. "fi" should only match exactly.
        assert_eq!(
            classify_command("find . -name foo"),
            Classification::Supported {
                rtco_equivalent: "rtco find",
                category: "Files",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_fi_still_ignored_exact() {
        // Bare "fi" (shell keyword) should still be ignored
        assert_eq!(classify_command("fi"), Classification::Ignored);
    }

    #[test]
    fn test_done_still_ignored_exact() {
        // Bare "done" (shell keyword) should still be ignored
        assert_eq!(classify_command("done"), Classification::Ignored);
    }

    #[test]
    fn test_split_chain_and() {
        assert_eq!(split_command_chain("a && b"), vec!["a", "b"]);
    }

    #[test]
    fn test_split_chain_semicolon() {
        assert_eq!(split_command_chain("a ; b"), vec!["a", "b"]);
    }

    #[test]
    fn test_split_pipe_first_only() {
        assert_eq!(split_command_chain("a | b"), vec!["a"]);
    }

    #[test]
    fn test_split_single() {
        assert_eq!(split_command_chain("git status"), vec!["git status"]);
    }

    #[test]
    fn test_split_quoted_and() {
        assert_eq!(
            split_command_chain(r#"echo "a && b""#),
            vec![r#"echo "a && b""#]
        );
    }

    #[test]
    fn test_split_heredoc_no_split() {
        let cmd = "cat <<'EOF'\nhello && world\nEOF";
        assert_eq!(split_command_chain(cmd), vec![cmd]);
    }

    #[test]
    fn test_classify_mypy() {
        assert_eq!(
            classify_command("mypy src/"),
            Classification::Supported {
                rtco_equivalent: "rtco mypy",
                category: "Build",
                estimated_savings_pct: 80.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_python_m_mypy() {
        assert_eq!(
            classify_command("python3 -m mypy --strict"),
            Classification::Supported {
                rtco_equivalent: "rtco mypy",
                category: "Build",
                estimated_savings_pct: 80.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    // --- rewrite_command tests ---

    #[test]
    fn test_rewrite_git_status() {
        assert_eq!(
            rewrite_command_no_prefixes("git status", &[]),
            Some("rtco git status".into())
        );
    }

    #[test]
    fn test_rewrite_git_log() {
        assert_eq!(
            rewrite_command_no_prefixes("git log -10", &[]),
            Some("rtco git log -10".into())
        );
    }

    // --- git -C <path> support (#555) ---

    #[test]
    fn test_rewrite_git_dash_c_status() {
        assert_eq!(
            rewrite_command_no_prefixes("git -C /path/to/repo status", &[]),
            Some("rtco git -C /path/to/repo status".into())
        );
    }

    #[test]
    fn test_rewrite_git_dash_c_log() {
        assert_eq!(
            rewrite_command_no_prefixes("git -C /tmp/myrepo log --oneline -5", &[]),
            Some("rtco git -C /tmp/myrepo log --oneline -5".into())
        );
    }

    #[test]
    fn test_rewrite_git_dash_c_diff() {
        assert_eq!(
            rewrite_command_no_prefixes("git -C /home/user/project diff --name-only", &[]),
            Some("rtco git -C /home/user/project diff --name-only".into())
        );
    }

    #[test]
    fn test_classify_git_dash_c() {
        let result = classify_command("git -C /tmp status");
        assert!(
            matches!(
                result,
                Classification::Supported {
                    rtco_equivalent: "rtco git",
                    ..
                }
            ),
            "git -C should be classified as supported, got: {:?}",
            result
        );
    }

    #[test]
    fn test_rewrite_cargo_test() {
        assert_eq!(
            rewrite_command_no_prefixes("cargo test", &[]),
            Some("rtco cargo test".into())
        );
    }

    #[test]
    fn test_rewrite_compound_and() {
        assert_eq!(
            rewrite_command_no_prefixes("git add . && cargo test", &[]),
            Some("rtco git add . && rtco cargo test".into())
        );
    }

    #[test]
    fn test_rewrite_compound_three_segments() {
        assert_eq!(
            rewrite_command_no_prefixes(
                "cargo fmt --all && cargo clippy --all-targets && cargo test",
                &[]
            ),
            Some(
                "rtco cargo fmt --all && rtco cargo clippy --all-targets && rtco cargo test".into()
            )
        );
    }

    #[test]
    fn test_rewrite_already_rtco() {
        assert_eq!(
            rewrite_command_no_prefixes("rtco git status", &[]),
            Some("rtco git status".into())
        );
    }

    #[test]
    fn test_rewrite_background_single_amp() {
        assert_eq!(
            rewrite_command_no_prefixes("cargo test & git status", &[]),
            Some("rtco cargo test & rtco git status".into())
        );
    }

    #[test]
    fn test_rewrite_background_unsupported_right() {
        assert_eq!(
            rewrite_command_no_prefixes("cargo test & htop", &[]),
            Some("rtco cargo test & htop".into())
        );
    }

    #[test]
    fn test_rewrite_background_does_not_affect_double_amp() {
        // `&&` must still work after adding `&` support
        assert_eq!(
            rewrite_command_no_prefixes("cargo test && git status", &[]),
            Some("rtco cargo test && rtco git status".into())
        );
    }

    #[test]
    fn test_rewrite_unsupported_returns_none() {
        assert_eq!(rewrite_command_no_prefixes("htop", &[]), None);
    }

    #[test]
    fn test_rewrite_ignored_cd() {
        assert_eq!(rewrite_command_no_prefixes("cd /tmp", &[]), None);
    }

    #[test]
    fn test_rewrite_with_env_prefix() {
        assert_eq!(
            rewrite_command_no_prefixes("GIT_SSH_COMMAND=ssh git push", &[]),
            Some("GIT_SSH_COMMAND=ssh rtco git push".into())
        );
    }

    #[test]
    fn test_rewrite_tsc() {
        let commands = vec![
            "npm exec tsc",
            "npm rum tsc",
            "npm run tsc",
            "npm run-script tsc",
            "npm urn tsc",
            "npm x tsc",
            "pnpm dlx tsc",
            "pnpm exec tsc",
            "pnpm run tsc",
            "pnpm run-script tsc",
            "npm tsc",
            "npx tsc",
            "pnpm tsc",
            "pnpx tsc",
            "tsc",
        ];
        for command in commands {
            assert_eq!(
                rewrite_command_no_prefixes(&format!("{command} --noEmit"), &[]),
                Some("rtco tsc --noEmit".into()),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_cat_file() {
        assert_eq!(
            rewrite_command_no_prefixes("cat src/main.rs", &[]),
            Some("rtco read src/main.rs".into())
        );
    }

    #[test]
    fn test_rewrite_cat_with_incompatible_flags_skipped() {
        // cat flags with different semantics than rtco read — skip rewrite
        assert_eq!(rewrite_command_no_prefixes("cat -A file.cpp", &[]), None);
        assert_eq!(rewrite_command_no_prefixes("cat -v file.txt", &[]), None);
        assert_eq!(rewrite_command_no_prefixes("cat -e file.txt", &[]), None);
        assert_eq!(rewrite_command_no_prefixes("cat -t file.txt", &[]), None);
        assert_eq!(rewrite_command_no_prefixes("cat -s file.txt", &[]), None);
        assert_eq!(
            rewrite_command_no_prefixes("cat --show-all file.txt", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_cat_with_compatible_flags() {
        // cat -n (line numbers) maps to rtco read -n — allow rewrite
        assert_eq!(
            rewrite_command_no_prefixes("cat -n file.txt", &[]),
            Some("rtco read -n file.txt".into())
        );
    }

    #[test]
    fn test_rewrite_rg_pattern() {
        assert_eq!(
            rewrite_command_no_prefixes("rg \"fn main\"", &[]),
            Some("rtco rg \"fn main\"".into())
        );
    }

    #[test]
    fn test_rewrite_playwright() {
        let commands = vec![
            "npm exec playwright",
            "npm rum playwright",
            "npm run playwright",
            "npm run-script playwright",
            "npm urn playwright",
            "npm x playwright",
            "pnpm dlx playwright",
            "pnpm exec playwright",
            "pnpm run playwright",
            "pnpm run-script playwright",
            "npm playwright",
            "npx playwright",
            "pnpm playwright",
            "pnpx playwright",
            "playwright",
        ];
        for command in commands {
            assert_eq!(
                rewrite_command_no_prefixes(&format!("{command} test"), &[]),
                Some("rtco playwright test".into()),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_next_build() {
        let commands = vec![
            "npm exec next build",
            "npm rum next build",
            "npm run next build",
            "npm run-script next build",
            "npm urn next build",
            "npm x next build",
            "pnpm dlx next build",
            "pnpm exec next build",
            "pnpm run next build",
            "pnpm run-script next build",
            "npm next build",
            "npx next build",
            "pnpm next build",
            "pnpx next build",
            "next build",
        ];
        for command in commands {
            assert_eq!(
                rewrite_command_no_prefixes(&format!("{command} --turbo"), &[]),
                Some("rtco next --turbo".into()),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_pipe_first_only() {
        // Producer stays raw; the pipeline-safe final grep stage is rewritten.
        assert_eq!(
            rewrite_command_no_prefixes("git log -10 | grep feat", &[]),
            Some("git log -10 | rtco grep feat".into())
        );
    }

    #[test]
    fn test_rewrite_find_pipe_skipped() {
        // find in a pipe should NOT be rewritten — rtco find output format
        // is incompatible with pipe consumers like xargs (#439)
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*.rs' | xargs grep 'fn run'", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_find_pipe_xargs_wc() {
        assert_eq!(
            rewrite_command_no_prefixes("find src -type f | wc -l", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_find_no_pipe_still_rewritten() {
        // find WITHOUT a pipe should still be rewritten
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*.rs'", &[]),
            Some("rtco find . -name '*.rs'".into())
        );
    }

    #[test]
    fn test_rewrite_heredoc_returns_none() {
        assert_eq!(
            rewrite_command_no_prefixes("cat <<'EOF'\nfoo\nEOF", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_empty_returns_none() {
        assert_eq!(rewrite_command_no_prefixes("", &[]), None);
        assert_eq!(rewrite_command_no_prefixes("   ", &[]), None);
    }

    #[test]
    fn test_rewrite_mixed_compound_partial() {
        // First segment already RTCO, second gets rewritten
        assert_eq!(
            rewrite_command_no_prefixes("rtco git add . && cargo test", &[]),
            Some("rtco git add . && rtco cargo test".into())
        );
    }

    // --- #345: RTK_DISABLED ---

    #[test]
    fn test_rewrite_rtk_disabled_curl() {
        assert_eq!(
            rewrite_command_no_prefixes("RTK_DISABLED=1 curl https://example.com", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_rtk_disabled_git_status() {
        assert_eq!(
            rewrite_command_no_prefixes("RTK_DISABLED=1 git status", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_rtk_disabled_multi_env() {
        assert_eq!(
            rewrite_command_no_prefixes("FOO=1 RTK_DISABLED=1 git status", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_rtk_disabled_warns_on_stderr() {
        assert_eq!(
            rewrite_command_no_prefixes("RTK_DISABLED=1 git status", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_rtk_disabled_subprocess_warns() {
        let rtco_bin = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("debug")
            .join("rtco");
        if !rtco_bin.exists() {
            return;
        }
        let rtco_mtime = std::fs::metadata(&rtco_bin)
            .ok()
            .and_then(|m| m.modified().ok());
        let test_mtime = std::env::current_exe()
            .ok()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());
        if let (Some(rtco_t), Some(test_t)) = (rtco_mtime, test_mtime) {
            if rtco_t < test_t {
                return;
            }
        }

        let output = std::process::Command::new(&rtco_bin)
            .args(["rewrite", "RTK_DISABLED=1 git status"])
            .output()
            .expect("Failed to run rtco");

        assert!(
            !output.status.success(),
            "Should exit non-zero (no rewrite)"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("RTK_DISABLED=1 detected"),
            "Should warn on stderr, got: {}",
            stderr
        );
    }

    #[test]
    fn test_rewrite_non_rtk_disabled_env_still_rewrites() {
        assert_eq!(
            rewrite_command_no_prefixes("SOME_VAR=1 git status", &[]),
            Some("SOME_VAR=1 rtco git status".into())
        );
    }

    #[test]
    fn test_rewrite_env_quoted_value_with_spaces() {
        assert_eq!(
            rewrite_command_no_prefixes(
                r#"GIT_SSH_COMMAND="ssh -o StrictHostKeyChecking=no" git push"#,
                &[]
            ),
            Some(r#"GIT_SSH_COMMAND="ssh -o StrictHostKeyChecking=no" rtco git push"#.into())
        );
    }

    #[test]
    fn test_rewrite_env_single_quoted_value_with_spaces() {
        assert_eq!(
            rewrite_command_no_prefixes("EDITOR='vim -u NONE' git commit", &[]),
            Some("EDITOR='vim -u NONE' rtco git commit".into())
        );
    }

    #[test]
    fn test_rewrite_env_quoted_plus_unquoted() {
        assert_eq!(
            rewrite_command_no_prefixes(r#"FOO="bar baz" BAR=1 git status"#, &[]),
            Some(r#"FOO="bar baz" BAR=1 rtco git status"#.into())
        );
    }

    #[test]
    fn test_rewrite_env_escaped_quotes_in_value() {
        assert_eq!(
            rewrite_command_no_prefixes(r#"FOO="he said \"hello\"" git status"#, &[]),
            Some(r#"FOO="he said \"hello\"" rtco git status"#.into())
        );
    }

    #[test]
    fn test_classify_env_quoted_value_stripped() {
        assert_eq!(
            classify_command(r#"GIT_SSH_COMMAND="ssh -o StrictHostKeyChecking=no" git push"#),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    // --- #346: 2>&1 and &> redirect detection ---

    #[test]
    fn test_rewrite_redirect_2_gt_amp_1_with_pipe() {
        // Producer stays raw and `head` is not pipeline_final_safe → passthrough.
        assert_eq!(
            rewrite_command_no_prefixes("cargo test 2>&1 | head", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_redirect_2_gt_amp_1_trailing() {
        assert_eq!(
            rewrite_command_no_prefixes("cargo test 2>&1", &[]),
            Some("rtco cargo test 2>&1".into())
        );
    }

    #[test]
    fn test_rewrite_redirect_plain_2_devnull() {
        // 2>/dev/null has no `&`, never broken — non-regression
        assert_eq!(
            rewrite_command_no_prefixes("git status 2>/dev/null", &[]),
            Some("rtco git status 2>/dev/null".into())
        );
    }

    #[test]
    fn test_rewrite_redirect_2_gt_amp_1_with_and() {
        assert_eq!(
            rewrite_command_no_prefixes("cargo test 2>&1 && echo done", &[]),
            Some("rtco cargo test 2>&1 && echo done".into())
        );
    }

    #[test]
    fn test_rewrite_redirect_amp_gt_devnull() {
        assert_eq!(
            rewrite_command_no_prefixes("cargo test &>/dev/null", &[]),
            Some("rtco cargo test &>/dev/null".into())
        );
    }

    #[test]
    fn test_rewrite_redirect_double() {
        // Double redirect: only last one stripped, but full command rewrites correctly
        assert_eq!(
            rewrite_command_no_prefixes("git status 2>&1 >/dev/null", &[]),
            Some("rtco git status 2>&1 >/dev/null".into())
        );
    }

    #[test]
    fn test_rewrite_redirect_fd_close() {
        // 2>&- (close stderr fd)
        assert_eq!(
            rewrite_command_no_prefixes("git status 2>&-", &[]),
            Some("rtco git status 2>&-".into())
        );
    }

    #[test]
    fn test_rewrite_redirect_quotes_not_stripped() {
        // Redirect-like chars inside quotes should NOT be stripped
        // Known limitation: apostrophes cause conservative no-strip (safe fallback)
        let result = rewrite_command_no_prefixes("git commit -m \"it's fixed\" 2>&1", &[]);
        assert!(
            result.is_some(),
            "Should still rewrite even with apostrophe"
        );
    }

    #[test]
    fn test_rewrite_background_amp_non_regression() {
        // background `&` must still work after redirect fix
        assert_eq!(
            rewrite_command_no_prefixes("cargo test & git status", &[]),
            Some("rtco cargo test & rtco git status".into())
        );
    }

    // --- P0.2: head -N rewrite ---

    #[test]
    fn test_rewrite_head_numeric_flag() {
        // head -20 file → rtco read file --max-lines 20 (not rtco read -20 file)
        assert_eq!(
            rewrite_command_no_prefixes("head -20 src/main.rs", &[]),
            Some("rtco read src/main.rs --max-lines 20".into())
        );
    }

    #[test]
    fn test_rewrite_head_lines_long_flag() {
        assert_eq!(
            rewrite_command_no_prefixes("head --lines=50 src/lib.rs", &[]),
            Some("rtco read src/lib.rs --max-lines 50".into())
        );
    }

    #[test]
    fn test_rewrite_head_no_flag_still_rewrites() {
        // plain `head file` → `rtco read file` (no numeric flag)
        assert_eq!(
            rewrite_command_no_prefixes("head src/main.rs", &[]),
            Some("rtco read src/main.rs".into())
        );
    }

    #[test]
    fn test_rewrite_head_other_flag_skipped() {
        // head -c 100 file: unsupported flag, skip rewriting
        assert_eq!(
            rewrite_command_no_prefixes("head -c 100 src/main.rs", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_tail_numeric_flag() {
        assert_eq!(
            rewrite_command_no_prefixes("tail -20 src/main.rs", &[]),
            Some("rtco read src/main.rs --tail-lines 20".into())
        );
    }

    #[test]
    fn test_rewrite_tail_n_space_flag() {
        assert_eq!(
            rewrite_command_no_prefixes("tail -n 12 src/lib.rs", &[]),
            Some("rtco read src/lib.rs --tail-lines 12".into())
        );
    }

    #[test]
    fn test_rewrite_tail_lines_long_flag() {
        assert_eq!(
            rewrite_command_no_prefixes("tail --lines=7 src/lib.rs", &[]),
            Some("rtco read src/lib.rs --tail-lines 7".into())
        );
    }

    #[test]
    fn test_rewrite_tail_lines_space_flag() {
        assert_eq!(
            rewrite_command_no_prefixes("tail --lines 7 src/lib.rs", &[]),
            Some("rtco read src/lib.rs --tail-lines 7".into())
        );
    }

    #[test]
    fn test_rewrite_tail_other_flag_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("tail -c 100 src/main.rs", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_tail_plain_file_skipped() {
        assert_eq!(rewrite_command_no_prefixes("tail src/main.rs", &[]), None);
    }

    // --- Issue #1362: head/tail with multiple files falls back to native command ---
    //
    // `rtco read <file> --max-lines N` only accepts a single positional file path in
    // a shape that maps cleanly to `head -N`. Rewriting `head -N a b c` to
    // `rtco read a b c --max-lines N` previously produced a command where `rtco read`
    // would concatenate the files without the `==> name <==` banners that native
    // `head` emits, so the fix is to skip the rewrite and let the shell run the
    // real `head`/`tail` binary.

    #[test]
    fn test_rewrite_head_numeric_flag_multi_file_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("head -3 /tmp/a /tmp/b /tmp/c", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_head_lines_long_flag_multi_file_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("head --lines=50 src/main.rs src/lib.rs", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_tail_numeric_flag_multi_file_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("tail -20 a.log b.log", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_tail_n_space_flag_multi_file_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("tail -n 12 a.log b.log c.log", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_tail_lines_eq_multi_file_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("tail --lines=7 a.log b.log", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_tail_lines_space_multi_file_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("tail --lines 7 a.log b.log", &[]),
            None
        );
    }

    // --- New registry entries ---

    #[test]
    fn test_classify_gh_release() {
        assert!(matches!(
            classify_command("gh release list"),
            Classification::Supported {
                rtco_equivalent: "rtco gh",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_glab_mr() {
        assert!(matches!(
            classify_command("glab mr list"),
            Classification::Supported {
                rtco_equivalent: "rtco glab",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_glab_ci() {
        assert!(matches!(
            classify_command("glab ci list"),
            Classification::Supported {
                rtco_equivalent: "rtco glab",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_glab_release() {
        assert!(matches!(
            classify_command("glab release list"),
            Classification::Supported {
                rtco_equivalent: "rtco glab",
                ..
            }
        ));
    }

    #[test]
    fn test_rewrite_glab_mr_list() {
        assert_eq!(
            rewrite_command_no_prefixes("glab mr list", &[]),
            Some("rtco glab mr list".into())
        );
    }

    #[test]
    fn test_rewrite_glab_ci_status() {
        assert_eq!(
            rewrite_command_no_prefixes("glab ci status", &[]),
            Some("rtco glab ci status".into())
        );
    }

    #[test]
    fn test_classify_cargo_install() {
        assert!(matches!(
            classify_command("cargo install rtk"),
            Classification::Supported {
                rtco_equivalent: "rtco cargo",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_docker_run() {
        assert!(matches!(
            classify_command("docker run --rm ubuntu bash"),
            Classification::Supported {
                rtco_equivalent: "rtco docker",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_docker_exec() {
        assert!(matches!(
            classify_command("docker exec -it mycontainer bash"),
            Classification::Supported {
                rtco_equivalent: "rtco docker",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_docker_build() {
        assert!(matches!(
            classify_command("docker build -t myimage ."),
            Classification::Supported {
                rtco_equivalent: "rtco docker",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_kubectl_describe() {
        assert!(matches!(
            classify_command("kubectl describe pod mypod"),
            Classification::Supported {
                rtco_equivalent: "rtco kubectl",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_kubectl_apply() {
        assert!(matches!(
            classify_command("kubectl apply -f deploy.yaml"),
            Classification::Supported {
                rtco_equivalent: "rtco kubectl",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_tree() {
        assert!(matches!(
            classify_command("tree src/"),
            Classification::Supported {
                rtco_equivalent: "rtco tree",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_diff() {
        assert!(matches!(
            classify_command("diff file1.txt file2.txt"),
            Classification::Supported {
                rtco_equivalent: "rtco diff",
                ..
            }
        ));
    }

    #[test]
    fn test_rewrite_tree() {
        assert_eq!(
            rewrite_command_no_prefixes("tree src/", &[]),
            Some("rtco tree src/".into())
        );
    }

    #[test]
    fn test_rewrite_diff() {
        assert_eq!(
            rewrite_command_no_prefixes("diff file1.txt file2.txt", &[]),
            Some("rtco diff file1.txt file2.txt".into())
        );
    }

    #[test]
    fn test_rewrite_gh_release() {
        assert_eq!(
            rewrite_command_no_prefixes("gh release list", &[]),
            Some("rtco gh release list".into())
        );
    }

    #[test]
    fn test_rewrite_cargo_install() {
        assert_eq!(
            rewrite_command_no_prefixes("cargo install rtk", &[]),
            Some("rtco cargo install rtk".into())
        );
    }

    #[test]
    fn test_rewrite_kubectl_describe() {
        assert_eq!(
            rewrite_command_no_prefixes("kubectl describe pod mypod", &[]),
            Some("rtco kubectl describe pod mypod".into())
        );
    }

    #[test]
    fn test_rewrite_docker_run() {
        assert_eq!(
            rewrite_command_no_prefixes("docker run --rm ubuntu bash", &[]),
            Some("rtco docker run --rm ubuntu bash".into())
        );
    }

    #[test]
    fn test_classify_swift_test() {
        assert!(matches!(
            classify_command("swift test"),
            Classification::Supported {
                rtco_equivalent: "rtco swift",
                category: "Build",
                estimated_savings_pct: 90.0,
                status: RtcoStatus::Existing,
            }
        ));
    }

    #[test]
    fn test_rewrite_swift_test() {
        assert_eq!(
            rewrite_command_no_prefixes("swift test --parallel", &[]),
            Some("rtco swift test --parallel".into())
        );
    }

    // --- #336: docker compose supported subcommands rewritten, unsupported skipped ---

    #[test]
    fn test_rewrite_docker_compose_ps() {
        assert_eq!(
            rewrite_command_no_prefixes("docker compose ps", &[]),
            Some("rtco docker compose ps".into())
        );
    }

    #[test]
    fn test_rewrite_docker_compose_logs() {
        assert_eq!(
            rewrite_command_no_prefixes("docker compose logs web", &[]),
            Some("rtco docker compose logs web".into())
        );
    }

    #[test]
    fn test_rewrite_docker_compose_build() {
        assert_eq!(
            rewrite_command_no_prefixes("docker compose build", &[]),
            Some("rtco docker compose build".into())
        );
    }

    #[test]
    fn test_rewrite_docker_compose_up_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("docker compose up -d", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_docker_compose_down_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("docker compose down", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_docker_compose_config_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("docker compose -f foo.yaml config --services", &[]),
            None
        );
    }

    // --- AWS / psql (PR #216) ---

    #[test]
    fn test_classify_aws() {
        assert!(matches!(
            classify_command("aws s3 ls"),
            Classification::Supported {
                rtco_equivalent: "rtco aws",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_aws_ec2() {
        assert!(matches!(
            classify_command("aws ec2 describe-instances"),
            Classification::Supported {
                rtco_equivalent: "rtco aws",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_psql() {
        assert!(matches!(
            classify_command("psql -U postgres"),
            Classification::Supported {
                rtco_equivalent: "rtco psql",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_psql_url() {
        assert!(matches!(
            classify_command("psql postgres://localhost/mydb"),
            Classification::Supported {
                rtco_equivalent: "rtco psql",
                ..
            }
        ));
    }

    #[test]
    fn test_rewrite_aws() {
        assert_eq!(
            rewrite_command_no_prefixes("aws s3 ls", &[]),
            Some("rtco aws s3 ls".into())
        );
    }

    #[test]
    fn test_rewrite_aws_ec2() {
        assert_eq!(
            rewrite_command_no_prefixes("aws ec2 describe-instances --region us-east-1", &[]),
            Some("rtco aws ec2 describe-instances --region us-east-1".into())
        );
    }

    #[test]
    fn test_rewrite_psql() {
        assert_eq!(
            rewrite_command_no_prefixes("psql -U postgres -d mydb", &[]),
            Some("rtco psql -U postgres -d mydb".into())
        );
    }

    // --- Python tooling ---

    #[test]
    fn test_classify_ruff_check() {
        assert!(matches!(
            classify_command("ruff check ."),
            Classification::Supported {
                rtco_equivalent: "rtco ruff",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_ruff_format() {
        assert!(matches!(
            classify_command("ruff format src/"),
            Classification::Supported {
                rtco_equivalent: "rtco ruff",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_pytest() {
        assert!(matches!(
            classify_command("pytest tests/"),
            Classification::Supported {
                rtco_equivalent: "rtco pytest",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_python_m_pytest() {
        assert!(matches!(
            classify_command("python -m pytest tests/"),
            Classification::Supported {
                rtco_equivalent: "rtco pytest",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_pip_list() {
        assert!(matches!(
            classify_command("pip list"),
            Classification::Supported {
                rtco_equivalent: "rtco pip",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_uv_pip_list() {
        assert!(matches!(
            classify_command("uv pip list"),
            Classification::Supported {
                rtco_equivalent: "rtco pip",
                ..
            }
        ));
    }

    #[test]
    fn test_rewrite_ruff_check() {
        assert_eq!(
            rewrite_command_no_prefixes("ruff check .", &[]),
            Some("rtco ruff check .".into())
        );
    }

    #[test]
    fn test_rewrite_ruff_format() {
        assert_eq!(
            rewrite_command_no_prefixes("ruff format src/", &[]),
            Some("rtco ruff format src/".into())
        );
    }

    #[test]
    fn test_rewrite_pytest() {
        assert_eq!(
            rewrite_command_no_prefixes("pytest tests/", &[]),
            Some("rtco pytest tests/".into())
        );
    }

    #[test]
    fn test_rewrite_python_m_pytest() {
        assert_eq!(
            rewrite_command_no_prefixes("python -m pytest -x tests/", &[]),
            Some("rtco pytest -x tests/".into())
        );
    }

    #[test]
    fn test_rewrite_pip_list() {
        assert_eq!(
            rewrite_command_no_prefixes("pip list", &[]),
            Some("rtco pip list".into())
        );
    }

    #[test]
    fn test_rewrite_pip_outdated() {
        assert_eq!(
            rewrite_command_no_prefixes("pip outdated", &[]),
            Some("rtco pip outdated".into())
        );
    }

    #[test]
    fn test_rewrite_uv_pip_list() {
        assert_eq!(
            rewrite_command_no_prefixes("uv pip list", &[]),
            Some("rtco pip list".into())
        );
    }

    // --- Go tooling ---

    #[test]
    fn test_classify_go_test() {
        assert!(matches!(
            classify_command("go test ./..."),
            Classification::Supported {
                rtco_equivalent: "rtco go",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_go_build() {
        assert!(matches!(
            classify_command("go build ./..."),
            Classification::Supported {
                rtco_equivalent: "rtco go",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_go_vet() {
        assert!(matches!(
            classify_command("go vet ./..."),
            Classification::Supported {
                rtco_equivalent: "rtco go",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_golangci_lint() {
        assert!(matches!(
            classify_command("golangci-lint run"),
            Classification::Supported {
                rtco_equivalent: "rtco golangci-lint run",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_golangci_lint_with_flag_before_run() {
        assert!(matches!(
            classify_command("golangci-lint -v run ./..."),
            Classification::Supported {
                rtco_equivalent: "rtco golangci-lint run",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_golangci_lint_with_value_flag_before_run() {
        assert!(matches!(
            classify_command("golangci-lint --color never run ./..."),
            Classification::Supported {
                rtco_equivalent: "rtco golangci-lint run",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_golangci_lint_with_inline_value_flag_before_run() {
        assert!(matches!(
            classify_command("golangci-lint --color=never run ./..."),
            Classification::Supported {
                rtco_equivalent: "rtco golangci-lint run",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_golangci_lint_with_inline_config_flag_before_run() {
        assert!(matches!(
            classify_command("golangci-lint --config=foo.yml run ./..."),
            Classification::Supported {
                rtco_equivalent: "rtco golangci-lint run",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_golangci_lint_bare_is_not_compact_wrapper() {
        assert!(!matches!(
            classify_command("golangci-lint"),
            Classification::Supported {
                rtco_equivalent: "rtco golangci-lint run",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_golangci_lint_other_subcommand_is_not_compact_wrapper() {
        assert!(!matches!(
            classify_command("golangci-lint version"),
            Classification::Supported {
                rtco_equivalent: "rtco golangci-lint run",
                ..
            }
        ));
    }

    #[test]
    fn test_rewrite_go_test() {
        assert_eq!(
            rewrite_command_no_prefixes("go test ./...", &[]),
            Some("rtco go test ./...".into())
        );
    }

    #[test]
    fn test_rewrite_go_build() {
        assert_eq!(
            rewrite_command_no_prefixes("go build ./...", &[]),
            Some("rtco go build ./...".into())
        );
    }

    #[test]
    fn test_rewrite_go_vet() {
        assert_eq!(
            rewrite_command_no_prefixes("go vet ./...", &[]),
            Some("rtco go vet ./...".into())
        );
    }

    #[test]
    fn test_rewrite_golangci_lint() {
        assert_eq!(
            rewrite_command_no_prefixes("golangci-lint run ./...", &[]),
            Some("rtco golangci-lint run ./...".into())
        );
    }

    #[test]
    fn test_rewrite_golangci_lint_with_flag_before_run() {
        assert_eq!(
            rewrite_command_no_prefixes("golangci-lint -v run ./...", &[]),
            Some("rtco golangci-lint -v run ./...".into())
        );
    }

    #[test]
    fn test_rewrite_golangci_lint_with_value_flag_before_run() {
        assert_eq!(
            rewrite_command_no_prefixes("golangci-lint --color never run ./...", &[]),
            Some("rtco golangci-lint --color never run ./...".into())
        );
    }

    #[test]
    fn test_rewrite_golangci_lint_with_inline_value_flag_before_run() {
        assert_eq!(
            rewrite_command_no_prefixes("golangci-lint --color=never run ./...", &[]),
            Some("rtco golangci-lint --color=never run ./...".into())
        );
    }

    #[test]
    fn test_rewrite_golangci_lint_with_inline_config_flag_before_run() {
        assert_eq!(
            rewrite_command_no_prefixes("golangci-lint --config=foo.yml run ./...", &[]),
            Some("rtco golangci-lint --config=foo.yml run ./...".into())
        );
    }

    #[test]
    fn test_rewrite_env_prefixed_golangci_lint_with_value_flag_before_run() {
        assert_eq!(
            rewrite_command_no_prefixes("FOO=1 golangci-lint --color never run ./...", &[]),
            Some("FOO=1 rtco golangci-lint --color never run ./...".into())
        );
    }

    #[test]
    fn test_rewrite_env_prefixed_golangci_lint_with_inline_value_flag_before_run() {
        assert_eq!(
            rewrite_command_no_prefixes("FOO=1 golangci-lint --color=never run ./...", &[]),
            Some("FOO=1 rtco golangci-lint --color=never run ./...".into())
        );
    }

    #[test]
    fn test_rewrite_bare_golangci_lint_skips_compact_wrapper() {
        assert_eq!(rewrite_command_no_prefixes("golangci-lint", &[]), None);
    }

    #[test]
    fn test_rewrite_other_golangci_lint_subcommand_skips_compact_wrapper() {
        assert_eq!(
            rewrite_command_no_prefixes("golangci-lint version", &[]),
            None
        );
    }

    // --- JS/TS tooling ---

    #[test]
    fn test_classify_lint() {
        let commands = vec![
            "npm exec biome",
            "npm exec eslint",
            "npm rum biome",
            "npm rum eslint",
            "npm rum lint",
            "npm run biome",
            "npm run eslint",
            "npm run lint",
            "npm run-script biome",
            "npm run-script eslint",
            "npm run-script lint",
            "npm urn biome",
            "npm urn eslint",
            "npm urn lint",
            "npm x biome",
            "npm x eslint",
            "pnpm dlx biome",
            "pnpm dlx eslint",
            "pnpm exec biome",
            "pnpm exec eslint",
            "pnpm run biome",
            "pnpm run eslint",
            "pnpm run lint",
            "pnpm run-script biome",
            "pnpm run-script eslint",
            "pnpm run-script lint",
            "npm biome",
            "npm eslint",
            "npm lint",
            "npx biome",
            "npx eslint",
            "npx lint",
            "pnpm biome",
            "pnpm eslint",
            "pnpm lint",
            "pnpx biome",
            "pnpx eslint",
            "pnpx lint",
            "biome",
            "eslint",
            "lint",
        ];
        for command in commands {
            assert!(
                matches!(
                    classify_command(command),
                    Classification::Supported {
                        rtco_equivalent: "rtco lint",
                        ..
                    }
                ),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_lint() {
        let commands = vec![
            "npm exec biome",
            "npm exec eslint",
            "npm rum biome",
            "npm rum eslint",
            "npm rum lint",
            "npm run biome",
            "npm run eslint",
            "npm run lint",
            "npm run-script biome",
            "npm run-script eslint",
            "npm run-script lint",
            "npm urn biome",
            "npm urn eslint",
            "npm urn lint",
            "npm x biome",
            "npm x eslint",
            "pnpm dlx biome",
            "pnpm dlx eslint",
            "pnpm exec biome",
            "pnpm exec eslint",
            "pnpm run biome",
            "pnpm run eslint",
            "pnpm run lint",
            "pnpm run-script biome",
            "pnpm run-script eslint",
            "pnpm run-script lint",
            "npm biome",
            "npm eslint",
            "npm lint",
            "npx biome",
            "npx eslint",
            "npx lint",
            "pnpm biome",
            "pnpm eslint",
            "pnpm lint",
            "pnpx biome",
            "pnpx eslint",
            "pnpx lint",
            "biome",
            "eslint",
            "lint",
        ];
        for command in commands {
            assert_eq!(
                rewrite_command_no_prefixes(command, &[]),
                Some("rtco lint".into()),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_classify_jest() {
        let commands = vec![
            "jest run",
            "jest",
            "npm exec jest run",
            "npm exec jest",
            "npm jest run",
            "npm jest",
            "npm rum jest run",
            "npm rum jest",
            "npm run jest run",
            "npm run jest",
            "npm run-script jest run",
            "npm run-script jest",
            "npm urn jest run",
            "npm urn jest",
            "npm x jest run",
            "npm x jest",
            "npx jest run",
            "npx jest",
            "pnpm dlx jest run",
            "pnpm dlx jest",
            "pnpm exec jest run",
            "pnpm exec jest",
            "pnpm jest run",
            "pnpm jest",
            "pnpm run jest run",
            "pnpm run jest",
            "pnpm run-script jest run",
            "pnpm run-script jest",
            "pnpx jest run",
            "pnpx jest",
        ];
        for command in commands {
            assert!(
                matches!(
                    classify_command(command),
                    Classification::Supported {
                        rtco_equivalent: "rtco jest",
                        ..
                    }
                ),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_jest() {
        let commands = vec![
            "jest run",
            "jest",
            "npm exec jest run",
            "npm exec jest",
            "npm jest run",
            "npm jest",
            "npm rum jest run",
            "npm rum jest",
            "npm run jest run",
            "npm run jest",
            "npm run-script jest run",
            "npm run-script jest",
            "npm urn jest run",
            "npm urn jest",
            "npm x jest run",
            "npm x jest",
            "npx jest run",
            "npx jest",
            "pnpm dlx jest run",
            "pnpm dlx jest",
            "pnpm exec jest run",
            "pnpm exec jest",
            "pnpm jest run",
            "pnpm jest",
            "pnpm run jest run",
            "pnpm run jest",
            "pnpm run-script jest run",
            "pnpm run-script jest",
            "pnpx jest run",
            "pnpx jest",
        ];
        for command in commands {
            assert_eq!(
                rewrite_command_no_prefixes(command, &[]),
                Some("rtco jest".into()),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_classify_vitest() {
        let commands = vec![
            "npm exec vitest run",
            "npm exec vitest",
            "npm rum vitest run",
            "npm rum vitest",
            "npm run vitest run",
            "npm run vitest",
            "npm run-script vitest run",
            "npm run-script vitest",
            "npm urn vitest run",
            "npm urn vitest",
            "npm vitest run",
            "npm vitest",
            "npm x vitest run",
            "npm x vitest",
            "npx vitest run",
            "npx vitest",
            "pnpm dlx vitest run",
            "pnpm dlx vitest",
            "pnpm exec vitest run",
            "pnpm exec vitest",
            "pnpm run vitest run",
            "pnpm run vitest",
            "pnpm run-script vitest run",
            "pnpm run-script vitest",
            "pnpm vitest run",
            "pnpm vitest",
            "pnpx vitest run",
            "pnpx vitest",
            "vitest run",
            "vitest",
        ];
        for command in commands {
            assert!(
                matches!(
                    classify_command(command),
                    Classification::Supported {
                        rtco_equivalent: "rtco vitest",
                        ..
                    }
                ),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_vitest() {
        let commands = vec![
            "npm exec vitest run",
            "npm exec vitest",
            "npm rum vitest run",
            "npm rum vitest",
            "npm run vitest run",
            "npm run vitest",
            "npm run-script vitest run",
            "npm run-script vitest",
            "npm urn vitest run",
            "npm urn vitest",
            "npm vitest run",
            "npm vitest",
            "npm x vitest run",
            "npm x vitest",
            "npx vitest run",
            "npx vitest",
            "pnpm dlx vitest run",
            "pnpm dlx vitest",
            "pnpm exec vitest run",
            "pnpm exec vitest",
            "pnpm run vitest run",
            "pnpm run vitest",
            "pnpm run-script vitest run",
            "pnpm run-script vitest",
            "pnpm vitest run",
            "pnpm vitest",
            "pnpx vitest run",
            "pnpx vitest",
            "vitest run",
            "vitest",
        ];
        for command in commands {
            assert_eq!(
                rewrite_command_no_prefixes(command, &[]),
                Some("rtco vitest".into()),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_classify_prisma() {
        let commands = vec![
            "npm exec prisma",
            "npm rum prisma",
            "npm run prisma",
            "npm run-script prisma",
            "npm urn prisma",
            "npm x prisma",
            "pnpm dlx prisma",
            "pnpm exec prisma",
            "pnpm run prisma",
            "pnpm run-script prisma",
            "npm prisma",
            "npx prisma",
            "pnpm prisma",
            "pnpx prisma",
            "prisma",
        ];
        for command in commands {
            assert!(
                matches!(
                    classify_command(format!("{command} migrate dev").as_str()),
                    Classification::Supported {
                        rtco_equivalent: "rtco prisma",
                        ..
                    }
                ),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_prisma() {
        let commands = vec![
            "npm exec prisma",
            "npm rum prisma",
            "npm run prisma",
            "npm run-script prisma",
            "npm urn prisma",
            "npm x prisma",
            "pnpm dlx prisma",
            "pnpm exec prisma",
            "pnpm run prisma",
            "pnpm run-script prisma",
            "npm prisma",
            "npx prisma",
            "pnpm prisma",
            "pnpx prisma",
            "prisma",
        ];
        for command in commands {
            assert_eq!(
                rewrite_command_no_prefixes(format!("{command} migrate dev").as_str(), &[]),
                Some("rtco prisma migrate dev".into()),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_prettier() {
        let commands = vec![
            "npm exec prettier",
            "npm rum prettier",
            "npm run prettier",
            "npm run-script prettier",
            "npm urn prettier",
            "npm x prettier",
            "pnpm dlx prettier",
            "pnpm exec prettier",
            "pnpm run prettier",
            "pnpm run-script prettier",
            "npm prettier",
            "npx prettier",
            "pnpm prettier",
            "pnpx prettier",
            "prettier",
        ];
        for command in commands {
            assert_eq!(
                rewrite_command_no_prefixes(format!("{command} --check src/").as_str(), &[]),
                Some("rtco prettier --check src/".into()),
                "Failed for command: {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_pnpm_command() {
        let commands = vec![
            "exec",
            "i",
            "install",
            "list",
            "ls",
            "outdated",
            "run",
            "run-script",
        ];
        for command in commands {
            assert_eq!(
                rewrite_command_no_prefixes(format!("pnpm {command}").as_str(), &[]),
                Some(format!("rtco pnpm {command}")),
                "Failed for command: pnpm {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_npm_bare_subcommand() {
        let commands = vec!["exec", "run", "run-script", "x"];
        for command in commands {
            assert_eq!(
                rewrite_command_no_prefixes(format!("npm {command}").as_str(), &[]),
                Some(format!("rtco npm {command}")),
                "Failed for bare command: npm {}",
                command
            );
        }
    }

    #[test]
    fn test_rewrite_npm_with_args() {
        assert_eq!(
            rewrite_command_no_prefixes("npm run test", &[]),
            Some("rtco npm run test".to_string()),
        );
        assert_eq!(
            rewrite_command_no_prefixes("npm exec vitest", &[]),
            Some("rtco vitest".to_string()),
        );
    }

    #[test]
    fn test_rewrite_npx() {
        assert_eq!(
            rewrite_command_no_prefixes("npx svgo", &[]),
            Some("rtco npx svgo".to_string()),
        );
    }

    // --- Gradle ---

    #[test]
    fn test_classify_gradlew() {
        assert!(matches!(
            classify_command("./gradlew assembleDebug"),
            Classification::Supported {
                rtco_equivalent: "rtco gradlew",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_gradlew_no_dot_slash() {
        assert!(matches!(
            classify_command("gradlew build"),
            Classification::Supported {
                rtco_equivalent: "rtco gradlew",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_gradlew_bat() {
        assert!(matches!(
            classify_command("gradlew.bat clean"),
            Classification::Supported {
                rtco_equivalent: "rtco gradlew",
                ..
            }
        ));
    }

    #[test]
    fn test_classify_gradle() {
        assert!(matches!(
            classify_command("gradle build"),
            Classification::Supported {
                rtco_equivalent: "rtco gradlew",
                ..
            }
        ));
    }

    #[test]
    fn test_rewrite_gradlew() {
        assert_eq!(
            rewrite_command_no_prefixes("./gradlew assembleDebug", &[]),
            Some("rtco gradlew assembleDebug".into())
        );
    }

    #[test]
    fn test_rewrite_gradlew_no_dot_slash() {
        assert_eq!(
            rewrite_command_no_prefixes("gradlew build", &[]),
            Some("rtco gradlew build".into())
        );
    }

    #[test]
    fn test_rewrite_gradlew_bat() {
        assert_eq!(
            rewrite_command_no_prefixes("gradlew.bat clean", &[]),
            Some("rtco gradlew clean".into())
        );
    }

    #[test]
    fn test_rewrite_gradle() {
        assert_eq!(
            rewrite_command_no_prefixes("gradle build", &[]),
            Some("rtco gradlew build".into())
        );
    }

    #[test]
    fn test_rewrite_gradlew_test_savings() {
        assert_eq!(
            classify_command("./gradlew test"),
            Classification::Supported {
                rtco_equivalent: "rtco gradlew",
                category: "Build",
                estimated_savings_pct: 90.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    // --- Compound operator edge cases ---

    #[test]
    fn test_rewrite_compound_or() {
        // `||` fallback: left rewritten, right rewritten
        assert_eq!(
            rewrite_command_no_prefixes("cargo test || cargo build", &[]),
            Some("rtco cargo test || rtco cargo build".into())
        );
    }

    #[test]
    fn test_rewrite_compound_semicolon() {
        assert_eq!(
            rewrite_command_no_prefixes("git status; cargo test", &[]),
            Some("rtco git status; rtco cargo test".into())
        );
    }

    #[test]
    fn test_rewrite_compound_pipe_raw_filter() {
        // Producers stay raw; only a pipeline-safe final stage is rewritten
        // (upstream 523c803 pipeline_final_safe).
        assert_eq!(
            rewrite_command_no_prefixes("cargo test | grep FAILED", &[]),
            Some("cargo test | rtco grep FAILED".into())
        );
    }

    #[test]
    fn test_rewrite_compound_pipe_git_grep() {
        assert_eq!(
            rewrite_command_no_prefixes("git log -10 | grep feat", &[]),
            Some("git log -10 | rtco grep feat".into())
        );
    }

    #[test]
    fn test_rewrite_compound_four_segments() {
        assert_eq!(
            rewrite_command_no_prefixes(
                "cargo fmt --all && cargo clippy && cargo test && git status",
                &[]
            ),
            Some(
                "rtco cargo fmt --all && rtco cargo clippy && rtco cargo test && rtco git status"
                    .into()
            )
        );
    }

    #[test]
    fn test_rewrite_compound_mixed_supported_unsupported() {
        // unsupported segments stay raw
        assert_eq!(
            rewrite_command_no_prefixes("cargo test && htop", &[]),
            Some("rtco cargo test && htop".into())
        );
    }

    #[test]
    fn test_rewrite_compound_all_unsupported_returns_none() {
        // No rewrite at all: returns None
        assert_eq!(rewrite_command_no_prefixes("htop && top", &[]), None);
    }

    // --- sudo / env prefix + rewrite ---

    #[test]
    fn test_rewrite_sudo_docker() {
        assert_eq!(
            rewrite_command_no_prefixes("sudo docker ps", &[]),
            Some("sudo rtco docker ps".into())
        );
    }

    #[test]
    fn test_rewrite_env_var_prefix() {
        assert_eq!(
            rewrite_command_no_prefixes("GIT_SSH_COMMAND=ssh git push origin main", &[]),
            Some("GIT_SSH_COMMAND=ssh rtco git push origin main".into())
        );
    }

    // --- find with native flags ---

    #[test]
    fn test_rewrite_find_with_flags() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*.rs' -type f", &[]),
            Some("rtco find . -name '*.rs' -type f".into())
        );
    }

    // --- #664: rewrite-layer guard for non-compact find invocations ---
    //
    // Default-deny: only rewrite when the invocation fits one of two strict
    // shapes that match RTCO's existing compact-find semantics exactly.
    // See `is_supported_simple_find` in this file for the grammar.

    // Supported shapes (must still rewrite).

    #[test]
    fn rewrite_find_keeps_native_simple_name() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*.rs'", &[]),
            Some("rtco find . -name '*.rs'".into())
        );
    }

    #[test]
    fn rewrite_find_keeps_native_type_and_maxdepth() {
        assert_eq!(
            rewrite_command_no_prefixes("find src -type f -maxdepth 2 -name '*.rs'", &[]),
            Some("rtco find src -type f -maxdepth 2 -name '*.rs'".into())
        );
    }

    #[test]
    fn rewrite_find_keeps_max_flag() {
        assert_eq!(
            rewrite_command_no_prefixes("find '*.rs' src -m 5", &[]),
            Some("rtco find '*.rs' src -m 5".into())
        );
    }

    #[test]
    fn rewrite_find_keeps_iname_alone() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -iname '*.RS'", &[]),
            Some("rtco find . -iname '*.RS'".into())
        );
    }

    #[test]
    fn rewrite_find_keeps_no_explicit_path() {
        // `find -name '*.rs'` with no path — both native and rtco default to cwd.
        assert_eq!(
            rewrite_command_no_prefixes("find -name '*.rs'", &[]),
            Some("rtco find -name '*.rs'".into())
        );
    }

    // Loud-fail set: rtco find errors out, breaking `&&` chains.

    #[test]
    fn rewrite_find_skips_exec() {
        // #664 reproduction case.
        assert_eq!(
            rewrite_command_no_prefixes("find . -type f -exec ls -lh {} \\;", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_not() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*.md' -not -path './node_modules/*'", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_delete() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*.tmp' -delete", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_or_predicate() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*.rs' -o -name '*.md'", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_bang_predicate() {
        assert_eq!(
            rewrite_command_no_prefixes("find . ! -name '*.test.rs'", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_paren_grouping() {
        assert_eq!(
            rewrite_command_no_prefixes("find . \\( -name '*.rs' -o -name '*.md' \\)", &[]),
            None
        );
    }

    // Silent-fail set: rtco find returns wrong results with exit 0.

    #[test]
    fn rewrite_find_skips_mindepth() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -mindepth 2 -name '*.rs'", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_path_predicate() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -path '*/src/*' -name '*.rs'", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_printf_action() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*.rs' -printf '%p\\n'", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_multiple_start_paths() {
        // rtco find silently drops 'tests', returns only matches under 'src'.
        assert_eq!(
            rewrite_command_no_prefixes("find src tests -name '*.rs'", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_extra_bare_arg_after_expression() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -name foo bar", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_bare_path_only() {
        // `find src` natively = "all under src/"; rtco parser would treat
        // 'src' as PATTERN. Decline because no selector predicate present.
        assert_eq!(rewrite_command_no_prefixes("find src", &[]), None);
    }

    #[test]
    fn rewrite_find_skips_bare_dot_only() {
        // Same ambiguity — decline.
        assert_eq!(rewrite_command_no_prefixes("find .", &[]), None);
    }

    #[test]
    fn rewrite_find_skips_duplicate_name_predicates() {
        // Native: implicit AND (impossible match). RTCO: last wins ('*.md' only).
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*.rs' -name '*.md'", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_name_and_iname_combo() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*.rs' -iname '*.MD'", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_duplicate_type() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -type f -type d", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_unsupported_type_value_l() {
        // -type l (symlink) — RTCO only distinguishes "d" vs everything else,
        // so it returns files + symlinks indiscriminately. Decline.
        assert_eq!(rewrite_command_no_prefixes("find . -type l", &[]), None);
    }

    #[test]
    fn rewrite_find_skips_type_with_compound_value() {
        // GNU find allows `-type f,d` (comma-list). RTCO has no equivalent.
        assert_eq!(rewrite_command_no_prefixes("find . -type f,d", &[]), None);
    }

    #[test]
    fn rewrite_find_skips_maxdepth_only() {
        // FindArgs::default() pins file_type="f" → rtco drops dirs while native
        // returns files AND directories. `-maxdepth` alone is not a selector.
        assert_eq!(rewrite_command_no_prefixes("find . -maxdepth 2", &[]), None);
    }

    #[test]
    fn rewrite_find_skips_maxdepth_zero() {
        // Native prints the start path; rtco strips it to empty and skips.
        assert_eq!(
            rewrite_command_no_prefixes("find . -maxdepth 0 -name foo", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_file_start_path() {
        // `find Cargo.toml -type f` — file root gets stripped to empty in rtco;
        // native prints it. Cargo.toml exists at the crate root during tests.
        assert_eq!(
            rewrite_command_no_prefixes("find Cargo.toml -type f", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_missing_start_path() {
        // Native errors non-zero; rtco returns "0 for ..." with success.
        assert_eq!(
            rewrite_command_no_prefixes("find /this/does/not/exist/rtk-test -name '*.rs'", &[]),
            None
        );
    }

    #[test]
    fn rewrite_find_skips_unexpanded_tilde() {
        // Shell hasn't expanded `~` at hook time. Path::new("~").is_dir()
        // is false → decline. Native runs after shell expands → correct via
        // passthrough.
        assert_eq!(
            rewrite_command_no_prefixes("find ~ -name '*.rs'", &[]),
            None
        );
    }

    // Quoting + edge cases.

    #[test]
    fn rewrite_find_quoted_dash_in_pattern_is_not_a_flag() {
        // Quoted glob containing a dash must not be misread as an unknown flag.
        assert_eq!(
            rewrite_command_no_prefixes("find . -name '*-not-a-flag*'", &[]),
            Some("rtco find . -name '*-not-a-flag*'".into())
        );
    }

    #[test]
    fn rewrite_find_dangling_flag_value_skips() {
        assert_eq!(rewrite_command_no_prefixes("find . -name", &[]), None);
    }

    #[test]
    fn rewrite_find_skips_maxdepth_non_integer() {
        assert_eq!(
            rewrite_command_no_prefixes("find . -maxdepth abc -name '*.rs'", &[]),
            None
        );
    }

    // Compound command — other segments must still rewrite even when find
    // segment is declined.

    #[test]
    fn rewrite_find_unsupported_in_compound_leaves_segment_raw_but_rewrites_others() {
        let out =
            rewrite_command_no_prefixes("find . -type f -exec ls -lh {} \\; && git status", &[]);
        assert!(
            out.is_some(),
            "compound rewrite should still produce output if any segment changed"
        );
        let s = out.unwrap();
        assert!(
            s.contains("find . -type f -exec ls -lh {} \\;"),
            "find segment must be raw; got: {s}"
        );
        assert!(
            s.contains("rtco git status"),
            "git status segment must be rewritten; got: {s}"
        );
    }

    #[test]
    fn test_all_rules_are_complete() {
        for rule in RULES {
            assert!(
                !rule.pattern.is_empty(),
                "Rule '{}' has empty pattern",
                rule.rtco_cmd
            );
            assert!(!rule.rtco_cmd.is_empty(), "Rule with empty rtco_cmd found");
            assert!(
                rule.rtco_cmd.starts_with("rtco "),
                "rtco_cmd '{}' must start with 'rtco '",
                rule.rtco_cmd
            );
            assert!(
                !rule.rewrite_prefixes.is_empty(),
                "Rule '{}' has no rewrite_prefixes",
                rule.rtco_cmd
            );
        }
    }

    // --- exclude_commands (#243) ---

    #[test]
    fn test_rewrite_excludes_curl() {
        let excluded = vec!["curl".to_string()];
        assert_eq!(
            rewrite_command_no_prefixes("curl https://api.example.com/health", &excluded),
            None
        );
    }

    #[test]
    fn test_rewrite_exclude_does_not_affect_other_commands() {
        let excluded = vec!["curl".to_string()];
        assert_eq!(
            rewrite_command_no_prefixes("git status", &excluded),
            Some("rtco git status".into())
        );
    }

    #[test]
    fn test_rewrite_empty_excludes_rewrites_curl() {
        let excluded: Vec<String> = vec![];
        assert!(rewrite_command_no_prefixes("curl https://api.example.com", &excluded).is_some());
    }

    #[test]
    fn test_rewrite_compound_partial_exclude() {
        // curl excluded but git still rewrites
        let excluded = vec!["curl".to_string()];
        assert_eq!(
            rewrite_command_no_prefixes("git status && curl https://api.example.com", &excluded),
            Some("rtco git status && curl https://api.example.com".into())
        );
    }

    #[test]
    fn test_exclude_env_prefixed_command() {
        let excluded = vec!["psql".to_string()];
        assert_eq!(
            rewrite_command_no_prefixes("PGPASSWORD=postgres psql -h localhost", &excluded),
            None
        );
    }

    #[test]
    fn test_exclude_subcommand_pattern() {
        let excluded = vec!["git push".to_string()];
        assert_eq!(
            rewrite_command_no_prefixes("git push origin main", &excluded),
            None
        );
    }

    #[test]
    fn test_exclude_regex_pattern() {
        let excluded = vec!["^curl".to_string()];
        assert_eq!(
            rewrite_command_no_prefixes("curl http://example.com", &excluded),
            None
        );
    }

    #[test]
    fn test_exclude_invalid_regex_fallback() {
        let excluded = vec!["curl[".to_string()];
        assert!(rewrite_command_no_prefixes("curl http://example.com", &excluded).is_some());
    }

    #[test]
    fn test_exclude_does_not_substring_match() {
        let excluded = vec!["go".to_string()];
        assert!(rewrite_command_no_prefixes("golangci-lint run ./...", &excluded).is_some());
    }

    #[test]
    fn test_exclude_does_not_match_hyphenated_command() {
        let excluded = vec!["golangci".to_string()];
        assert!(rewrite_command_no_prefixes("golangci-lint run ./...", &excluded).is_some());
    }

    #[test]
    fn test_exclude_empty_pattern_ignored() {
        let excluded = vec!["".to_string()];
        assert!(rewrite_command_no_prefixes("git status", &excluded).is_some());
    }

    #[test]
    fn test_exclude_bare_anchor_ignored() {
        let excluded = vec!["^".to_string()];
        assert!(rewrite_command_no_prefixes("git status", &excluded).is_some());
    }

    #[test]
    fn test_all_patterns_are_valid_regex() {
        use regex::Regex;
        for (i, rule) in RULES.iter().enumerate() {
            assert!(
                Regex::new(rule.pattern).is_ok(),
                "RULES[{i}] ({}) has invalid pattern '{}'",
                rule.rtco_cmd,
                rule.pattern
            );
        }
    }

    // --- #196: gh --json/--jq/--template passthrough ---

    #[test]
    fn test_rewrite_gh_json_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("gh pr list --json number,title", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_gh_jq_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("gh pr list --json number --jq '.[].number'", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_gh_template_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("gh pr view 42 --template '{{.title}}'", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_gh_api_json_skipped() {
        assert_eq!(
            rewrite_command_no_prefixes("gh api repos/owner/repo --jq '.name'", &[]),
            None
        );
    }

    #[test]
    fn test_rewrite_gh_without_json_still_works() {
        assert_eq!(
            rewrite_command_no_prefixes("gh pr list", &[]),
            Some("rtco gh pr list".into())
        );
    }

    // --- #508: RTK_DISABLED detection helpers ---

    #[test]
    fn test_cmd_has_rtk_disabled_prefix() {
        assert!(cmd_has_rtco_disabled_prefix("RTK_DISABLED=1 git status"));
        assert!(cmd_has_rtco_disabled_prefix(
            "FOO=1 RTK_DISABLED=1 cargo test"
        ));
        assert!(cmd_has_rtco_disabled_prefix(
            "RTK_DISABLED=true git log --oneline"
        ));
        assert!(!cmd_has_rtco_disabled_prefix("git status"));
        assert!(!cmd_has_rtco_disabled_prefix("rtco git status"));
        assert!(!cmd_has_rtco_disabled_prefix("SOME_VAR=1 git status"));
    }

    #[test]
    fn test_strip_disabled_prefix() {
        assert_eq!(
            strip_disabled_prefix("RTK_DISABLED=1 git status"),
            ("RTK_DISABLED=1 ", "git status")
        );
        assert_eq!(
            strip_disabled_prefix("FOO=1 RTK_DISABLED=1 cargo test"),
            ("FOO=1 RTK_DISABLED=1 ", "cargo test")
        );
        assert_eq!(strip_disabled_prefix("git status"), ("", "git status"));
    }

    // --- #485: absolute path normalization ---

    #[test]
    fn test_classify_absolute_path_grep() {
        assert_eq!(
            classify_command("/usr/bin/grep -rni pattern"),
            Classification::Supported {
                rtco_equivalent: "rtco grep",
                category: "Files",
                estimated_savings_pct: 75.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_absolute_path_ls() {
        assert_eq!(
            classify_command("/bin/ls -la"),
            Classification::Supported {
                rtco_equivalent: "rtco ls",
                category: "Files",
                estimated_savings_pct: 65.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_absolute_path_git() {
        assert_eq!(
            classify_command("/usr/local/bin/git status"),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_absolute_path_no_args() {
        // /usr/bin/find alone → still classified
        assert_eq!(
            classify_command("/usr/bin/find ."),
            Classification::Supported {
                rtco_equivalent: "rtco find",
                category: "Files",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_strip_absolute_path_helper() {
        assert_eq!(strip_absolute_path("/usr/bin/grep -rn foo"), "grep -rn foo");
        assert_eq!(strip_absolute_path("/bin/ls -la"), "ls -la");
        assert_eq!(strip_absolute_path("grep -rn foo"), "grep -rn foo");
        assert_eq!(strip_absolute_path("/usr/local/bin/git"), "git");
    }

    // --- #163: git global options ---

    #[test]
    fn test_classify_git_with_dash_c_path() {
        assert_eq!(
            classify_command("git -C /tmp status"),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_git_no_pager_log() {
        assert_eq!(
            classify_command("git --no-pager log -5"),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_git_git_dir() {
        assert_eq!(
            classify_command("git --git-dir /tmp/.git status"),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_rewrite_git_dash_c() {
        assert_eq!(
            rewrite_command_no_prefixes("git -C /tmp status", &[]),
            Some("rtco git -C /tmp status".to_string())
        );
    }

    #[test]
    fn test_rewrite_git_no_pager() {
        assert_eq!(
            rewrite_command_no_prefixes("git --no-pager log -5", &[]),
            Some("rtco git --no-pager log -5".to_string())
        );
    }

    #[test]
    fn test_strip_git_global_opts_helper() {
        assert_eq!(strip_git_global_opts("git -C /tmp status"), "git status");
        assert_eq!(strip_git_global_opts("git --no-pager log"), "git log");
        assert_eq!(strip_git_global_opts("git status"), "git status");
        assert_eq!(strip_git_global_opts("cargo test"), "cargo test");
    }

    #[test]
    fn test_strip_golangci_global_opts_helper() {
        assert_eq!(
            strip_golangci_global_opts("golangci-lint -v run ./..."),
            "golangci-lint run ./..."
        );
        assert_eq!(
            strip_golangci_global_opts("golangci-lint --color never run ./..."),
            "golangci-lint run ./..."
        );
        assert_eq!(
            strip_golangci_global_opts("golangci-lint --color=never run ./..."),
            "golangci-lint run ./..."
        );
        assert_eq!(
            strip_golangci_global_opts("golangci-lint --config=foo.yml run ./..."),
            "golangci-lint run ./..."
        );
        assert_eq!(
            strip_golangci_global_opts("golangci-lint version"),
            "golangci-lint version"
        );
        assert_eq!(strip_golangci_global_opts("cargo test"), "cargo test");
    }

    // --- #wc: wc filter was silently ignored by the hook ---

    #[test]
    fn test_classify_wc_supported() {
        // BUG: "wc " was in IGNORED_PREFIXES despite wc_cmd.rs having a full filter.
        // This test documents the bug: it must FAIL before the fix and PASS after.
        assert_eq!(
            classify_command("wc -l src/main.rs"),
            Classification::Supported {
                rtco_equivalent: "rtco wc",
                category: "Files",
                estimated_savings_pct: 60.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_classify_wc_multi_file() {
        assert_eq!(
            classify_command("wc src/*.rs"),
            Classification::Supported {
                rtco_equivalent: "rtco wc",
                category: "Files",
                estimated_savings_pct: 60.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_rewrite_wc() {
        assert_eq!(
            rewrite_command_no_prefixes("wc -l src/main.rs", &[]),
            Some("rtco wc -l src/main.rs".into())
        );
    }

    #[test]
    fn test_rewrite_wc_multi_file() {
        assert_eq!(
            rewrite_command_no_prefixes("wc src/*.rs", &[]),
            Some("rtco wc src/*.rs".into())
        );
    }

    #[test]
    fn test_classify_command_substitution_passthrough() {
        assert_eq!(
            classify_command("git log $(git rev-parse HEAD~1)"),
            Classification::Supported {
                rtco_equivalent: "rtco git",
                category: "Git",
                estimated_savings_pct: 70.0,
                status: RtcoStatus::Existing,
            }
        );
    }

    #[test]
    fn test_rewrite_command_substitution_passthrough() {
        assert_eq!(
            rewrite_command_no_prefixes("git log $(git rev-parse HEAD~1)", &[]),
            Some("rtco git log $(git rev-parse HEAD~1)".into())
        );
    }

    #[test]
    fn test_split_command_substitution_no_split() {
        assert_eq!(
            split_command_chain("git log $(git rev-parse HEAD~1)"),
            vec!["git log $(git rev-parse HEAD~1)"]
        );
    }

    #[test]
    fn test_shell_prefix_noglob() {
        assert_eq!(
            rewrite_command_no_prefixes("noglob git status", &[]),
            Some("noglob rtco git status".into())
        );
    }

    #[test]
    fn test_shell_prefix_command() {
        assert_eq!(
            rewrite_command_no_prefixes("command git status", &[]),
            Some("command rtco git status".into())
        );
    }

    #[test]
    fn test_shell_prefix_builtin_exec_nocorrect() {
        assert_eq!(
            rewrite_command_no_prefixes("builtin git status", &[]),
            Some("builtin rtco git status".into())
        );
        assert_eq!(
            rewrite_command_no_prefixes("exec git status", &[]),
            Some("exec rtco git status".into())
        );
        assert_eq!(
            rewrite_command_no_prefixes("nocorrect git status", &[]),
            Some("nocorrect rtco git status".into())
        );
    }

    #[test]
    fn test_shell_prefix_unknown_inner() {
        assert_eq!(
            rewrite_command_no_prefixes("noglob unknown_cmd --flag", &[]),
            None
        );
    }

    // --- transparent_prefixes tests ---

    #[test]
    fn test_transparent_prefix_strips_and_reprepends() {
        let prefixes = vec!["shadowenv exec --".to_string()];
        assert_eq!(
            super::rewrite_command("shadowenv exec -- git status", &[], &prefixes),
            Some("shadowenv exec -- rtco git status".into())
        );
    }

    #[test]
    fn test_transparent_prefix_with_test_runner() {
        let prefixes = vec!["shadowenv exec --".to_string()];
        assert_eq!(
            super::rewrite_command("shadowenv exec -- cargo test", &[], &prefixes),
            Some("shadowenv exec -- rtco cargo test".into())
        );
    }

    #[test]
    fn test_transparent_prefix_unknown_inner_returns_none() {
        let prefixes = vec!["shadowenv exec --".to_string()];
        assert_eq!(
            super::rewrite_command("shadowenv exec -- htop", &[], &prefixes),
            None
        );
    }

    #[test]
    fn test_transparent_prefix_not_matched_is_passthrough() {
        // Without the prefix configured, the wrapper breaks routing.
        assert_eq!(
            super::rewrite_command("shadowenv exec -- git status", &[], &[]),
            None
        );
    }

    #[test]
    fn test_transparent_prefix_composed_with_builtin() {
        // `noglob shadowenv exec -- git status` — builtin layer strips noglob,
        // user layer strips shadowenv exec --, inner `git status` routes.
        let prefixes = vec!["shadowenv exec --".to_string()];
        assert_eq!(
            super::rewrite_command("noglob shadowenv exec -- git status", &[], &prefixes),
            Some("noglob shadowenv exec -- rtco git status".into())
        );
    }

    #[test]
    fn test_transparent_prefix_composed_with_env_prefix() {
        let prefixes = vec!["bundle exec".to_string()];
        assert_eq!(
            super::rewrite_command("RAILS_ENV=test bundle exec git status", &[], &prefixes),
            Some("RAILS_ENV=test bundle exec rtco git status".into())
        );
    }

    #[test]
    fn test_env_prefix_composed_with_builtin() {
        assert_eq!(
            rewrite_command_no_prefixes("sudo noglob git status", &[]),
            Some("sudo noglob rtco git status".into())
        );
    }

    #[test]
    fn test_transparent_prefix_multiple_configured() {
        let prefixes = vec!["shadowenv exec --".to_string(), "direnv exec .".to_string()];
        assert_eq!(
            super::rewrite_command("direnv exec . git status", &[], &prefixes),
            Some("direnv exec . rtco git status".into())
        );
    }

    #[test]
    fn test_transparent_prefixes_normalize_once() {
        let prefixes = vec![
            "  docker exec mycontainer  ".to_string(),
            "".to_string(),
            "docker".to_string(),
            "docker exec mycontainer".to_string(),
        ];
        assert_eq!(
            normalize_transparent_prefixes(&prefixes),
            vec!["docker exec mycontainer".to_string(), "docker".to_string()]
        );
    }

    #[test]
    fn test_transparent_prefix_overlapping_entries_use_longest_match() {
        let prefixes = vec!["docker".to_string(), "docker exec app".to_string()];
        assert_eq!(
            super::rewrite_command("docker exec app git status", &[], &prefixes),
            Some("docker exec app rtco git status".into())
        );
    }

    #[test]
    fn test_transparent_prefix_whole_word_matching() {
        // A prefix `"foo"` must NOT match `"foobar git status"`.
        let prefixes = vec!["foo".to_string()];
        assert_eq!(
            super::rewrite_command("foobar git status", &[], &prefixes),
            None
        );
    }

    #[test]
    fn test_transparent_prefix_empty_rest_returns_none() {
        let prefixes = vec!["shadowenv exec --".to_string()];
        assert_eq!(
            super::rewrite_command("shadowenv exec --", &[], &prefixes),
            None
        );
    }

    #[test]
    fn test_transparent_prefix_empty_entry_is_skipped() {
        // A blank entry in the config should not cause spurious matches or panics.
        let prefixes = vec!["".to_string(), "   ".to_string()];
        assert_eq!(
            super::rewrite_command("git status", &[], &prefixes),
            Some("rtco git status".into())
        );
    }

    #[test]
    fn test_transparent_prefix_inside_compound() {
        // Each segment of `&&` / `;` should independently get prefix-stripped.
        let prefixes = vec!["shadowenv exec --".to_string()];
        assert_eq!(
            super::rewrite_command(
                "shadowenv exec -- git status && shadowenv exec -- cargo test",
                &[],
                &prefixes
            ),
            Some("shadowenv exec -- rtco git status && shadowenv exec -- rtco cargo test".into())
        );
    }

    #[test]
    fn test_transparent_prefix_respects_excluded() {
        // An excluded inner command should still produce no rewrite even behind
        // a transparent prefix.
        let prefixes = vec!["shadowenv exec --".to_string()];
        let excluded = vec!["git".to_string()];
        assert_eq!(
            super::rewrite_command("shadowenv exec -- git status", &excluded, &prefixes),
            None
        );
    }

    #[test]
    fn test_transparent_prefix_recursion_bounded() {
        // A prefix that could recurse forever (e.g. one that maps to itself)
        // must terminate once MAX_PREFIX_DEPTH is reached.
        let prefixes = vec!["wrap".to_string()];
        let mut cmd = String::new();
        for _ in 0..(MAX_PREFIX_DEPTH + 2) {
            cmd.push_str("wrap ");
        }
        cmd.push_str("git status");
        // Doesn't matter exactly what it returns — just that it doesn't stack-
        // overflow or loop forever. Exercise the code path.
        let _ = super::rewrite_command(&cmd, &[], &prefixes);
    }

    #[test]
    fn test_python3_m_pytest() {
        assert_eq!(
            rewrite_command_no_prefixes("python3 -m pytest tests/", &[]),
            Some("rtco pytest tests/".into())
        );
    }

    #[test]
    fn test_pip_show() {
        assert_eq!(
            rewrite_command_no_prefixes("pip show flask", &[]),
            Some("rtco pip show flask".into())
        );
    }

    #[test]
    fn test_gt_graphite() {
        assert_eq!(
            rewrite_command_no_prefixes("gt log", &[]),
            Some("rtco gt log".into())
        );
    }

    #[test]
    fn test_command_no_longer_ignored() {
        assert_ne!(
            classify_command("command git status"),
            Classification::Ignored
        );
    }

    // --- Pipe + operator rewrite ---

    #[test]
    fn test_rewrite_pipe_then_and() {
        assert_eq!(
            rewrite_command_no_prefixes("git log | head -5 && git stash", &[]),
            Some("git log | head -5 && rtco git stash".into())
        );
    }

    #[test]
    fn test_rewrite_pipe_then_semicolon() {
        assert_eq!(
            rewrite_command_no_prefixes("cargo test | head; git status", &[]),
            Some("cargo test | head; rtco git status".into())
        );
    }

    #[test]
    fn test_rewrite_pipe_then_or() {
        assert_eq!(
            rewrite_command_no_prefixes("cargo test | grep FAIL || git stash", &[]),
            Some("cargo test | rtco grep FAIL || rtco git stash".into())
        );
    }

    #[test]
    fn test_rewrite_env_pipe_then_and() {
        assert_eq!(
            rewrite_command_no_prefixes(
                "RUST_BACKTRACE=1 cargo test 2>&1 | grep FAILED && git stash",
                &[]
            ),
            Some("RUST_BACKTRACE=1 cargo test 2>&1 | rtco grep FAILED && rtco git stash".into())
        );
    }

    #[test]
    fn test_rewrite_and_then_pipe() {
        // The `&&` clause is rewritten normally; only the pipeline producer
        // stays raw, and the safe final grep stage is rewritten.
        assert_eq!(
            rewrite_command_no_prefixes("git status && cargo test | grep FAIL", &[]),
            Some("rtco git status && cargo test | rtco grep FAIL".into())
        );
    }

    #[test]
    fn test_rewrite_multi_pipe_then_and() {
        assert_eq!(
            rewrite_command_no_prefixes("git log | head | tail && git status", &[]),
            Some("git log | head | tail && rtco git status".into())
        );
    }

    #[test]
    fn test_pipeline_final_safe_rule_set() {
        let safe_rules: Vec<_> = RULES
            .iter()
            .filter(|rule| rule.pipeline_final_safe)
            .map(|rule| rule.rtco_cmd)
            .collect();

        assert_eq!(safe_rules, vec!["rtco rg", "rtco grep"]);
    }

    #[test]
    fn test_pipeline_final_search_pattern_file_is_unsafe() {
        for command in [
            "grep -f patterns.txt input.txt",
            "grep -rfpatterns.txt input",
            "grep --file patterns.txt input.txt",
            "grep --file=patterns.txt input.txt",
            "rg -f patterns.txt input.txt",
            "rg --file=patterns.txt input.txt",
        ] {
            assert!(search_uses_pattern_file(command), "{command}");
        }

        assert!(!search_uses_pattern_file("grep -- -f"));
        assert!(!search_uses_pattern_file("grep -F pattern"));
    }

    #[test]
    fn test_pipeline_final_pattern_file_passes_through() {
        // grep reading patterns from a file is not pipeline-final-safe: the
        // rewritten `rtco grep` would not see the `-f` file path the same way.
        assert_eq!(
            rewrite_command_no_prefixes("git log | grep -f patterns.txt", &[]),
            None
        );
    }

    // --- line-continuation handling (issue #1564) -------------------

    #[test]
    fn test_rewrite_leading_backslash_newline() {
        // The exact reproduction from #1564: a leading `\<NL>` made
        // the matcher see `\` as the command and bail out.
        assert_eq!(
            rewrite_command_no_prefixes("\\\ngit diff HEAD~1", &[]),
            Some("rtco git diff HEAD~1".into())
        );
    }

    #[test]
    fn test_rewrite_leading_backslash_crlf() {
        // CRLF line ending — same shape, Windows shells / Git Bash.
        assert_eq!(
            rewrite_command_no_prefixes("\\\r\ngit diff HEAD~1", &[]),
            Some("rtco git diff HEAD~1".into())
        );
    }

    #[test]
    fn test_rewrite_internal_backslash_newline() {
        // Embedded line continuation between subcommand and args:
        // `git diff \<NL>HEAD~1` is exactly equivalent to
        // `git diff HEAD~1` per bash semantics.
        assert_eq!(
            rewrite_command_no_prefixes("git diff \\\nHEAD~1", &[]),
            Some("rtco git diff HEAD~1".into())
        );
    }

    #[test]
    fn test_rewrite_backslash_newline_with_indent() {
        // Continuation followed by indentation — also collapsed.
        assert_eq!(
            rewrite_command_no_prefixes("git \\\n    diff HEAD~1", &[]),
            Some("rtco git diff HEAD~1".into())
        );
    }

    #[test]
    fn test_rewrite_no_line_continuation_unchanged() {
        // Sanity check: a command without any `\<NL>` should match
        // unchanged. This pins that the normalization step does not
        // regress the no-op fast path.
        assert_eq!(
            rewrite_command_no_prefixes("git diff HEAD~1", &[]),
            Some("rtco git diff HEAD~1".into())
        );
    }

    #[test]
    fn test_collapse_line_continuations_no_op() {
        // Helper-level: no continuations → returns Borrowed (no
        // allocation). We can only spot-check the equality here, but
        // the `Cow::Borrowed` variant is implied by `replace_all`
        // when no replacement occurs.
        assert_eq!(
            collapse_line_continuations("git diff HEAD~1"),
            std::borrow::Cow::<str>::Borrowed("git diff HEAD~1"),
        );
    }

    // --- multi-line block rewriting (port from upstream rtk #1243, #3319) ---
    mod multiline_blocks {
        use super::rewrite_command_no_prefixes;

        #[test]
        fn test_rewrites_each_line() {
            assert_eq!(
                rewrite_command_no_prefixes("git status\ngit log --oneline -3", &[]),
                Some("rtco git status\nrtco git log --oneline -3".into())
            );
        }

        #[test]
        fn test_preserves_blank_lines_comments_and_indentation() {
            assert_eq!(
                rewrite_command_no_prefixes("git status\n\n# check history\n  git log -3", &[]),
                Some("rtco git status\n\n# check history\n  rtco git log -3".into())
            );
        }

        #[test]
        fn test_compound_line_inside_block() {
            assert_eq!(
                rewrite_command_no_prefixes("cd /tmp && git status\ngrep -rn foo src", &[]),
                Some("cd /tmp && rtco git status\nrtco grep -rn foo src".into())
            );
        }

        #[test]
        fn test_crlf_separators_preserved() {
            assert_eq!(
                rewrite_command_no_prefixes("git status\r\ngit log -3", &[]),
                Some("rtco git status\r\nrtco git log -3".into())
            );
        }

        #[test]
        fn test_newline_inside_quotes_rewrites_as_one_command() {
            // The quoted body is never treated as a command line of its own;
            // the whole thing is one logical command and gets one prefix.
            assert_eq!(
                rewrite_command_no_prefixes("git commit -m \"subject\ngit status in body\"", &[]),
                Some("rtco git commit -m \"subject\ngit status in body\"".into())
            );
            assert_eq!(
                rewrite_command_no_prefixes("git commit -m 'multi\nline\nmessage'", &[]),
                Some("rtco git commit -m 'multi\nline\nmessage'".into())
            );
        }

        #[test]
        fn test_unbalanced_swallowed_newline_passes_through() {
            assert_eq!(
                rewrite_command_no_prefixes("git commit -m \"subject\ngit status", &[]),
                None
            );
        }

        #[test]
        fn test_comment_apostrophe_swallowing_newline_passes_through() {
            // The lexer has no comment state: the apostrophe in `don't` opens
            // a quote that swallows the newline and hides the next line. The
            // block must pass through so native permission handling sees the
            // original command — never a partially rewritten one.
            assert_eq!(
                rewrite_command_no_prefixes("git status # don't\nrm -rf /tmp/x", &[]),
                None
            );
        }

        #[test]
        fn test_comment_apostrophe_hidden_in_later_segment_passes_through() {
            // Same hazard when a clean split point precedes the contaminated
            // line: the swallowed-newline check is global, not per-segment.
            assert_eq!(
                rewrite_command_no_prefixes("git log -3\ngit status # don't\nrm -rf /tmp/x", &[]),
                None
            );
        }

        #[test]
        fn test_comment_with_balanced_quotes_still_rewrites() {
            // Both apostrophes close before the newline, so the split is safe
            // and the trailing comment rides along untouched.
            assert_eq!(
                rewrite_command_no_prefixes(
                    "git status # isn't it what's expected\ngit log -3",
                    &[]
                ),
                Some("rtco git status # isn't it what's expected\nrtco git log -3".into())
            );
        }

        #[test]
        fn test_arithmetic_spanning_lines_passes_through() {
            // `(( x = ls ))` is arithmetic evaluation; injecting `rtco` before
            // `ls` would splice a command into arithmetic context.
            assert_eq!(rewrite_command_no_prefixes("(( x =\nls ))", &[]), None);
        }

        #[test]
        fn test_array_assignment_spanning_lines_passes_through() {
            // The inner line is an array element, not a command; rewriting it
            // would mutate the array's contents.
            assert_eq!(
                rewrite_command_no_prefixes("arr=(one\ngit status\ntwo)", &[]),
                None
            );
        }

        #[test]
        fn test_function_definition_spanning_lines_passes_through() {
            assert_eq!(
                rewrite_command_no_prefixes("foo() {\n  git status\n}", &[]),
                None
            );
        }

        #[test]
        fn test_continuation_operator_behind_comment_passes_through() {
            // Bash continues the pipeline across the newline even though the
            // line ends in comment text; the next line is a pipeline stage,
            // not an independent command.
            assert_eq!(
                rewrite_command_no_prefixes("git log | # keep pipeline\ngrep -f patterns.txt", &[]),
                None
            );
            assert_eq!(
                rewrite_command_no_prefixes("git status && # continue\ngit log -3", &[]),
                None
            );
        }

        #[test]
        fn test_ansi_c_escaped_quote_passes_through() {
            // Inside $'...' bash treats \' as a literal quote that does not
            // close the string, so the second line is string content — the
            // lexer can't see that, so the block forgoes the rewrite.
            assert_eq!(
                rewrite_command_no_prefixes("x=$'foo\\'\ngit status\n'", &[]),
                None
            );
        }

        #[test]
        fn test_ansi_c_without_escaped_quote_still_rewrites() {
            assert_eq!(
                rewrite_command_no_prefixes("echo $'a\\tb'\ngit status", &[]),
                Some("echo $'a\\tb'\nrtco git status".into())
            );
        }

        #[test]
        fn test_balanced_grouping_within_a_line_still_rewrites() {
            // `${HOME}` braces (quoted or not) must not trip the
            // unbalanced-grouping bail.
            assert_eq!(
                rewrite_command_no_prefixes("echo ${HOME}\ngit status", &[]),
                Some("echo ${HOME}\nrtco git status".into())
            );
            assert_eq!(
                rewrite_command_no_prefixes("echo \"${HOME}\"\ngit status", &[]),
                Some("echo \"${HOME}\"\nrtco git status".into())
            );
        }

        #[test]
        fn test_no_rewritable_line_passes_through() {
            assert_eq!(rewrite_command_no_prefixes("echo one\necho two", &[]), None);
        }
    }

    // --- ffs / hashline rewrite rules -------------------------------

    #[test]
    fn test_rewrite_ffs_find() {
        assert_eq!(
            rewrite_command_no_prefixes("ffs find sbt_cmd", &[]),
            Some("rtco ffs find sbt_cmd".into())
        );
    }

    #[test]
    fn test_rewrite_ffs_grep() {
        assert_eq!(
            rewrite_command_no_prefixes("ffs grep LazyLock", &[]),
            Some("rtco ffs grep LazyLock".into())
        );
    }

    #[test]
    fn test_rewrite_hashline_read() {
        assert_eq!(
            rewrite_command_no_prefixes("hashline read src/main.rs", &[]),
            Some("rtco hashline read src/main.rs".into())
        );
    }

    #[test]
    fn test_rewrite_hashline_patch() {
        assert_eq!(
            rewrite_command_no_prefixes("hashline patch src/main.rs 'SWAP 1:ab:'", &[]),
            Some("rtco hashline patch src/main.rs 'SWAP 1:ab:'".into())
        );
    }

    #[test]
    fn test_rewrite_ffs_already_rtco_passthrough() {
        // Already-rtco commands return as-is (Some, unchanged).
        assert_eq!(
            rewrite_command_no_prefixes("rtco ffs find foo", &[]),
            Some("rtco ffs find foo".into())
        );
    }

    #[test]
    fn test_rewrite_hashline_already_rtco_passthrough() {
        assert_eq!(
            rewrite_command_no_prefixes("rtco hashline read f", &[]),
            Some("rtco hashline read f".into())
        );
    }

    #[test]
    fn test_pipeline_final_ffs_not_safe() {
        // ffs is not a grep-pipeline filter: `git log | ffs grep foo` has no
        // rewritable stage (producer stays raw, final stage not pipeline-safe),
        // so the whole pipeline passes through unchanged (None = no rewrite).
        assert_eq!(
            rewrite_command_no_prefixes("git log | ffs grep foo", &[]),
            None
        );
    }

    // --- line-continuation handling (issue #1564) -------------------
}
