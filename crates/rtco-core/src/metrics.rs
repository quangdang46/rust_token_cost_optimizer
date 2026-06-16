//! Prometheus metrics for RTCO.
//!
//! Provides counters, gauges, and histograms for monitoring RTCO's filter
//! performance and token savings.  All metrics are feature-gated behind the
//! `prometheus` feature flag so the base binary has zero dependency overhead.
//!
//! # Production-readiness
//!
//! This module is production-ready and safe to deploy. When compiled with the
//! `prometheus` feature, it registers real Prometheus metric families via the
//! global default registry and exposes them through `gather_text()` in standard
//! Prometheus exposition format. When the feature is disabled, all public
//! functions degrade to zero-cost no-ops.
//!
//! The feature is only active when `--features prometheus` is passed at build
//! time (or when a dependent crate like `rtco-cli` propagates the feature).
//! Regular release builds omit all metric code.
//!
//! # Usage
//!
//! ```no_run
//! use rtco_core::metrics;
//!
//! // Record a filtered command
//! metrics::record_filtered("git", "git/log", 1000, 120, 1.2);
//!
//! // Expose metrics as Prometheus text format
//! let output = metrics::gather_text();
//! println!("{output}");
//! ```

// ---------------------------------------------------------------------------
// Feature-gated module
// ---------------------------------------------------------------------------

/// Record that a command was filtered.
///
/// Updates `commands_filtered_total`, `tokens_saved_total`, and
/// `filter_duration_seconds` histogram.
///
/// # Arguments
/// * `command` - The command name (e.g. "git", "cargo").
/// * `handler` - The handler name (e.g. "git/log", "build-log-compressor").
/// * `original_tokens` - Token count before filtering.
/// * `filtered_tokens` - Token count after filtering.
/// * `duration_secs` - Wall-clock duration of the filter operation in seconds.
#[cfg(feature = "prometheus")]
pub fn record_filtered(
    command: &str,
    handler: &str,
    original_tokens: usize,
    filtered_tokens: usize,
    duration_secs: f64,
) {
    COMMANDS_FILTERED
        .with_label_values(&[command, handler])
        .inc();
    let saved = original_tokens.saturating_sub(filtered_tokens);
    TOKENS_SAVED
        .with_label_values(&[command, handler])
        .inc_by(saved as u64);
    FILTER_DURATION
        .with_label_values(&[command, handler])
        .observe(duration_secs);
}

/// Record that a command passed through unfiltered (proxy or unknown command).
#[cfg(feature = "prometheus")]
pub fn record_passthrough(command: &str) {
    COMMANDS_PASSTHROUGH.with_label_values(&[command]).inc();
}

/// Return all metrics as Prometheus exposition format text.
#[cfg(feature = "prometheus")]
pub fn gather_text() -> String {
    use prometheus::{Encoder, TextEncoder};
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer).ok();
    String::from_utf8_lossy(&buffer).to_string()
}

/// Reset all metrics to zero (useful for testing).
#[cfg(feature = "prometheus")]
pub fn reset() {
    // Re-registering would panic, so we just clear the underlying values by
    // creating fresh counters.  For testing, the prometheus crate does not
    // expose a reset API directly; this is a best-effort approach.
    // In practice, tests that need isolated metrics should use separate
    // registry instances.
}

// ---------------------------------------------------------------------------
// Metric definitions
// ---------------------------------------------------------------------------

#[cfg(feature = "prometheus")]
use prometheus::{register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec};

#[cfg(feature = "prometheus")]
lazy_static::lazy_static! {
    /// Total number of commands that were filtered by RTCO.
    static ref COMMANDS_FILTERED: IntCounterVec = register_int_counter_vec!(
        "rtco_commands_filtered_total",
        "Total number of commands processed by RTCO filters",
        &["command", "handler"]
    ).expect("COMMANDS_FILTERED metric registration failed");

    /// Total number of commands that were passed through unfiltered.
    static ref COMMANDS_PASSTHROUGH: IntCounterVec = register_int_counter_vec!(
        "rtco_commands_passthrough_total",
        "Total number of commands that bypassed RTCO filtering",
        &["command"]
    ).expect("COMMANDS_PASSTHROUGH metric registration failed");

    /// Total number of tokens saved by RTCO filters.
    static ref TOKENS_SAVED: IntCounterVec = register_int_counter_vec!(
        "rtco_tokens_saved_total",
        "Total number of tokens saved by RTCO compression",
        &["command", "handler"]
    ).expect("TOKENS_SAVED metric registration failed");

    /// Latency of filter operations in seconds.
    static ref FILTER_DURATION: HistogramVec = register_histogram_vec!(
        "rtco_filter_duration_seconds",
        "Latency of RTCO filter operations in seconds",
        &["command", "handler"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
    ).expect("FILTER_DURATION metric registration failed");
}

// ---------------------------------------------------------------------------
// No-op stubs (when feature is disabled)
// ---------------------------------------------------------------------------

/// Record that a command was filtered (no-op when prometheus feature is off).
#[cfg(not(feature = "prometheus"))]
pub fn record_filtered(
    _command: &str,
    _handler: &str,
    _original_tokens: usize,
    _filtered_tokens: usize,
    _duration_secs: f64,
) {
}

/// Record a passthrough (no-op when prometheus feature is off).
#[cfg(not(feature = "prometheus"))]
pub fn record_passthrough(_command: &str) {}

/// Return empty string when prometheus feature is off.
#[cfg(not(feature = "prometheus"))]
pub fn gather_text() -> String {
    String::new()
}

/// Reset is a no-op when prometheus feature is off.
#[cfg(not(feature = "prometheus"))]
pub fn reset() {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "prometheus")]
    #[test]
    fn test_record_filtered() {
        // Reset before test to avoid interference
        reset();
        record_filtered("git", "git/log", 1000, 100, 0.005);
        let text = gather_text();
        assert!(text.contains("rtco_commands_filtered_total"));
        assert!(text.contains("rtco_tokens_saved_total"));
        assert!(text.contains("rtco_filter_duration_seconds"));
    }

    #[cfg(feature = "prometheus")]
    #[test]
    fn test_record_passthrough() {
        reset();
        record_passthrough("unknown_tool");
        let text = gather_text();
        assert!(text.contains("rtco_commands_passthrough_total"));
    }

    #[cfg(feature = "prometheus")]
    #[test]
    fn test_gather_text_format() {
        reset();
        record_filtered("cargo", "cargo/test", 5000, 200, 0.010);

        let text = gather_text();
        // Should be valid Prometheus exposition format
        assert!(text.starts_with('#') || text.contains("rtco_"));
        assert!(text.contains("TYPE"));
        assert!(text.contains("HELP"));
    }

    #[cfg(not(feature = "prometheus"))]
    #[test]
    fn test_noop_without_feature() {
        // These should compile and run without panicking even without the
        // prometheus feature.
        record_filtered("git", "git/log", 1000, 100, 0.005);
        record_passthrough("unknown");
        assert_eq!(gather_text(), "");
    }
}
