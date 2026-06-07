//! Stack trace detection across multiple programming languages.
//!
//! Scans a slice of lines and identifies contiguous regions that look like
//! stack traces from Python, JavaScript/Node, Rust, Java, Go, or unknown
//! formats. Each detected region is returned as a [`StackTrace`] with its
//! language, line range, and the individual frames.
//!
//! # Usage
//!
//! ```
//! use rtco_core::stack_trace::{detect_stack_traces, StackLanguage};
//!
//! let lines = vec![
//!     "Traceback (most recent call last):",
//!     "  File \"main.py\", line 10, in <module>",
//!     "    foo()",
//!     "  File \"main.py\", line 5, in foo",
//!     "    bar()",
//!     "KeyError: 'missing'",
//!     "done",
//! ];
//! let traces = detect_stack_traces(&lines);
//! assert_eq!(traces.len(), 1);
//! assert_eq!(traces[0].language, StackLanguage::Python);
//! assert_eq!(traces[0].frames.len(), 6);
//! ```

use lazy_static::lazy_static;
use regex::Regex;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Programming language a stack trace originates from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackLanguage {
    Python,
    JavaScript,
    Rust,
    Java,
    Go,
    Unknown,
}

/// A detected contiguous stack trace region within a larger output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackTrace {
    /// Language that produced this trace.
    pub language: StackLanguage,
    /// Zero-based index of the first line belonging to this trace.
    pub start_line: usize,
    /// Zero-based index of the last line belonging to this trace (inclusive).
    pub end_line: usize,
    /// The raw text of each frame / line in the trace, in order.
    pub frames: Vec<String>,
}

// ---------------------------------------------------------------------------
// Lazily compiled regexes
// ---------------------------------------------------------------------------

