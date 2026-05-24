//! Raw output recovery -- saves unfiltered output to disk on command failure.

use super::constants::RTK_DATA_DIR;
use crate::core::config::Config;
use std::path::PathBuf;

/// Minimum output size to tee (smaller outputs don't need recovery)
const MIN_TEE_SIZE: usize = 500;

/// Default max files to keep in tee directory
const DEFAULT_MAX_FILES: usize = 20;

/// Default max file size (1MB)
const DEFAULT_MAX_FILE_SIZE: usize = 1_048_576;

/// Sanitize a command slug for use in filenames.
/// Replaces non-alphanumeric chars (except underscore/hyphen) with underscore,
/// truncates at 40 chars.
fn sanitize_slug(slug: &str) -> String {
    let sanitized: String = slug
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.len() > 40 {
        sanitized[..40].to_string()
    } else {
        sanitized
    }
}

/// Get the tee directory, respecting config and env overrides.
fn get_tee_dir(config: &Config) -> Option<PathBuf> {
    // Env var override
    if let Ok(dir) = std::env::var("RTCO_TEE_DIR") {
        return Some(PathBuf::from(dir));
    }

    // Config override
    if let Some(ref dir) = config.tee.directory {
        return Some(dir.clone());
    }

    // Default: ~/.local/share/rtk/tee/
    dirs::data_local_dir().map(|d| d.join(RTK_DATA_DIR).join("tee"))
}

/// Rotate old tee files: keep only the last `max_files`, delete oldest.
fn cleanup_old_files(dir: &std::path::Path, max_files: usize) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
        .collect();

    if entries.len() <= max_files {
        return;
    }

    // Sort by filename (which starts with epoch timestamp = chronological)
    entries.sort_by_key(|e| e.file_name());

    let to_remove = entries.len() - max_files;
    for entry in entries.iter().take(to_remove) {
        let _ = std::fs::remove_file(entry.path());
    }
}

/// Check if tee should be skipped based on config, mode, exit code, and size.
/// Returns None if should skip, Some(tee_dir) if should proceed.
fn should_tee(
    config: &TeeConfig,
    raw_len: usize,
    exit_code: i32,
    tee_dir: Option<PathBuf>,
) -> Option<PathBuf> {
    if !config.enabled {
        return None;
    }

    match config.mode {
        TeeMode::Never => return None,
        TeeMode::Failures => {
            if exit_code == 0 {
                return None;
            }
        }
        TeeMode::Always => {}
    }

    if raw_len < MIN_TEE_SIZE {
        return None;
    }

    tee_dir
}

/// Write raw output to a tee file in the given directory.
/// Returns file path on success.
fn write_tee_file(
    raw: &str,
    command_slug: &str,
    tee_dir: &std::path::Path,
    max_file_size: usize,
    max_files: usize,
) -> Option<PathBuf> {
    std::fs::create_dir_all(tee_dir).ok()?;

    let slug = sanitize_slug(command_slug);
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let filename = format!("{}_{}.log", epoch, slug);
    let filepath = tee_dir.join(filename);

    // Truncate at max_file_size (find a safe UTF-8 char boundary)
    let content = if raw.len() > max_file_size {
        let boundary = raw
            .char_indices()
            .take_while(|(i, _)| *i < max_file_size)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!(
            "{}\n\n--- truncated at {} bytes ---",
            &raw[..boundary],
            max_file_size
        )
    } else {
        raw.to_string()
    };

    std::fs::write(&filepath, content).ok()?;

    // Rotate old files
    cleanup_old_files(tee_dir, max_files);

    Some(filepath)
}

/// Write raw output to tee file if conditions are met.
/// Returns file path on success, None if skipped/failed.
pub fn tee_raw(raw: &str, command_slug: &str, exit_code: i32) -> Option<PathBuf> {
    // Check RTCO_TEE=0 env override (disable)
    if std::env::var("RTCO_TEE").ok().as_deref() == Some("0") {
        return None;
    }

    let config = Config::load().ok()?;
    let tee_dir = get_tee_dir(&config)?;

    let tee_dir = should_tee(&config.tee, raw.len(), exit_code, Some(tee_dir))?;

    write_tee_file(
        raw,
        command_slug,
        &tee_dir,
        config.tee.max_file_size,
        config.tee.max_files,
    )
}

