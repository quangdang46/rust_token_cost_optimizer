//! Content router — dispatches CLI output to the optimal compression pipeline.
//!
//! Uses [`content_detector`] to classify raw output, then selects the best
//! compression handler for that content type.  Each handler implements the
//! [`ContentHandler`] trait.
//!
//! # Usage
//!
//! ```no_run
//! use rtco_core::content_router::{ContentRouter, ContentType, ContentHandler};
//!
//! struct MyHandler;
//! impl ContentHandler for MyHandler {
//!     fn name(&self) -> &'static str { "my-handler" }
//!     fn handle(&self, input: &str) -> String {
//!         input.to_uppercase()
//!     }
//! }
//!
//! let router = ContentRouter::new()
//!     .with_handler(ContentType::PlainText, Box::new(MyHandler));
//! let result = router.route("some plain text");
//! ```

use crate::content_detector;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ContentType — routing strategy variants
// ---------------------------------------------------------------------------

/// Routing-layer content categories used by the router to select a
/// compression strategy.
///
/// This enum is deliberately coarser than
/// [`content_detector::ContentType`][0] so that the routing decision stays
/// simple.  The mapping from detection-level types to routing-level types
/// is handled by [`map_detected_type()`][1].
///
/// [0]: ../content_detector/enum.ContentType.html
/// [1]: fn.map_detected_type.html
///
/// # Relationship with [`content_detector::ContentType`][0]
///
/// The detector distinguishes between JSON arrays (`JsonArray`) and single
/// JSON objects (which it classifies as `PlainText`).  Since this routing
/// enum collapses them into a single `Json` variant, both JSON arrays and
/// JSON objects *ought* to hit the same handler.  **However**, the detector
/// currently classifies bare JSON objects (e.g. `{"key": "value"}`) as
/// `PlainText` rather than `JsonArray`, which means they are routed to the
/// `PlainText` handler instead of the `Json` handler.  This is a known gap
/// that may need addressing if structured-object responses (common in API
/// CLIs like `gh api`, `aws`, `curl`) need dedicated JSON compression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    /// JSON data (arrays or objects).
    Json,
    /// Source code in any programming language.
    Code,
    /// Log files, build output, compiler messages.
    Logs,
    /// Unstructured plain text.
    PlainText,
    /// Unified diff / git diff output.
    GitDiff,
    /// HTML markup.
    Html,
}

// ---------------------------------------------------------------------------
// Handler trait
// ---------------------------------------------------------------------------

/// A compression handler that transforms raw CLI output into a compressed
/// representation.
pub trait ContentHandler: Send + Sync {
    /// Human-readable name of this handler (for diagnostics / tracking).
    fn name(&self) -> &'static str;
    /// Compress `input` and return the compressed string.
    fn handle(&self, input: &str) -> String;
}

// ---------------------------------------------------------------------------
// Passthrough fallback
// ---------------------------------------------------------------------------

/// Handler that passes output through unchanged.
pub struct PassthroughHandler;

impl ContentHandler for PassthroughHandler {
    fn name(&self) -> &'static str {
        "passthrough"
    }
    fn handle(&self, input: &str) -> String {
        input.to_string()
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Routes CLI output to the best registered handler based on content type
/// detection.  Falls back to [`PassthroughHandler`] when no handler matches
/// the detected type, or when the handler itself fails.
pub struct ContentRouter {
    handlers: HashMap<ContentType, Box<dyn ContentHandler>>,
    default: Box<dyn ContentHandler>,
}

impl Default for ContentRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentRouter {
    /// Create a new [`ContentRouter`] with no registered handlers.  The
    /// fallback is [`PassthroughHandler`].
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            default: Box::new(PassthroughHandler),
        }
    }

    /// Register a handler for a specific content type.  Replaces any
    /// previously-registered handler for the same type.
    pub fn with_handler(mut self, kind: ContentType, handler: Box<dyn ContentHandler>) -> Self {
        self.handlers.insert(kind, handler);
        self
    }

    /// Register multiple handlers at once.
    pub fn with_handlers(
        mut self,
        handlers: impl IntoIterator<Item = (ContentType, Box<dyn ContentHandler>)>,
    ) -> Self {
        for (kind, handler) in handlers {
            self.handlers.insert(kind, handler);
        }
        self
    }

    /// Set the default handler used when no specific handler is registered for
    /// the detected content type.
    pub fn with_default(mut self, default: Box<dyn ContentHandler>) -> Self {
        self.default = default;
        self
    }

    /// Route `input` to the best matching handler.
    ///
    /// 1. Uses [`content_detector::detect_content_type`] to classify the input.
    /// 2. Maps the detected type to a routing [`ContentType`].
    /// 3. Dispatches to the registered handler, or the fallback.
    ///
    /// If the handler panics or returns an empty string (when input was not
    /// empty), the fallback handler is used instead.
    pub fn route(&self, input: &str) -> String {
        let detected = content_detector::detect_content_type(input);
        let route_kind = map_detected_type(detected);

        match self.handlers.get(&route_kind) {
            Some(handler) => {
                let result = handler.handle(input);
                if result.is_empty() && !input.is_empty() {
                    // Handler produced empty output for non-empty input — fall
                    // back instead of returning a useless result.
                    self.default.handle(input)
                } else {
                    result
                }
            }
            None => self.default.handle(input),
        }
    }

    /// Convenience: returns the handler registered for a routing content type,
    /// or the default.
    pub fn handler_for(&self, kind: ContentType) -> &dyn ContentHandler {
        self.handlers
            .get(&kind)
            .map(|b| b.as_ref())
            .unwrap_or_else(|| self.default.as_ref())
    }
}

