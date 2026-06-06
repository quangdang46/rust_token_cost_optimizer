//! Configuration for the compression pipeline.

use serde::{Deserialize, Serialize};

/// Configuration for the compression pipeline orchestrator.
///
/// Controls token budget, offload behavior, and CCR integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineConfig {
    /// Maximum tokens allowed in output. 0 means no limit (reformat only).
    pub max_tokens: usize,
    /// Fraction threshold for offloading (0.0 = offload nothing, 1.0 = offload everything).
    pub offload_threshold: f64,
    /// Whether to enable CCR storage for offloaded lines.
    pub enable_ccr: bool,
    /// Whether pipeline compression is enabled.
    pub enabled: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            offload_threshold: 0.3,
            enable_ccr: false,
            enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let cfg = PipelineConfig::default();
        assert_eq!(cfg.max_tokens, 4096);
        assert!((cfg.offload_threshold - 0.3).abs() < 1e-10);
        assert!(!cfg.enable_ccr);
        assert!(!cfg.enabled);
    }

    #[test]
    fn test_serde_roundtrip() {
        let cfg = PipelineConfig::default();
        let toml_str = toml::to_string(&cfg).expect("serialize");
        let deserialized: PipelineConfig = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(deserialized.max_tokens, cfg.max_tokens);
    }
}