fn display_path(path: &std::path::Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(relative) = path.strip_prefix(&home) {
            return format!("~/{}", relative.display());
        }
    }
    path.display().to_string()
}

fn format_hint(path: &std::path::Path) -> String {
    // Surface the recovery action inline so an AI consumer knows exactly which
    // command to run if it needs the full output. Without this, models often
    // ignore the path entirely and proceed with partial context.
    format!("[full output saved — run: cat {}]", display_path(path))
}

/// Convenience: tee + format hint in one call.
/// Returns hint string if file was written, None if skipped.
pub fn tee_and_hint(raw: &str, command_slug: &str, exit_code: i32) -> Option<String> {
    let path = tee_raw(raw, command_slug, exit_code)?;
    Some(format_hint(&path))
}

fn force_tee_path(content: &str, command_slug: &str) -> Option<PathBuf> {
    if std::env::var("RTCO_TEE").ok().as_deref() == Some("0") {
        return None;
    }

    if content.is_empty() {
        return None;
    }

    let config = Config::load().ok()?;

    if !config.tee.enabled {
        return None;
    }

    let tee_dir = get_tee_dir(&config)?;
    let tee_dir = std::fs::create_dir_all(&tee_dir).ok().and(Some(tee_dir))?;

    write_tee_file(
        content,
        command_slug,
        &tee_dir,
        config.tee.max_file_size,
        config.tee.max_files,
    )
}

/// Returns `[full output saved — run: cat ~/path]`, or None if tee is disabled/skipped.
pub fn force_tee_hint(raw: &str, command_slug: &str) -> Option<String> {
    let path = force_tee_path(raw, command_slug)?;
    Some(format_hint(&path))
}

/// Returns `[see remaining: tail -n +{line_offset} ~/path]`, or None if tee is disabled/skipped.
pub fn force_tee_tail_hint(
    content: &str,
    command_slug: &str,
    line_offset: usize,
) -> Option<String> {
    let path = force_tee_path(content, command_slug)?;
    Some(format!(
        "[see remaining: tail -n +{} {}]",
        line_offset,
        display_path(&path)
    ))
}

/// Like `force_tee_tail_hint`, but also injects a preview of the first hidden
/// lines so an AI consumer can decide whether the truncated portion is worth
/// reading without making an extra `cat`/`tail` tool call.
///
/// `line_offset` is 1-based: it identifies the first hidden line (i.e., the
/// line just after the truncation cut). The preview shows up to the first 3
/// non-empty hidden lines, each capped at 80 characters.
///
/// Output formats:
/// - With preview lines:
///   `[+N items hidden — first hidden: <l1> | <l2> | <l3>]\n[full: cat ~/path]`
/// - Without preview (all hidden lines empty/blank):
///   `[+N items hidden — run: cat ~/path]`
///
/// Returns `None` if tee is disabled or skipped.
//
// NOTE: Intentionally not yet wired into any filter caller. This change ships
// the helper so consumers can adopt it incrementally per filter (the existing
// `force_tee_tail_hint` callers stay on the simpler hint until they prove the
// preview adds signal). Tests below exercise it directly.
#[allow(dead_code)]
pub fn force_tee_tail_hint_with_preview(
    content: &str,
    command_slug: &str,
    line_offset: usize,
) -> Option<String> {
    let path = force_tee_path(content, command_slug)?;

    let total_lines = content.lines().count();
    // line_offset is 1-based; skip everything before it.
    let skip = line_offset.saturating_sub(1);
    let hidden_count = total_lines.saturating_sub(skip);

    // Build up to 3 preview snippets from the first hidden lines, char-safe
    // truncated to 80 characters each. Skip blank lines.
    let previews: Vec<String> = content
        .lines()
        .skip(skip)
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed.chars().count() > 80 {
                let truncated: String = trimmed.chars().take(80).collect();
                Some(format!("{}…", truncated))
            } else {
                Some(trimmed.to_string())
            }
        })
        .take(3)
        .collect();

    let display = display_path(&path);
    if previews.is_empty() {
        Some(format!(
            "[+{} items hidden — run: cat {}]",
            hidden_count, display
        ))
    } else {
        Some(format!(
            "[+{} items hidden — first hidden: {}]\n[full: cat {}]",
            hidden_count,
            previews.join(" | "),
            display
        ))
    }
}

/// TeeMode controls when tee writes files.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TeeMode {
    #[default]
    Failures,
    Always,
    Never,
}

