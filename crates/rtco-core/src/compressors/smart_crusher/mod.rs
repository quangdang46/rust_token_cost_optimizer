//! SmartCrusher — JSON array compression.
//!
//! Analyzes JSON output, classifies arrays by element type, selects
//! optimal compression strategies, and crushes them in-place.
//! Ported from headroom's `transforms/smart_crusher/`.
//!
//! # Usage
//!
//! ```rust
//! use rtco_core::compressors::smart_crusher;
//!
//! let data = r#"{"items":[{"id":1},{"id":2},{"id":3}]}"#;
//! let compressed = smart_crusher::compress_json(data);
//! assert!(!compressed.is_empty());
//! let (output, stats) = smart_crusher::compress_json_with_stats(data).unwrap();
//! println!("Saved {:.1}%", stats.savings_ratio() * 100.0);
//! ```

pub mod anchors;
pub mod classifier;
pub mod constraints;
pub mod crusher;
pub mod error_keywords;
pub mod field_detect;
pub mod hashing;
pub mod outliers;
pub mod planner;
pub mod traits;

pub use anchors::{extract_query_anchors, item_matches_anchors};
pub use constraints::{
    default_oss_constraints, KeepErrorsConstraint, KeepStructuralOutliersConstraint,
};
pub use crusher::{compress_json, compress_json_with_stats};
pub use error_keywords::ERROR_KEYWORDS;
pub use hashing::hash_field_name;
pub use outliers::{
    detect_error_items_for_preservation, detect_rare_status_values, detect_structural_outliers,
};
pub use planner::SmartCrusherConfig;
pub use traits::{Constraint, CrushEvent, Observer};
