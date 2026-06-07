//! Extension traits for SmartCrusher.
//!
//! Ported from headroom's `transforms/smart_crusher/traits.rs`.
//! Three traits capture every decision a SmartCrusher makes:
//! - `Constraint`: which indices must be kept regardless of score/budget
//! - `Observer`: telemetry events after each crush()
//! - `Scorer`: relevance scoring (re-exported from signals module)

use serde_json::Value;

/// A hard preservation constraint: indices the allocator must keep
/// regardless of token budget or saliency score.
///
/// Constraints stack — the must-keep set is the union of every
/// constraint's output.
pub trait Constraint: Send + Sync {
    /// Stable identifier (e.g. `"keep_errors"`).
    fn name(&self) -> &str;

    /// Indices of items the allocator MUST keep.
    fn must_keep(&self, items: &[Value], item_strings: Option<&[String]>) -> Vec<usize>;
}

/// Telemetry event emitted at the end of each SmartCrusher::crush call.
pub struct CrushEvent {
    /// Strategy debug string (e.g. `"top_n(50->15)"`).
    pub strategy: String,
    /// Input byte length.
    pub input_bytes: usize,
    /// Output byte length.
    pub output_bytes: usize,
    /// Wall-clock duration in nanoseconds.
    pub elapsed_ns: u64,
    /// Whether the output differs from input.
    pub was_modified: bool,
}

/// Decision-stream hook after each SmartCrusher::crush invocation.
///
/// Observers fire synchronously on the crusher's thread in the order
/// they were added. Keep them cheap — network calls belong in a
/// background task.
pub trait Observer: Send + Sync {
    /// Stable identifier for filtering.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Called once per crush invocation after the result is computed.
    fn on_event(&self, event: &CrushEvent);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct AlwaysKeepFirst;
    impl Constraint for AlwaysKeepFirst {
        fn name(&self) -> &str {
            "always_keep_first"
        }
        fn must_keep(&self, items: &[Value], _: Option<&[String]>) -> Vec<usize> {
            if items.is_empty() {
                vec![]
            } else {
                vec![0]
            }
        }
    }

    #[test]
    fn test_constraint_returns_indices() {
        let items = vec![json!({"a": 1}), json!({"a": 2})];
        let c = AlwaysKeepFirst;
        assert_eq!(c.must_keep(&items, None), vec![0]);
        assert_eq!(c.name(), "always_keep_first");
    }

    #[test]
    fn test_constraint_handles_empty() {
        assert!(AlwaysKeepFirst.must_keep(&[], None).is_empty());
    }

    #[test]
    fn test_observer_event() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);
        struct TestObserver(Arc<AtomicUsize>);
        impl Observer for TestObserver {
            fn name(&self) -> &str {
                "test_observer"
            }
            fn on_event(&self, _: &CrushEvent) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let observer = TestObserver(count_clone);
        let event = CrushEvent {
            strategy: "smart_sample(30->15)".into(),
            input_bytes: 1000,
            output_bytes: 500,
            elapsed_ns: 12_345,
            was_modified: true,
        };
        observer.on_event(&event);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
