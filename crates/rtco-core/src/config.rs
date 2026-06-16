//! Reads user settings from config.toml.

use super::constants::{CONFIG_TOML, DEFAULT_HISTORY_DAYS, RTCO_DATA_DIR};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub tracking: TrackingConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub filters: FilterConfig,
    #[serde(default)]
    pub tee: crate::tee::TeeConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub hooks: HooksConfig,
    #[serde(default)]
    pub limits: LimitsConfig,
    #[serde(default)]
    pub tokenizer: TokenEstimatorConfig,
    #[serde(default)]
    pub ccr: CcrConfig,

    #[serde(default)]
    pub pipeline: crate::pipeline::PipelineConfig,

    #[serde(default)]
    pub compressors: CompressorsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksConfig {
    #[serde(default)]
    pub exclude_commands: Vec<String>,
    #[serde(default)]
    pub transparent_prefixes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingConfig {
    pub enabled: bool,
    pub history_days: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_path: Option<PathBuf>,
    /// Replace stored project paths with `<basename>#<8-hex-sha256>` so the
    /// tracking DB does not pin a user's private filesystem layout. Defaults
    /// to `true`; set to `false` to keep raw paths for `rtco gain --by-project`
    /// scripts that depend on prefix matching.
    #[serde(default = "default_true")]
    pub hash_project_paths: bool,
}

fn default_true() -> bool {
    true
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            history_days: DEFAULT_HISTORY_DAYS as u32,
            database_path: None,
            hash_project_paths: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub colors: bool,
    pub emoji: bool,
    pub max_width: usize,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            colors: true,
            emoji: true,
            max_width: 120,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterConfig {
    pub ignore_dirs: Vec<String>,
    pub ignore_files: Vec<String>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            ignore_dirs: vec![
                ".git".into(),
                "node_modules".into(),
                "target".into(),
                "__pycache__".into(),
                ".venv".into(),
                "vendor".into(),
            ],
            ignore_files: vec!["*.lock".into(), "*.min.js".into(), "*.min.css".into()],
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consent_given: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consent_date: Option<String>,
}

/// Configuration for token estimation.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenEstimatorConfig {
    /// Tokenizer backend to use: "approximate" (default), "tiktoken", or "huggingface".
    #[serde(default = "default_tokenizer_backend")]
    pub backend: String,
    /// Whether token estimation is enabled. When disabled, the old
    /// whitespace-based count is used.
    #[serde(default = "default_tokenizer_enabled")]
    pub enabled: bool,
}

fn default_tokenizer_backend() -> String {
    "approximate".to_string()
}

fn default_tokenizer_enabled() -> bool {
    true
}

impl Default for TokenEstimatorConfig {
    fn default() -> Self {
        Self {
            backend: default_tokenizer_backend(),
            enabled: true,
        }
    }
}

/// Configuration for the Compression Context Registry (CCR).
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CcrConfig {
    /// Whether CCR is enabled. When disabled, no content is stored.
    pub enabled: bool,
    /// Default TTL in days for stored entries. 0 means no expiry.
    pub default_ttl_days: u64,
    /// Minimum line length in chars to trigger offload storage.
    pub offload_threshold_chars: usize,
}

impl Default for CcrConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_ttl_days: 7,
            offload_threshold_chars: 100,
        }
    }
}

/// Configuration for an LLM model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    /// Maximum context window size in tokens.
    pub context_limit: usize,
    /// Cost per token in USD (fractional).
    pub cost_per_token: f64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            name: "claude-sonnet-4-20250514".into(),
            context_limit: 200_000,
            cost_per_token: 0.000_003,
        }
    }
}

/// Load model definitions from an external models.toml file.
///
/// Returns a list of `ModelConfig`s with known pricing and context limits.
/// Falls back to sensible built-in defaults when the file is missing.
pub fn load_models() -> Vec<ModelConfig> {
    let models_path = models_toml_path();
    models_path
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|content| toml::from_str::<ModelsFile>(&content).ok())
        .map(|f| f.model)
        .unwrap_or_else(default_models)
}

#[derive(Debug, Deserialize)]
struct ModelsFile {
    #[serde(rename = "model")]
    model: Vec<ModelConfig>,
}

fn models_toml_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("rtco").join("models.toml"))
}

fn default_models() -> Vec<ModelConfig> {
    vec![
        ModelConfig {
            name: "claude-sonnet-4-20250514".into(),
            context_limit: 200_000,
            cost_per_token: 0.000_003,
        },
        ModelConfig {
            name: "claude-sonnet-4-20250514-100k".into(),
            context_limit: 100_000,
            cost_per_token: 0.000_003,
        },
        ModelConfig {
            name: "claude-3-5-sonnet-20241022".into(),
            context_limit: 200_000,
            cost_per_token: 0.000_003,
        },
        ModelConfig {
            name: "gpt-4o-2024-11-20".into(),
            context_limit: 128_000,
            cost_per_token: 0.000_002_5,
        },
        ModelConfig {
            name: "gemini-2.5-pro".into(),
            context_limit: 1_000_000,
            cost_per_token: 0.000_001_25,
        },
    ]
}