/// Configuration for the tee feature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeeConfig {
    pub enabled: bool,
    pub mode: TeeMode,
    pub max_files: usize,
    pub max_file_size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<PathBuf>,
}

impl Default for TeeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: TeeMode::default(),
            max_files: DEFAULT_MAX_FILES,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            directory: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sanitize_slug() {
        assert_eq!(sanitize_slug("cargo_test"), "cargo_test");
        assert_eq!(sanitize_slug("cargo test"), "cargo_test");
        assert_eq!(sanitize_slug("cargo-test"), "cargo-test");
        assert_eq!(sanitize_slug("go/test/./pkg"), "go_test___pkg");
        // Truncate at 40
        let long = "a".repeat(50);
        assert_eq!(sanitize_slug(&long).len(), 40);
    }

    #[test]
    fn test_should_tee_disabled() {
        let config = TeeConfig {
            enabled: false,
            ..TeeConfig::default()
        };
        let dir = PathBuf::from("/tmp/tee");
        assert!(should_tee(&config, 1000, 1, Some(dir)).is_none());
    }

    #[test]
    fn test_should_tee_never_mode() {
        let config = TeeConfig {
            mode: TeeMode::Never,
            ..TeeConfig::default()
        };
        let dir = PathBuf::from("/tmp/tee");
        assert!(should_tee(&config, 1000, 1, Some(dir)).is_none());
    }

    #[test]
    fn test_should_tee_skip_small_output() {
        let config = TeeConfig::default();
        let dir = PathBuf::from("/tmp/tee");
        // Below MIN_TEE_SIZE (500)
        assert!(should_tee(&config, 100, 1, Some(dir)).is_none());
    }

    #[test]
    fn test_should_tee_skip_success_in_failures_mode() {
        let config = TeeConfig::default(); // mode = Failures
        let dir = PathBuf::from("/tmp/tee");
        assert!(should_tee(&config, 1000, 0, Some(dir)).is_none());
    }

    #[test]
    fn test_should_tee_proceed_on_failure() {
        let config = TeeConfig::default(); // mode = Failures
        let dir = PathBuf::from("/tmp/tee");
        assert!(should_tee(&config, 1000, 1, Some(dir)).is_some());
    }

    #[test]
    fn test_should_tee_always_mode_success() {
        let config = TeeConfig {
            mode: TeeMode::Always,
            ..TeeConfig::default()
        };
        let dir = PathBuf::from("/tmp/tee");
        assert!(should_tee(&config, 1000, 0, Some(dir)).is_some());
    }

    #[test]
    fn test_write_tee_file_creates_file() {
        let tmpdir = tempfile::tempdir().unwrap();
        let content = "error: test failed\n".repeat(50);
        let result = write_tee_file(
            &content,
            "cargo_test",
            tmpdir.path(),
            DEFAULT_MAX_FILE_SIZE,
            20,
        );
        assert!(result.is_some());

        let path = result.unwrap();
        assert!(path.exists());
        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains("error: test failed"));
    }

    #[test]
    fn test_write_tee_file_truncation() {
        let tmpdir = tempfile::tempdir().unwrap();
        let big_output = "x".repeat(2000);
        // Set max_file_size to 1000 bytes
        let result = write_tee_file(&big_output, "test", tmpdir.path(), 1000, 20);
        assert!(result.is_some());

        let path = result.unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("--- truncated at 1000 bytes ---"));
        assert!(content.len() < 2000);
    }

    #[test]
    fn test_write_tee_file_truncation_utf8_boundary() {
        let tmpdir = tempfile::tempdir().unwrap();
        // Create a string where the truncation point falls inside a multi-byte char.
        // Japanese chars are 3 bytes each in UTF-8.
        // 332 chars * 3 bytes = 996 bytes, then one more = 999 bytes.
        // With max_file_size=998, the cut falls mid-character.
        let japanese = "\u{6F22}".repeat(333); // 999 bytes of 3-byte chars
        assert_eq!(japanese.len(), 999);

        // Truncate at 998 — falls in the middle of the 333rd character
        let result = write_tee_file(&japanese, "test_utf8", tmpdir.path(), 998, 20);
        assert!(result.is_some());

        let path = result.unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("--- truncated at 998 bytes ---"));
        // Should contain 332 full characters (996 bytes), not panic
        assert!(content.starts_with(&"\u{6F22}".repeat(332)));
    }

    #[test]
    fn test_write_tee_file_truncation_emoji() {
        let tmpdir = tempfile::tempdir().unwrap();
        // Emoji are 4 bytes each in UTF-8
        let emojis = "\u{1F600}".repeat(100); // 400 bytes
        assert_eq!(emojis.len(), 400);

        // Truncate at 201 — falls mid-emoji (4-byte boundary is at 200, 204)
        let result = write_tee_file(&emojis, "test_emoji", tmpdir.path(), 201, 20);
        assert!(result.is_some());

        let path = result.unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("--- truncated at 201 bytes ---"));
        // The emoji portion should be exactly 200 bytes (50 emojis),
        // rounded down from 201 to the nearest char boundary
        let target = "\u{1F600}".repeat(50);
        assert!(content.starts_with(&target));
    }

    #[test]
    fn test_cleanup_old_files() {
        let tmpdir = tempfile::tempdir().unwrap();
        let dir = tmpdir.path();

        // Create 25 .log files
        for i in 0..25 {
            let filename = format!("{:010}_{}.log", 1000000 + i, "test");
            fs::write(dir.join(&filename), "content").unwrap();
        }

        cleanup_old_files(dir, 20);

        let remaining: Vec<_> = fs::read_dir(dir).unwrap().filter_map(|e| e.ok()).collect();
        assert_eq!(remaining.len(), 20);

        // Oldest 5 should be removed
        for i in 0..5 {
            let filename = format!("{:010}_{}.log", 1000000 + i, "test");
            assert!(!dir.join(&filename).exists());
        }
        // Newest 20 should remain
        for i in 5..25 {
            let filename = format!("{:010}_{}.log", 1000000 + i, "test");
            assert!(dir.join(&filename).exists());
        }
    }

    #[test]
    fn test_format_hint() {
        let path = PathBuf::from("/tmp/rtk/tee/123_cargo_test.log");
        let hint = format_hint(&path);
        assert!(hint.starts_with("[full output saved — run: cat "));
        assert!(hint.ends_with(']'));
        assert!(hint.contains("123_cargo_test.log"));
    }

    #[test]
    fn test_tee_hint_includes_cat_command() {
        // The hint must surface a runnable `cat` command so AI consumers know
        // exactly how to retrieve the full output, not just where it lives.
        let path = PathBuf::from("/tmp/rtk/tee/456_go_test.log");
        let hint = format_hint(&path);
        assert!(
            hint.contains("run: cat "),
            "hint must include 'run: cat <path>' instruction, got: {hint}"
        );
        assert!(
            hint.contains("456_go_test.log"),
            "hint must include the path, got: {hint}"
        );
    }

    #[test]
    fn test_tee_config_default() {
        let config = TeeConfig::default();
        assert!(config.enabled);
        assert_eq!(config.mode, TeeMode::Failures);
        assert_eq!(config.max_files, 20);
        assert_eq!(config.max_file_size, 1_048_576);
        assert!(config.directory.is_none());
    }

    #[test]
    fn test_tee_config_deserialize() {
        let toml_str = r#"
enabled = true
mode = "always"
max_files = 10
max_file_size = 524288
directory = "/tmp/rtk-tee"
"#;
        let config: TeeConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enabled);
        assert_eq!(config.mode, TeeMode::Always);
        assert_eq!(config.max_files, 10);
        assert_eq!(config.max_file_size, 524288);
        assert_eq!(config.directory, Some(PathBuf::from("/tmp/rtk-tee")));

        // Round-trip
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: TeeConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.mode, TeeMode::Always);
        assert_eq!(deserialized.max_files, 10);
    }

    #[test]
    fn test_tee_mode_serde() {
        // Test all modes via JSON
        let mode: TeeMode = serde_json::from_str(r#""always""#).unwrap();
        assert_eq!(mode, TeeMode::Always);

        let mode: TeeMode = serde_json::from_str(r#""failures""#).unwrap();
        assert_eq!(mode, TeeMode::Failures);

        let mode: TeeMode = serde_json::from_str(r#""never""#).unwrap();
        assert_eq!(mode, TeeMode::Never);
    }

    #[test]
    fn test_force_tee_hint_skip_empty() {
        let hint = force_tee_hint("", "test_cmd");
        assert!(hint.is_none(), "Should skip empty content");
    }

    #[test]
    fn test_force_tee_hint_respects_env_disable() {
        // When RTCO_TEE=0, force_tee_hint should return None
        std::env::set_var("RTCO_TEE", "0");
        let large_output = "x".repeat(1000);
        let hint = force_tee_hint(&large_output, "test_cmd");
        std::env::remove_var("RTCO_TEE");
        assert!(hint.is_none(), "Should respect RTCO_TEE=0");
    }

    #[test]
    fn test_force_tee_tail_hint_skip_empty() {
        let hint = force_tee_tail_hint("", "test_cmd", 22);
        assert!(hint.is_none(), "Should skip empty content");
    }

    #[test]
    fn test_force_tee_tail_hint_format() {
        let path = std::path::PathBuf::from("/tmp/rtk/tee/123_docker_images.log");
        let display = display_path(&path);
        let hint = format!("[see remaining: tail -n +{} {}]", 22, display);
        assert!(hint.starts_with("[see remaining: tail -n +22 "));
        assert!(hint.ends_with(']'));
        assert!(hint.contains("123_docker_images.log"));
    }

    #[test]
    fn test_force_tee_tail_hint_with_preview_skip_empty() {
        let hint = force_tee_tail_hint_with_preview("", "test_cmd", 5);
        assert!(hint.is_none(), "Should skip empty content");
    }

    #[test]
    fn test_truncate_preview_shows_hidden_lines() {
        // Build content where lines 4..=10 are hidden (offset = 4, 1-based).
        // The preview should surface the first 3 non-empty hidden lines and
        // include a runnable `cat` command for the saved file.
        let lines: Vec<String> = (1..=10).map(|i| format!("item-{i}")).collect();
        let content = lines.join("\n");

        // Use a per-test tee dir so we don't pollute the user environment.
        let tmpdir = tempfile::tempdir().unwrap();
        std::env::set_var("RTCO_TEE_DIR", tmpdir.path());
        // Make sure no global env is forcing tee off.
        std::env::remove_var("RTCO_TEE");

        let hint = force_tee_tail_hint_with_preview(&content, "preview_test", 4);

        std::env::remove_var("RTCO_TEE_DIR");

        let hint = hint.expect("preview hint should be produced for non-empty content");
        assert!(
            hint.contains("+7 items hidden"),
            "should report 7 hidden items (lines 4..=10), got: {hint}"
        );
        assert!(
            hint.contains("first hidden: item-4 | item-5 | item-6"),
            "should preview the first 3 hidden lines, got: {hint}"
        );
        assert!(
            hint.contains("[full: cat "),
            "should include `cat` recovery command, got: {hint}"
        );
    }

    #[test]
    fn test_truncate_preview_truncates_long_lines() {
        let long_line = "x".repeat(200);
        let content = format!("visible\n{long_line}\nshort tail");

        let tmpdir = tempfile::tempdir().unwrap();
        std::env::set_var("RTCO_TEE_DIR", tmpdir.path());
        std::env::remove_var("RTCO_TEE");

        let hint = force_tee_tail_hint_with_preview(&content, "preview_long", 2);

        std::env::remove_var("RTCO_TEE_DIR");

        let hint = hint.expect("hint should be produced");
        // The 200-char line must be truncated to 80 chars + ellipsis in preview.
        let expected_prefix: String = "x".repeat(80);
        assert!(
            hint.contains(&format!("{expected_prefix}…")),
            "long preview line should be truncated to 80 chars + ellipsis, got: {hint}"
        );
        assert!(hint.contains("short tail"));
    }

    #[test]
    fn test_truncate_preview_skips_blank_hidden_lines() {
        // Hidden region starts with blank lines; preview should skip them and
        // surface the first non-empty line instead.
        let content = "visible-1\n\n\n\nactual-content\nmore-content";

        let tmpdir = tempfile::tempdir().unwrap();
        std::env::set_var("RTCO_TEE_DIR", tmpdir.path());
        std::env::remove_var("RTCO_TEE");

        let hint = force_tee_tail_hint_with_preview(content, "preview_blanks", 2);

        std::env::remove_var("RTCO_TEE_DIR");

        let hint = hint.expect("hint should be produced");
        assert!(
            hint.contains("first hidden: actual-content | more-content"),
            "should skip blank lines and preview content, got: {hint}"
        );
    }
}
