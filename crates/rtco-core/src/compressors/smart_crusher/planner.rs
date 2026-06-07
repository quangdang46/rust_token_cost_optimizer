//! Strategy selection for JSON array compression.
//!
//! Given an `ArrayAnalysis` and `SmartCrusherConfig`, selects the best
//! compression strategy based on array type, size, uniqueness, and
//! configured thresholds. Ported from headroom's `smart_crusher/config.rs`
//! and `planning.rs`.

use serde::{Deserialize, Serialize};

use crate::compressors::{ArrayAnalysis, ArrayType, CompressionPlan, CompressionStrategy};

/// Configuration for SmartCrusher strategy selection.
///
/// 18 tuning knobs matching headroom's SmartCrusherConfig (Python).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartCrusherConfig {
    /// Master gate: whether SmartCrusher is enabled.
    pub enabled: bool,
    /// Minimum array length to consider crushing.
    pub min_items_to_analyze: usize,
    /// Only crush content with more than this many tokens.
    pub min_tokens_to_crush: usize,
    /// Standard deviations from the mean to count as a change point.
    #[serde(default = "default_variance_threshold")]
    pub variance_threshold: f64,
    /// Below this unique-ratio, a field is treated as nearly constant.
    #[serde(default = "default_uniqueness_threshold")]
    pub uniqueness_threshold: f64,
    /// Similarity score above which strings cluster together.
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f64,
    /// Target maximum items in the output.
    #[serde(default = "default_max_items_after_crush")]
    pub max_items_after_crush: usize,
    /// Whether to preserve detected change points.
    pub preserve_change_points: bool,
    /// Factor out constant-value fields across all items.
    pub factor_out_constants: bool,
    /// Include generated text summaries in output.
    pub include_summaries: bool,
    /// Use feedback hints to adjust compression aggressiveness.
    pub use_feedback_hints: bool,
    /// Minimum confidence to apply TOIN recommendations.
    #[serde(default = "default_toin_confidence_threshold")]
    pub toin_confidence_threshold: f64,
    /// Drop content-identical items before sampling.
    pub dedup_identical_items: bool,
    /// Fraction of K to allocate to the start of the array.
    #[serde(default = "default_first_fraction")]
    pub first_fraction: f64,
    /// Fraction of K to allocate to the end of the array.
    #[serde(default = "default_last_fraction")]
    pub last_fraction: f64,
    /// Minimum byte-savings ratio for lossless compaction.
    #[serde(default = "default_lossless_min_savings_ratio")]
    pub lossless_min_savings_ratio: f64,
    /// Whether to emit CCR markers for dropped rows.
    pub enable_ccr_marker: bool,
}

fn default_variance_threshold() -> f64 {
    2.0
}
fn default_uniqueness_threshold() -> f64 {
    0.1
}
fn default_similarity_threshold() -> f64 {
    0.8
}
fn default_max_items_after_crush() -> usize {
    15
}
fn default_toin_confidence_threshold() -> f64 {
    0.5
}
fn default_first_fraction() -> f64 {
    0.3
}
fn default_last_fraction() -> f64 {
    0.15
}
fn default_lossless_min_savings_ratio() -> f64 {
    0.30
}

impl Default for SmartCrusherConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_items_to_analyze: 5,
            min_tokens_to_crush: 200,
            variance_threshold: 2.0,
            uniqueness_threshold: 0.1,
            similarity_threshold: 0.8,
            max_items_after_crush: 15,
            preserve_change_points: true,
            factor_out_constants: false,
            include_summaries: false,
            use_feedback_hints: true,
            toin_confidence_threshold: 0.5,
            dedup_identical_items: true,
            first_fraction: 0.3,
            last_fraction: 0.15,
            lossless_min_savings_ratio: 0.30,
            enable_ccr_marker: true,
        }
    }
}

/// Create a compression plan for an analyzed array.
pub fn plan_compression(analysis: &ArrayAnalysis, config: &SmartCrusherConfig) -> CompressionPlan {
    let field_path = analysis.field_stats.field_path.clone();
    let len = analysis.field_stats.array_length;

    if !analysis.crushable || len == 0 {
        return CompressionPlan {
            field_path,
            strategy: CompressionStrategy::None,
            estimated_reduction: 0.0,
        };
    }

    if len < config.min_items_to_analyze {
        return CompressionPlan {
            field_path: field_path.clone(),
            strategy: CompressionStrategy::None,
            estimated_reduction: 0.0,
        };
    }

    let strategy = select_strategy(&analysis.array_type, len, config);
    let keep = strategy.keep_count(len);
    let estimated_reduction = if len == 0 {
        0.0
    } else {
        1.0 - (keep as f64 / len as f64)
    };

    CompressionPlan {
        field_path,
        strategy,
        estimated_reduction,
    }
}

