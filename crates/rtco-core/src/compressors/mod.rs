//! Compression transforms for rtco.
//!
//! This module houses compression algorithms ported from headroom:
//! - SmartCrusher: JSON array compression
//! - DiffCompressor: unified diff compression
//! - LogCompressor: log output compression
//! - SearchCompressor: grep/rg output compression

pub mod diff_compressor;
pub mod log_compressor;
pub mod search_compressor;
pub mod smart_crusher;

use serde::{Deserialize, Serialize};

/// Compression strategy for a JSON array field.
///
/// Determines how an array is reduced to save tokens while preserving
/// semantic meaning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// Keep the array as-is — no compression applied
    #[serde(rename = "none")]
    None,
    /// Drop the array entirely
    #[serde(rename = "skip")]
    Skip,
    /// Keep only the first N elements
    #[serde(rename = "top_n")]
    TopN(usize),
    /// Keep a sampled fraction (0.0–1.0) of elements
    #[serde(rename = "sample")]
    Sample(f64),
    /// Keep a stratified sample of N elements (simulated clustering)
    #[serde(rename = "cluster_sample")]
    ClusterSample(usize),
}

impl CompressionStrategy {
    /// Returns the number of elements that would be kept for a given array size.
    pub fn keep_count(&self, array_len: usize) -> usize {
        match self {
            CompressionStrategy::None => array_len,
            CompressionStrategy::Skip => 0,
            CompressionStrategy::TopN(n) => (*n).min(array_len),
            CompressionStrategy::Sample(f) => {
                let n = (array_len as f64 * f).round() as usize;
                n.max(1).min(array_len)
            }
            CompressionStrategy::ClusterSample(n) => (*n).min(array_len),
        }
    }
}

/// Statistics about a JSON field discovered during analysis.
#[derive(Debug, Clone)]
pub struct FieldStats {
    /// JSON path to the field (e.g., ["results", "items"])
    pub field_path: Vec<String>,
    /// Original array length before compression
    pub array_length: usize,
    /// Approximate number of unique values (if computed)
    pub unique_values: Option<usize>,
    /// Estimated token savings after compression
    pub estimated_token_savings: usize,
}

/// Classification of the elements inside a JSON array.
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayType {
    /// Array of objects: `[{...}, {...}]`
    DictArray,
    /// Array of strings: `["a", "b", "c"]`
    StringArray,
    /// Array of numbers: `[1, 2, 3]`
    NumberArray,
    /// Array of booleans: `[true, false]`
    BoolArray,
    /// Array of arrays: `[[1,2], [3,4]]`
    NestedArray,
    /// Array of mixed/null types
    MixedArray,
    /// Empty array
    Empty,
}

impl ArrayType {
    /// Recommend a default compression strategy for this array type.
    pub fn default_strategy(&self, array_len: usize) -> CompressionStrategy {
        match self {
            ArrayType::DictArray | ArrayType::NestedArray => {
                if array_len > 50 {
                    CompressionStrategy::TopN(20)
                } else if array_len > 20 {
                    CompressionStrategy::TopN(10)
                } else {
                    CompressionStrategy::None
                }
            }
            ArrayType::StringArray => {
                if array_len > 100 {
                    CompressionStrategy::Sample(0.3)
                } else if array_len > 30 {
                    CompressionStrategy::TopN(15)
                } else {
                    CompressionStrategy::None
                }
            }
            ArrayType::NumberArray => {
                if array_len > 50 {
                    CompressionStrategy::ClusterSample(10)
                } else {
                    CompressionStrategy::None
                }
            }
            ArrayType::BoolArray | ArrayType::MixedArray => {
                if array_len > 20 {
                    CompressionStrategy::TopN(5)
                } else {
                    CompressionStrategy::None
                }
            }
            ArrayType::Empty => CompressionStrategy::Skip,
        }
    }
}

/// Analysis result for a JSON array.
#[derive(Debug, Clone)]
pub struct ArrayAnalysis {
    pub array_type: ArrayType,
    pub field_stats: FieldStats,
    pub crushable: bool,
}

/// Compression plan for a single JSON array field.
#[derive(Debug, Clone)]
pub struct CompressionPlan {
    pub field_path: Vec<String>,
    pub strategy: CompressionStrategy,
    pub estimated_reduction: f64,
}

/// Summary of a crushed field.
#[derive(Debug, Clone)]
pub struct CrushedField {
    pub field_path: String,
    pub original_count: usize,
    pub compressed_count: usize,
    pub strategy: CompressionStrategy,
}

/// Result of a SmartCrusher compression operation.
#[derive(Debug, Clone)]
pub struct CrushResult {
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub crushed_fields: Vec<CrushedField>,
}

impl CrushResult {
    /// Token savings ratio (0.0–1.0). Returns 0.0 if original was empty.
    pub fn savings_ratio(&self) -> f64 {
        if self.original_tokens == 0 {
            return 0.0;
        }
        1.0 - (self.compressed_tokens as f64 / self.original_tokens as f64)
    }
}