/// Stub: placeholder for future economics integration.
/// Will compute per-model savings in USD based on `ModelConfig`.
pub fn compute_model_savings(_model: &ModelConfig, _tokens_saved: usize) -> f64 {
    // TODO: wire into cc_economics for real cost projections
    0.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct CompressorsConfig {
    pub enabled: bool,
    pub diff_max_context_lines: usize,
    pub search_max_files: usize,
    pub log_collapse_repeated: bool,
    pub smart_crusher_min_array: usize,
}

impl Default for CompressorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            diff_max_context_lines: 3,
            search_max_files: 20,
            log_collapse_repeated: false,
            smart_crusher_min_array: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    pub grep_max_results: usize,
    pub grep_max_per_file: usize,
    pub status_max_files: usize,
    pub status_max_untracked: usize,
    pub passthrough_max_chars: usize,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            grep_max_results: 200,
            grep_max_per_file: 25,
            status_max_files: 15,
            status_max_untracked: 10,
            passthrough_max_chars: 2000,
        }
    }
}

/// Get limits config. Falls back to defaults if config can't be loaded.
pub fn limits() -> LimitsConfig {
    Config::load().map(|c| c.limits).unwrap_or_default()
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = get_config_path()?;

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = get_config_path()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn create_default() -> Result<PathBuf> {
        let config = Config::default();
        config.save()?;
        get_config_path()
    }
}

fn get_config_path() -> Result<PathBuf> {
    // Priority 1: explicit override (mirrors RTCO_DB_PATH for `Tracker`).
    // Mostly used by tests to keep `Config::load()` deterministic without
    // touching the real `$HOME`.
    if let Ok(custom) = std::env::var("RTCO_CONFIG_PATH") {
        return Ok(PathBuf::from(custom));
    }
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    Ok(config_dir.join(RTCO_DATA_DIR).join(CONFIG_TOML))
}

pub fn show_config() -> Result<()> {
    let path = get_config_path()?;
    println!("Config: {}", path.display());
    println!();

    if path.exists() {
        let config = Config::load()?;
        println!("{}", toml::to_string_pretty(&config)?);
    } else {
        println!("(default config, file not created)");
        println!();
        let config = Config::default();
        println!("{}", toml::to_string_pretty(&config)?);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_config_deserialize() {
        let toml = r#"
[hooks]
exclude_commands = ["curl", "gh"]
"#;
        let config: Config = toml::from_str(toml).expect("valid toml");
        assert_eq!(config.hooks.exclude_commands, vec!["curl", "gh"]);
    }

    #[test]
    fn test_hooks_config_default_empty() {
        let config = Config::default();
        assert!(config.hooks.exclude_commands.is_empty());
        assert!(config.hooks.transparent_prefixes.is_empty());
    }

    #[test]
    fn test_hooks_config_transparent_prefixes_deserialize() {
        let toml = r#"
[hooks]
transparent_prefixes = ["direnv exec .", "nix develop --command"]
"#;
        let config: Config = toml::from_str(toml).expect("valid toml");
        assert_eq!(
            config.hooks.transparent_prefixes,
            vec!["direnv exec .", "nix develop --command"]
        );
    }

    #[test]
    fn test_hooks_config_transparent_prefixes_missing_is_empty() {
        // Older configs that predate this field must still parse.
        let toml = r#"
[hooks]
exclude_commands = ["curl"]
"#;
        let config: Config = toml::from_str(toml).expect("valid toml");
        assert_eq!(config.hooks.exclude_commands, vec!["curl"]);
        assert!(config.hooks.transparent_prefixes.is_empty());
    }

    #[test]
    fn test_config_without_hooks_section_is_valid() {
        let toml = r#"
[tracking]
enabled = true
history_days = 90
"#;
        let config: Config = toml::from_str(toml).expect("valid toml");
        assert!(config.hooks.exclude_commands.is_empty());
    }

    #[test]
    fn test_old_toml_without_consent_fields() {
        let toml = r#"
[telemetry]
enabled = true
"#;
        let config: Config = toml::from_str(toml).expect("valid toml");
        assert!(config.telemetry.enabled);
        assert!(config.telemetry.consent_given.is_none());
        assert!(config.telemetry.consent_date.is_none());
    }

    #[test]
    fn test_telemetry_default_disabled() {
        let config = Config::default();
        assert!(!config.telemetry.enabled);
        assert!(config.telemetry.consent_given.is_none());
    }

    #[test]
    fn test_telemetry_consent_roundtrip() {
        let toml = r#"
[telemetry]
enabled = true
consent_given = true
consent_date = "2026-04-10T12:00:00Z"
"#;
        let config: Config = toml::from_str(toml).expect("valid toml");
        assert_eq!(config.telemetry.consent_given, Some(true));
        assert_eq!(
            config.telemetry.consent_date.as_deref(),
            Some("2026-04-10T12:00:00Z")
        );
    }
}