/// Choose the best compression strategy for a given array type and length.
fn select_strategy(
    array_type: &ArrayType,
    len: usize,
    config: &SmartCrusherConfig,
) -> CompressionStrategy {
    let k = config.max_items_after_crush;

    match array_type {
        ArrayType::DictArray | ArrayType::NestedArray => {
            if len > k * 3 {
                CompressionStrategy::TopN(k)
            } else if len > k {
                CompressionStrategy::TopN(k.saturating_sub(5).max(5))
            } else {
                CompressionStrategy::None
            }
        }
        ArrayType::StringArray => {
            if len > k * 5 {
                CompressionStrategy::Sample(0.2)
            } else if len > k {
                CompressionStrategy::TopN(k)
            } else {
                CompressionStrategy::None
            }
        }
        ArrayType::NumberArray => {
            if len > k * 3 {
                CompressionStrategy::ClusterSample(k)
            } else if len > k {
                CompressionStrategy::ClusterSample(k.saturating_sub(5).max(3))
            } else {
                CompressionStrategy::None
            }
        }
        ArrayType::BoolArray | ArrayType::MixedArray => {
            if len > 20 {
                CompressionStrategy::TopN(5)
            } else {
                CompressionStrategy::None
            }
        }
        ArrayType::Empty => CompressionStrategy::Skip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compressors::{ArrayAnalysis, ArrayType, FieldStats};

    fn make_analysis(array_type: ArrayType, len: usize) -> ArrayAnalysis {
        ArrayAnalysis {
            array_type: array_type.clone(),
            field_stats: FieldStats {
                field_path: vec!["test".into()],
                array_length: len,
                unique_values: None,
                estimated_token_savings: 0,
            },
            crushable: matches!(
                array_type,
                ArrayType::DictArray
                    | ArrayType::StringArray
                    | ArrayType::NumberArray
                    | ArrayType::NestedArray
            ),
        }
    }

    #[test]
    fn test_small_array_no_compression() {
        let analysis = make_analysis(ArrayType::DictArray, 3);
        let plan = plan_compression(&analysis, &SmartCrusherConfig::default());
        assert_eq!(plan.strategy, CompressionStrategy::None);
    }

    #[test]
    fn test_large_dict_array_top_n() {
        let analysis = make_analysis(ArrayType::DictArray, 100);
        let plan = plan_compression(&analysis, &SmartCrusherConfig::default());
        assert_eq!(plan.strategy, CompressionStrategy::TopN(15));
    }

    #[test]
    fn test_non_crushable_no_compression() {
        let analysis = make_analysis(ArrayType::Empty, 0);
        let plan = plan_compression(&analysis, &SmartCrusherConfig::default());
        assert_eq!(plan.strategy, CompressionStrategy::None);
    }

    #[test]
    fn test_estimated_reduction() {
        let analysis = make_analysis(ArrayType::StringArray, 100);
        let plan = plan_compression(&analysis, &SmartCrusherConfig::default());
        assert!(plan.estimated_reduction > 0.0);
        assert!(plan.estimated_reduction < 1.0);
    }

    #[test]
    fn test_variable_min_size() {
        let config = SmartCrusherConfig {
            min_items_to_analyze: 50,
            ..Default::default()
        };
        let analysis = make_analysis(ArrayType::DictArray, 30);
        let plan = plan_compression(&analysis, &config);
        assert_eq!(plan.strategy, CompressionStrategy::None);
    }

    #[test]
    fn test_defaults_match_headroom() {
        let c = SmartCrusherConfig::default();
        assert!(c.enabled);
        assert_eq!(c.min_items_to_analyze, 5);
        assert_eq!(c.min_tokens_to_crush, 200);
        assert!((c.variance_threshold - 2.0).abs() < 1e-9);
        assert!((c.uniqueness_threshold - 0.1).abs() < 1e-9);
        assert!((c.max_items_after_crush as f64 - 15.0).abs() < 1e-9);
        assert!(c.preserve_change_points);
        assert!(!c.factor_out_constants);
        assert!(!c.include_summaries);
        assert!(c.use_feedback_hints);
        assert!((c.toin_confidence_threshold - 0.5).abs() < 1e-9);
        assert!(c.dedup_identical_items);
        assert!((c.first_fraction - 0.3).abs() < 1e-9);
        assert!((c.last_fraction - 0.15).abs() < 1e-9);
        assert!((c.lossless_min_savings_ratio - 0.30).abs() < 1e-9);
        assert!(c.enable_ccr_marker);
    }
}
