//! SmartCrusher — JSON array compression.
//!
//! Analyzes JSON output, classifies arrays by element type, selects
//! optimal compression strategies, and crushes them in-place.
//! Ported from headroom's `transforms/smart_crusher/` (21 files → 4 files).
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

pub mod classifier;
pub mod crusher;
pub mod planner;

pub use crusher::{compress_json, compress_json_with_stats};
pub use planner::CrusherConfig;
