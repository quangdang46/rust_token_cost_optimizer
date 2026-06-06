//! Strategy selection for JSON array compression.
//!
//! Given an `ArrayAnalysis`, selects the best `CompressionStrategy`
//! based on array type, size, and uniqueness. Ported from headroom.

use crate::compressors::{ArrayAnalysis, ArrayType, CompressionPlan, CompressionStrategy};

/// Configuration for strategy selection.
#[derive(Debug, Clone)]
pub struct CrusherConfig {
    /// Maximum depth of nested arrays to analyze
    pub max_depth: usize,
    /// Minimum array length to consider crushing
    pub min_array_size: usize,
    /// Default strategy for arrays that pass thresholds
    pub default_strategy: CompressionStrategy,
}

impl Default for CrusherConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            min_array_size: 5,
            default_strategy: CompressionStrategy::TopN(10),
        }
    }
}

/// Create a compression plan for an analyzed array.
pub fn plan_compression(analysis: &ArrayAnalysis, config: &CrusherConfig) -> CompressionPlan {
    let field_path = analysis.field_stats.field_path.clone();
    let len = analysis.field_stats.array_length;

    // Not crushable → no compression
    if !analysis.crushable || len == 0 {
        return CompressionPlan {
            field_path,
            strategy: CompressionStrategy::None,
            estimated_reduction: 0.0,
        };
    }

    // Below min size → no compression
    if len < config.min_array_size {
        return CompressionPlan {
            field_path: field_path.clone(),
            strategy: CompressionStrategy::None,
            estimated_reduction: 0.0,
        };
    }

    // Select strategy based on type and length
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
    _config: &CrusherConfig,
) -> CompressionStrategy {
    match array_type {
        ArrayType::DictArray => {
            // Objects are high-value → conservative TopN
            if len > 200 {
                CompressionStrategy::TopN(30)
            } else if len > 50 {
                CompressionStrategy::TopN(15)
            } else {
                CompressionStrategy::TopN(10)
            }
        }
        ArrayType::StringArray => {
            // Strings — sample proportionally if many
            if len > 200 {
                CompressionStrategy::Sample(0.2)
            } else if len > 50 {
                CompressionStrategy::Sample(0.3)
            } else if len > 20 {
                CompressionStrategy::TopN(10)
            } else {
                CompressionStrategy::None
            }
        }
        ArrayType::NumberArray => {
            // Numbers — cluster sample to preserve distribution
            if len > 100 {
                CompressionStrategy::ClusterSample(15)
            } else if len > 30 {
                CompressionStrategy::ClusterSample(10)
            } else {
                CompressionStrategy::None
            }
        }
        ArrayType::BoolArray | ArrayType::MixedArray => {
            // Booleans/mixed — conservative
            if len > 20 {
                CompressionStrategy::TopN(5)
            } else {
                CompressionStrategy::None
            }
        }
        ArrayType::NestedArray => {
            // Nested arrays — TopN only
            if len > 30 {
                CompressionStrategy::TopN(10)
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
        let plan = plan_compression(&analysis, &CrusherConfig::default());
        assert_eq!(plan.strategy, CompressionStrategy::None);
    }

    #[test]
    fn test_large_dict_array_top_n() {
        let analysis = make_analysis(ArrayType::DictArray, 100);
        let plan = plan_compression(&analysis, &CrusherConfig::default());
        assert_eq!(plan.strategy, CompressionStrategy::TopN(15));
    }

    #[test]
    fn test_non_crushable_no_compression() {
        let analysis = make_analysis(ArrayType::Empty, 0);
        let plan = plan_compression(&analysis, &CrusherConfig::default());
        assert_eq!(plan.strategy, CompressionStrategy::None);
    }

    #[test]
    fn test_estimated_reduction() {
        let analysis = make_analysis(ArrayType::StringArray, 100);
        let plan = plan_compression(&analysis, &CrusherConfig::default());
        assert!(plan.estimated_reduction > 0.0);
        assert!(plan.estimated_reduction < 1.0);
    }

    #[test]
    fn test_variable_min_size() {
        let config = CrusherConfig {
            min_array_size: 50,
            ..Default::default()
        };
        let analysis = make_analysis(ArrayType::DictArray, 30);
        let plan = plan_compression(&analysis, &config);
        assert_eq!(plan.strategy, CompressionStrategy::None);
    }
}