lazy_static! {
    // -- Python --
    static ref PY_TRACEBACK_HEADER: Regex =
        Regex::new(r"^Traceback \(most recent call last\):").unwrap();
    static ref PY_FILE_LINE: Regex =
        Regex::new(r#"^\s+File ""#).unwrap();

    /// Exception chain message (Python 3 chained exceptions).
    /// e.g. "During handling of the above exception, another exception occurred:"
    ///      "The above exception was the direct cause of the following exception:"
    static ref PY_CHAIN_MSG: Regex =
        Regex::new(r"^(During handling of the above exception|The above exception was)").unwrap();
    static ref JS_AT_FRAME: Regex =
        Regex::new(r"^\s+at\s+").unwrap();
    static ref JS_ERROR_LINE: Regex =
        Regex::new(r"^(TypeError|ReferenceError|SyntaxError|RangeError|Error|URIError|EvalError)\b").unwrap();

    // -- Rust --
    static ref RUST_LOCATION: Regex =
        Regex::new(r"^\s*-->\s+\S+:\d+").unwrap();
    static ref RUST_PANIC_LINE: Regex =
        Regex::new(r"thread '.+' panicked at").unwrap();
    static ref RUST_AT_LOCATION: Regex =
        Regex::new(r"^\s+at\s+\S+\.rs:\d+").unwrap();
    // Numbered backtrace frames like "   0: std::panicking::begin_panic"
    static ref RUST_BACKTRACE_FRAME: Regex =
        Regex::new(r"^\s+\d+:\s+\S+").unwrap();
    // Compiler annotation lines like "    |" or " 10 | code"
    static ref RUST_ANNOTATION: Regex =
        Regex::new(r"^\s*\d+\s*\||^\s*\|").unwrap();

    // -- Java --
    // Java "at" frames always contain a class.method(File.java:NN) pattern.
    // We check Java before JS because both use "at " prefix.
    // The method name may contain angle brackets (e.g. <init>, <clinit>).
    static ref JAVA_AT_FRAME: Regex =
        Regex::new(r"^\s+at\s+[\w$.<>]+\([\w$.<>]+\.java:\d+\)").unwrap();
    static ref JAVA_CAUSED_BY: Regex =
        Regex::new(r"^Caused by:\s+").unwrap();

    // -- Go --
    // "goroutine N [state]:" where state can be running, chan receive, select, etc.
    static ref GO_GOROUTINE: Regex =
        Regex::new(r"^goroutine\s+\d+\s+\[.+\]").unwrap();
    // Indented file.go:line +offset
    static ref GO_FILE_LINE: Regex =
        Regex::new(r"^\s+\S+\.go:\d+").unwrap();
    // Function calls like "main.foo()" or "main.foo(0x1, 0x2)"
    static ref GO_FUNC_CALL: Regex =
        Regex::new(r"^\w[\w.]*\(.*\)$").unwrap();
}

// ---------------------------------------------------------------------------
// Per-language frame classification
// ---------------------------------------------------------------------------

/// Classify a single line as belonging to a JavaScript / Node stack trace.
///
/// Note: Java frames are checked first since both use the "at " prefix.
fn is_js_line(line: &str) -> bool {
    JS_AT_FRAME.is_match(line) || JS_ERROR_LINE.is_match(line)
}

/// Classify a single line as belonging to a Java stack trace.
fn is_java_line(line: &str) -> bool {
    JAVA_AT_FRAME.is_match(line)
        || JAVA_CAUSED_BY.is_match(line)
        || line.starts_with("Exception in thread")
}

/// Classify a single line as belonging to a Rust stack trace.
fn is_rust_line(line: &str) -> bool {
    RUST_LOCATION.is_match(line)
        || RUST_PANIC_LINE.is_match(line)
        || RUST_AT_LOCATION.is_match(line)
        || RUST_BACKTRACE_FRAME.is_match(line)
        || RUST_ANNOTATION.is_match(line)
        || line.starts_with("thread '")
        || line == "stack backtrace:"
        || line.starts_with("note: run with `RUST_BACKTRACE=1`")
        || line.starts_with("note: run with `RUST_BACKTRACE=")
}

/// Classify a single line as belonging to a Go stack trace.
fn is_go_line(line: &str) -> bool {
    GO_GOROUTINE.is_match(line)
        || GO_FILE_LINE.is_match(line)
        || GO_FUNC_CALL.is_match(line)
        || line.starts_with("created by ")
}

/// Detect the language from a set of accumulated frame lines.
///
/// Priority order matters: Java must be checked before JS (both use "at ").
fn classify_language(frames: &[&str]) -> StackLanguage {
    for line in frames {
        if PY_TRACEBACK_HEADER.is_match(line) || PY_FILE_LINE.is_match(line) {
            return StackLanguage::Python;
        }
        if is_java_line(line) {
            // Java is checked before JS to disambiguate "at " lines.
            return StackLanguage::Java;
        }
        if is_rust_line(line) {
            return StackLanguage::Rust;
        }
        if is_go_line(line) {
            return StackLanguage::Go;
        }
        if is_js_line(line) {
            return StackLanguage::JavaScript;
        }
    }
    StackLanguage::Unknown
}

// ---------------------------------------------------------------------------
// Trace boundary detection
// ---------------------------------------------------------------------------

/// Determine whether `line` ends the current trace for `language`.
///
/// This is the entry gate: we only begin collecting frames when this returns
/// true.  Note that some languages (JS, Go) start with indented lines.
fn is_trace_start(line: &str) -> bool {
    // Python: unindented "Traceback (...):"
    PY_TRACEBACK_HEADER.is_match(line)
    // Java: "Exception in thread" or unindented "Caused by:"
    || line.starts_with("Exception in thread")
    || (JAVA_CAUSED_BY.is_match(line))
    // Rust: "thread '...' panicked at ..."
    || RUST_PANIC_LINE.is_match(line)
    // Rust: "--> file:line:col"
    || RUST_LOCATION.is_match(line)
    // Go: "goroutine N [state]:"
    || GO_GOROUTINE.is_match(line)
    // JS: indented "at ..." (JS traces often start with an error line that
    // doesn't match the "at" pattern, but the "at" line is the first
    // unambiguous marker)
    || JS_AT_FRAME.is_match(line)
    // JS error preamble without preceding "at" (rare but possible)
    || JS_ERROR_LINE.is_match(line)
    // Java "at" frame with .java:NN
    || JAVA_AT_FRAME.is_match(line)
}

/// Determine whether `line` ends the current trace for `language`.
///
/// Returns `true` when `line` should **not** be consumed into the trace
/// (i.e. the trace ended before this line).
fn is_trace_end(language: StackLanguage, line: &str) -> bool {
    let trimmed = line.trim();

    // Blank lines are tolerated inside all trace types.
    if trimmed.is_empty() {
        return false;
    }

    match language {
        // Python ends on a non-blank, non-indented line that is NOT the
        // traceback header and NOT a File line.  The exception summary
        // (e.g. "KeyError: 'x'") is included in the trace, then the trace
        // ends on the NEXT non-blank, non-indented line.
        // Exception chain messages (e.g. "During handling...") are also
        // kept as part of the trace.
        StackLanguage::Python => {
            if PY_TRACEBACK_HEADER.is_match(line)
                || PY_FILE_LINE.is_match(line)
                || PY_CHAIN_MSG.is_match(line)
            {
                return false;
            }
            if line.starts_with("  ") {
                return false;
            }
            // Exception summary lines (non-indented, contain ":" or Error/Exception)
            // are part of the trace.  We need to distinguish them from the next
            // section.  The trick: a Python exception summary always contains
            // a colon or the word Error/Exception.
            !trimmed.is_empty()
        }
        // JS ends at a line that is not an "at" frame and not an Error preamble.
        StackLanguage::JavaScript => !is_js_line(line),
        // Rust ends at a non-trace line.
        StackLanguage::Rust => {
            if is_rust_line(line) {
                return false;
            }
            // Compiler diagnostic annotations.
            if trimmed.starts_with("= note:") || trimmed.starts_with("note:") {
                return false;
            }
            if trimmed.starts_with("= help:") || trimmed.starts_with("help:") {
                return false;
            }
            if trimmed.starts_with('=') {
                return false;
            }
            true
        }
        // Java ends at a non-Java line.
        StackLanguage::Java => {
            if is_java_line(line) {
                return false;
            }
            // "... N more" ellipsis lines are part of Java traces.
            if trimmed.starts_with("...") && trimmed.ends_with("more)") {
                return false;
            }
            true
        }
        // Go ends at a non-Go line.  A new "goroutine N [state]:" header
        // also ends the current trace (each goroutine is a separate trace).
        StackLanguage::Go => !is_go_line(line) || GO_GOROUTINE.is_match(line),
        // Unknown: conservative.
        StackLanguage::Unknown => !trimmed.is_empty() && !line.starts_with("  "),
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Detect all stack traces in a slice of lines.
///
/// Scans the input looking for contiguous regions that match known stack trace
/// patterns. Each region is returned as a [`StackTrace`] with its detected
/// language, the start/end line indices, and the raw frame text.
///
/// Lines that do not belong to any stack trace are silently skipped.
///
/// # Examples
///
/// ```
/// use rtco_core::stack_trace::{detect_stack_traces, StackLanguage};
///
/// let lines = vec![
///     "Traceback (most recent call last):",
///     "  File \"app.py\", line 7, in <module>",
///     "    main()",
///     "  File \"app.py\", line 3, in main",
///     "    raise ValueError('oops')",
///     "ValueError: oops",
/// ];
/// let traces = detect_stack_traces(&lines);
/// assert_eq!(traces.len(), 1);
/// assert_eq!(traces[0].language, StackLanguage::Python);
/// ```
pub fn detect_stack_traces(lines: &[&str]) -> Vec<StackTrace> {
    let mut traces = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        // Look for the start of a stack trace.
        if !is_trace_start(lines[i]) {
            i += 1;
            continue;
        }

        let start = i;
        let mut frames: Vec<&str> = Vec::new();

        // Push the first line.
        frames.push(lines[i]);
        i += 1;

        // Classify language early so continuation checks are accurate.
        let mut language = classify_language(&frames);

        // Consume continuation lines.
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();

            // Blank lines are tolerated inside traces.
            if trimmed.is_empty() {
                frames.push(line);
                i += 1;
                continue;
            }

            // Check end-of-trace BEFORE consuming the line.
            if is_trace_end(language, line) {
                // For Python, a non-indented line with Error/Exception/: is
                // the exception summary — include it in the trace before
                // ending.
                if language == StackLanguage::Python
                    && !line.starts_with(' ')
                    && (trimmed.contains("Error")
                        || trimmed.contains("Exception")
                        || trimmed.contains(':'))
                {
                    frames.push(line);
                    i += 1;
                }
                break;
            }

            frames.push(line);
            i += 1;

            // Re-classify if we have more data (language might be refined).
            language = classify_language(&frames);
        }

        // Trim trailing blank lines.
        while frames.last().is_some_and(|f| f.trim().is_empty()) {
            frames.pop();
        }

        if !frames.is_empty() {
            let end = start + frames.len() - 1;
            traces.push(StackTrace {
                language,
                start_line: start,
                end_line: end,
                frames: frames.iter().map(|s| s.to_string()).collect(),
            });
        }
    }

    traces
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Python --

    #[test]
    fn python_basic_traceback() {
        let lines = vec![
            "Traceback (most recent call last):",
            "  File \"main.py\", line 10, in <module>",
            "    foo()",
            "  File \"main.py\", line 5, in foo",
            "    bar()",
            "KeyError: 'missing'",
            "done",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Python);
        assert_eq!(traces[0].start_line, 0);
        assert!(traces[0].frames.len() >= 4);
        // The KeyError line should be included in the trace.
        assert!(traces[0].frames.iter().any(|f| f.contains("KeyError")));
    }

    #[test]
    fn python_traceback_with_value_error() {
        let lines = vec![
            "Traceback (most recent call last):",
            "  File \"app.py\", line 7, in <module>",
            "    main()",
            "  File \"app.py\", line 3, in main",
            "    raise ValueError('oops')",
            "ValueError: oops",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Python);
        assert!(traces[0].frames.iter().any(|f| f.contains("ValueError")));
    }

    #[test]
    fn python_traceback_not_confused_with_plain_text() {
        let lines = vec!["Hello world", "Building project...", "Done."];
        let traces = detect_stack_traces(&lines);
        assert!(traces.is_empty());
    }

    // -- JavaScript --

    #[test]
    fn js_node_stack_trace() {
        let lines = vec![
            "TypeError: Cannot read properties of undefined",
            "    at Object.<anonymous> (/app/index.js:10:5)",
            "    at Module._compile (node:internal/modules/cjs/loader:1198:14)",
            "    at Object.Module._extensions..js (node:internal/modules/cjs/loader:1252:10)",
            "Server started",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::JavaScript);
        assert!(traces[0].frames.iter().any(|f| f.contains("at Object")));
    }

    #[test]
    fn js_at_frames_only() {
        let lines = vec![
            "    at foo (/app/foo.js:1:1)",
            "    at bar (/app/bar.js:2:2)",
            "next section",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::JavaScript);
    }

    // -- Rust --

    #[test]
    fn rust_panic_trace() {
        let lines = vec![
            "thread 'main' panicked at 'index out of bounds', src/main.rs:10:5",
            "stack backtrace:",
            "   0: std::panicking::begin_panic",
            "             at /rustc/abc123/library/std/src/panicking.rs:578",
            "   1: main::main",
            "             at ./src/main.rs:10",
            "note: run with `RUST_BACKTRACE=1` for a backtrace",
            "next section",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Rust);
        assert!(traces[0].frames.iter().any(|f| f.contains("panicked")));
    }

    #[test]
    fn rust_location_trace() {
        let lines = vec![
            "   --> src/main.rs:10:5",
            "    |",
            " 10 |     let x = vec![1, 2, 3];",
            "    |     ^^^^^^^^^^^^^^^^^^^",
            "",
            "error: aborting due to previous error",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Rust);
    }

    // -- Java --

    #[test]
    fn java_basic_trace() {
        let lines = vec![
            "Exception in thread \"main\" java.lang.NullPointerException",
            "    at com.example.App.main(App.java:15)",
            "    at com.example.Runner.run(Runner.java:42)",
            "Process finished with exit code 1",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Java);
        assert_eq!(traces[0].frames.len(), 3);
    }

    #[test]
    fn java_caused_by_chain() {
        let lines = vec![
            "Exception in thread \"main\" java.lang.RuntimeException: wrapper",
            "    at com.example.App.main(App.java:15)",
            "Caused by: java.io.FileNotFoundException: /tmp/missing.txt",
            "    at java.io.FileInputStream.<init>(FileInputStream.java:146)",
            "    at com.example.App.read(App.java:20)",
            "    ... 1 more)",
            "next",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Java);
        assert!(traces[0].frames.iter().any(|f| f.contains("Caused by")));
        assert!(traces[0].frames.iter().any(|f| f.contains("... 1 more")));
    }

    // -- Go --

    #[test]
    fn go_goroutine_trace() {
        let lines = vec![
            "goroutine 1 [running]:",
            "main.foo()",
            "        /app/main.go:10 +0x42",
            "main.bar()",
            "        /app/main.go:20 +0x84",
            "created by main.init",
            "        /app/main.go:5 +0x20",
            "",
            "exit status 2",
        ];
        // Debug: check trace start for each line
        for (idx, line) in lines.iter().enumerate() {
            eprintln!("line[{}]: is_trace_start={}", idx, is_trace_start(line));
        }
        eprintln!("---");
        let traces = detect_stack_traces(&lines);
        eprintln!("traces.len() = {}", traces.len());
        for (idx, t) in traces.iter().enumerate() {
            eprintln!(
                "trace[{}]: lang={:?}, start={}, end={}, frames={:?}",
                idx, t.language, t.start_line, t.end_line, t.frames
            );
        }
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Go);
        assert!(traces[0].frames.iter().any(|f| f.contains("goroutine")));
        assert!(traces[0].frames.iter().any(|f| f.contains("created by")));
    }

    #[test]
    fn go_multiple_goroutines() {
        let lines = vec![
            "goroutine 1 [running]:",
            "main.foo()",
            "        /app/main.go:10 +0x42",
            "",
            "goroutine 7 [chan receive]:",
            "main.listener()",
            "        /app/main.go:50 +0x20",
            "",
            "exit status 1",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 2);
        assert_eq!(traces[0].language, StackLanguage::Go);
        assert_eq!(traces[1].language, StackLanguage::Go);
    }

    #[test]
    fn python_chained_exceptions() {
        let lines = vec![
            "Traceback (most recent call last):",
            "  File \"app.py\", line 2, in <module>",
            "    process()",
            "  File \"app.py\", line 5, in process",
            "    raise ValueError('original')",
            "ValueError: original",
            "",
            "During handling of the above exception, another exception occurred:",
            "",
            "Traceback (most recent call last):",
            "  File \"app.py\", line 2, in <module>",
            "    process()",
            "  File \"app.py\", line 8, in process",
            "    raise RuntimeError('wrapped')",
            "RuntimeError: wrapped",
            "done",
        ];
        let traces = detect_stack_traces(&lines);
        // Should detect both traces (each is a separate Traceback)
        assert!(traces.len() >= 2, "expected 2 traces, got {}", traces.len());
        // All traces should be Python
        for t in &traces {
            assert_eq!(
                t.language,
                StackLanguage::Python,
                "trace should be Python: {:?}",
                t
            );
        }
        // The chain message is standalone text between traces,
        // not part of either trace.
        let has_chain_in_trace = traces
            .iter()
            .any(|t| t.frames.iter().any(|f| f.contains("During handling")));
        assert!(
            !has_chain_in_trace,
            "chain message should not be inside a trace"
        );
    }

    // -- Multiple traces in one output --

    #[test]
    fn multiple_traces_python_and_js() {
        let lines = vec![
            "Traceback (most recent call last):",
            "  File \"main.py\", line 2, in <module>",
            "    raise RuntimeError('boom')",
            "RuntimeError: boom",
            "---",
            "    at Object.<anonymous> (/app/index.js:10:5)",
            "    at Module._compile (node:internal/modules/cjs/loader:1198:14)",
            "done",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 2);
        assert_eq!(traces[0].language, StackLanguage::Python);
        assert_eq!(traces[1].language, StackLanguage::JavaScript);
    }

    // -- Edge cases --

    #[test]
    fn empty_input() {
        let traces = detect_stack_traces(&[]);
        assert!(traces.is_empty());
    }

    #[test]
    fn no_traces_plain_text() {
        let lines = vec![
            "Starting build...",
            "Compiling rtco v0.28.2",
            "Build succeeded.",
        ];
        let traces = detect_stack_traces(&lines);
        assert!(traces.is_empty());
    }

    #[test]
    fn single_line_trace() {
        let lines = vec![
            "normal output",
            "    at foo (/app/foo.js:1:1)",
            "more output",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].frames.len(), 1);
    }

    #[test]
    fn trace_at_end_of_input() {
        let lines = vec![
            "some output",
            "Traceback (most recent call last):",
            "  File \"main.py\", line 1, in <module>",
            "    boom()",
            "RuntimeError: kaboom",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Python);
        assert_eq!(traces[0].start_line, 1);
    }

    #[test]
    fn blank_lines_inside_trace_are_preserved() {
        let lines = vec![
            "goroutine 1 [running]:",
            "main.foo()",
            "",
            "        /app/main.go:10 +0x42",
            "exit",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Go);
    }

    #[test]
    fn start_and_end_line_indices_are_correct() {
        let lines = vec![
            "line 0",
            "line 1",
            "Traceback (most recent call last):",
            "  File \"x.py\", line 1, in <module>",
            "    boom()",
            "ValueError: nope",
            "line 6",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].start_line, 2);
        assert_eq!(traces[0].end_line, 5);
    }

    #[test]
    fn unicode_frames_do_not_panic() {
        let lines = vec![
            "Traceback (most recent call last):",
            "  File \"\u{65e5}\u{672c}\u{8a9e}.py\", line 1, in <module>",
            "    boom()",
            "ValueError: some unicode message here",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Python);
    }

    // -- Line count / token savings --

    #[test]
    fn trace_frames_match_line_count() {
        let lines = vec![
            "Traceback (most recent call last):",
            "  File \"main.py\", line 10, in <module>",
            "    foo()",
            "  File \"main.py\", line 5, in foo",
            "    bar()",
            "KeyError: 'x'",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(
            traces[0].end_line - traces[0].start_line + 1,
            traces[0].frames.len()
        );
    }

    // -- Go state variants --

    #[test]
    fn go_chan_receive_state() {
        let lines = vec![
            "goroutine 7 [chan receive]:",
            "main.listener()",
            "        /app/main.go:50 +0x20",
            "next",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Go);
        assert!(traces[0].frames.iter().any(|f| f.contains("chan receive")));
    }

    #[test]
    fn go_select_state() {
        let lines = vec![
            "goroutine 5 [select]:",
            "main.loop()",
            "        /app/main.go:30 +0x10",
            "done",
        ];
        let traces = detect_stack_traces(&lines);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].language, StackLanguage::Go);
    }
}
