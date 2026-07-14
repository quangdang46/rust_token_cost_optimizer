//! Building blocks shared across all RTK modules.

pub mod adaptive_sizer;
pub mod anchor;
pub mod args_utils;
pub mod cache_aligner;
pub mod ccr;
pub mod compressors;
pub mod config;
pub mod constants;
pub mod content_detector;
pub mod content_router;
pub mod dedup;
pub mod display_helpers;
pub mod filter;
pub mod guard;
pub mod keyword_detector;
pub mod line_scorer;
pub mod metrics;
pub mod pipeline;
pub mod redact;
pub mod runner;
pub mod signals;
pub mod stack_trace;
pub mod stream;
pub mod tee;
pub mod telemetry;
pub mod text_stats;
pub mod tokenizer;
pub mod tracking;
pub mod truncate;
pub mod utils;

/// Result of filtering a command output.
#[derive(Debug, Clone)]
pub struct FilteredOutput {
    /// The filtered (compressed) text
    pub text: String,
    /// Estimated original token count
    pub original_tokens: usize,
    /// Estimated filtered token count
    pub filtered_tokens: usize,
    /// Token savings percentage (0.0 - 100.0)
    pub savings_percent: f64,
    /// Compression markers describing what was compressed
    pub markers: Vec<CompressionMarker>,
}

/// Marker indicating what portion of output was compressed.
#[derive(Debug, Clone)]
pub struct CompressionMarker {
    pub kind: MarkerKind,
    pub count: usize,
    pub details: String,
}

/// Kinds of compression markers.
#[derive(Debug, Clone)]
pub enum MarkerKind {
    LinesOmitted,
    StackTrace,
    ErrorGroup,
    DuplicatesRemoved,
}

/// Content type of command output.
pub use content_detector::ContentType;

/// Filter command output through RTCO's pipeline.
///
/// Returns a `FilteredOutput` with the compressed text and metadata.
/// Falls back to passthrough (no compression) on error.
#[allow(unused_variables)]
pub fn filter_output(command: &str, raw_output: &str) -> FilteredOutput {
    // If output is small, skip compression entirely
    if raw_output.len() < 512 {
        let tokens = utils::count_tokens(raw_output);
        return FilteredOutput {
            text: raw_output.to_string(),
            original_tokens: tokens,
            filtered_tokens: tokens,
            savings_percent: 0.0,
            markers: Vec::new(),
        };
    }

    let original_tokens = utils::count_tokens(raw_output);
    let filtered_tokens = original_tokens; // placeholder: no-op filter
    let savings_pct = if original_tokens > 0 {
        (1.0 - filtered_tokens as f64 / original_tokens as f64) * 100.0
    } else {
        0.0
    };
    FilteredOutput {
        text: raw_output.to_string(),
        original_tokens,
        filtered_tokens,
        savings_percent: savings_pct,
        markers: Vec::new(),
    }
}

/// Check if RTCO has a filter for the given command/tool name.
pub fn has_filter(command: &str) -> bool {
    matches!(
        command,
        "git"
            | "gh"
            | "cargo"
            | "npm"
            | "pnpm"
            | "npx"
            | "yarn"
            | "pip"
            | "pip3"
            | "mvn"
            | "gradle"
            | "gradlew"
            | "dotnet"
            | "go"
            | "golangci-lint"
            | "rustup"
            | "just"
            | "make"
            | "docker"
            | "kubectl"
            | "psql"
            | "ls"
            | "tree"
            | "find"
            | "grep"
            | "cat"
            | "read"
            | "wc"
            | "du"
            | "df"
            | "env"
            | "ps"
            | "curl"
            | "wget"
            | "terraform"
            | "tofu"
            | "helm"
            | "ssh"
            | "ping"
    )
}

/// Detect content type from raw output.
pub fn detect_content_type(output: &str) -> ContentType {
    content_detector::detect_content_type(output)
}