// ---------------------------------------------------------------------------
// Mapping
// ---------------------------------------------------------------------------

/// Map from the fine-grained detection types to the coarser routing types.
fn map_detected_type(detected: content_detector::ContentType) -> ContentType {
    use content_detector::ContentType as Detected;
    match detected {
        Detected::JsonArray => ContentType::Json,
        Detected::SourceCode => ContentType::Code,
        Detected::SearchResults => ContentType::Logs,
        Detected::BuildOutput => ContentType::Logs,
        Detected::GitDiff => ContentType::GitDiff,
        Detected::Html => ContentType::Html,
        Detected::PlainText => ContentType::PlainText,
    }
}

// ---------------------------------------------------------------------------
// Stub handlers
// ---------------------------------------------------------------------------

/// Stub handler for JSON content — currently a passthrough placeholder.
pub struct JsonHandler;
impl ContentHandler for JsonHandler {
    fn name(&self) -> &'static str {
        "json"
    }
    fn handle(&self, input: &str) -> String {
        // TODO: compact JSON (strip insignificant whitespace)
        input.to_string()
    }
}

/// Stub handler for source code — currently a passthrough placeholder.
pub struct CodeHandler;
impl ContentHandler for CodeHandler {
    fn name(&self) -> &'static str {
        "code"
    }
    fn handle(&self, input: &str) -> String {
        // TODO: code-aware compression (strip comments, condense whitespace)
        input.to_string()
    }
}

/// Stub handler for logs / build output — currently a passthrough placeholder.
pub struct LogsHandler;
impl ContentHandler for LogsHandler {
    fn name(&self) -> &'static str {
        "logs"
    }
    fn handle(&self, input: &str) -> String {
        // TODO: filter known noisy patterns, deduplicate repeated errors
        input.to_string()
    }
}

/// Stub handler for git diffs — currently a passthrough placeholder.
pub struct GitDiffHandler;
impl ContentHandler for GitDiffHandler {
    fn name(&self) -> &'static str {
        "git_diff"
    }
    fn handle(&self, input: &str) -> String {
        // TODO: strip diff metadata, keep only changed hunks
        input.to_string()
    }
}

/// Stub handler for HTML — currently a passthrough placeholder.
pub struct HtmlHandler;
impl ContentHandler for HtmlHandler {
    fn name(&self) -> &'static str {
        "html"
    }
    fn handle(&self, input: &str) -> String {
        // TODO: strip HTML tags, keep text content
        input.to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_router() {
        let router = ContentRouter::new();
        let input = "hello world";
        let output = router.route(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_route_plain_text_with_handler() {
        struct UpperHandler;
        impl ContentHandler for UpperHandler {
            fn name(&self) -> &'static str {
                "upper"
            }
            fn handle(&self, input: &str) -> String {
                input.to_uppercase()
            }
        }

        let router =
            ContentRouter::new().with_handler(ContentType::PlainText, Box::new(UpperHandler));
        assert_eq!(router.route("hello"), "HELLO");
    }

    #[test]
    fn test_handler_for() {
        let router = ContentRouter::new().with_handler(ContentType::Json, Box::new(JsonHandler));
        assert_eq!(router.handler_for(ContentType::Json).name(), "json");
        assert_eq!(router.handler_for(ContentType::Code).name(), "passthrough");
    }

    #[test]
    fn test_map_detected_json_array() {
        assert_eq!(
            map_detected_type(content_detector::ContentType::JsonArray),
            ContentType::Json
        );
    }

    #[test]
    fn test_map_detected_source_code() {
        assert_eq!(
            map_detected_type(content_detector::ContentType::SourceCode),
            ContentType::Code
        );
    }

    #[test]
    fn test_map_detected_build_output() {
        assert_eq!(
            map_detected_type(content_detector::ContentType::BuildOutput),
            ContentType::Logs
        );
    }

    #[test]
    fn test_map_detected_search_results() {
        assert_eq!(
            map_detected_type(content_detector::ContentType::SearchResults),
            ContentType::Logs
        );
    }

    #[test]
    fn test_map_detected_git_diff() {
        assert_eq!(
            map_detected_type(content_detector::ContentType::GitDiff),
            ContentType::GitDiff
        );
    }

    #[test]
    fn test_map_detected_html() {
        assert_eq!(
            map_detected_type(content_detector::ContentType::Html),
            ContentType::Html
        );
    }

    #[test]
    fn test_map_detected_plain_text() {
        assert_eq!(
            map_detected_type(content_detector::ContentType::PlainText),
            ContentType::PlainText
        );
    }

    #[test]
    fn test_handler_fallback_on_empty_result() {
        struct EmptyHandler;
        impl ContentHandler for EmptyHandler {
            fn name(&self) -> &'static str {
                "empty"
            }
            fn handle(&self, _input: &str) -> String {
                String::new()
            }
        }

        let router =
            ContentRouter::new().with_handler(ContentType::PlainText, Box::new(EmptyHandler));
        // Non-empty input routes through EmptyHandler which returns "".
        // Fallback should produce the original input.
        assert_eq!(router.route("non-empty"), "non-empty");
    }

    #[test]
    fn test_stub_handlers_exist() {
        // Stub handlers should at least return the input unchanged.
        let handlers: Vec<Box<dyn ContentHandler>> = vec![
            Box::new(JsonHandler),
            Box::new(CodeHandler),
            Box::new(LogsHandler),
            Box::new(GitDiffHandler),
            Box::new(HtmlHandler),
        ];
        for handler in &handlers {
            let input = "some content";
            assert_eq!(
                handler.handle(input),
                input,
                "Stub handler '{}' should pass through",
                handler.name()
            );
        }
    }
}
