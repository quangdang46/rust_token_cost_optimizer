//! Tag protection — keep workflow XML-style tags out of compressors.
//!
//! LLM workflows carry XML-style markers (`<system-reminder>`,
//! `<tool_call>`, `<thinking>`, etc.) that downstream code parses as
//! structure. Compressors see them as droppable noise and silently
//! strip them. Protect tags swaps custom-tag spans for opaque
//! placeholders before compression, then restores them after.
//!
//! Standard HTML5 elements (`<div>`, `<p>`, `<span>`, …) are *not*
//! protected — those are not expected in CLI output.
//!
//! # Algorithm
//!
//! Single-pass tag-stack walker over the input bytes (no regex
//! backtracking, no O(n²) restart loop):
//!
//! 1. Scan forward for `<`. If the next bytes form a valid tag-open
//!    (`<name attr=…>` or `<name/>`), classify the tag name.
//! 2. HTML tag → emit verbatim, continue.
//! 3. Custom tag, self-closing → emit a placeholder, record the span.
//! 4. Custom tag, opening → push `(name, start_offset)` onto a stack.
//! 5. `</name>` matching the top of the stack → pop, emit a placeholder
//!    for the whole `<name>…</name>` span.
//! 6. Mismatched close → write the close tag verbatim and move on.
//!
//! Output is built incrementally with offset-based slicing — never
//! `result.replace(original, placeholder, 1)`, which silently
//! misbehaves when two identical custom-tag blocks appear in the same
//! input.

use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Protect custom tags in `text`, returning (cleaned, blocks).
///
/// * `compress_tagged_content = false` (default) — replace each entire
///   `<custom>…</custom>` span (including nested children) with a
///   single placeholder.
/// * `compress_tagged_content = true` — replace only the tag markers
///   (open and close emitted as separate placeholders) so the
///   compressor can squash content while the tag boundaries survive.
pub fn protect_tags(text: &str, compress_tagged_content: bool) -> (String, Vec<(String, String)>) {
    if text.is_empty() || !text.contains('<') {
        return (text.to_string(), Vec::new());
    }

    let (prefix, _salted) = pick_placeholder_prefix(text);
    let spans = identify_spans(text, compress_tagged_content);
    match emit_output(text, &spans, &prefix) {
        Some((cleaned, blocks)) => (cleaned, blocks),
        None => (text.to_string(), Vec::new()),
    }
}

/// Restore protected tag spans after the compressor ran on the
/// cleaned text.
///
/// If a placeholder went missing during compression (the compressor
/// stripped it), the wrap is **discarded**: the compressed text flows
/// downstream as-is and the original tag bytes are NOT re-injected.
pub fn restore_tags(text: &str, blocks: &[(String, String)]) -> String {
    if blocks.is_empty() {
        return text.to_string();
    }

    let mut result = text.to_string();
    for (placeholder, original) in blocks {
        if result.contains(placeholder.as_str()) {
            result = result.replace(placeholder.as_str(), original);
        }
        // Lost placeholder → silently discard (no orphan tag injection)
    }
    result
}

// ---------------------------------------------------------------------------
// HTML5 tag detection
// ---------------------------------------------------------------------------

const HTML5_TAGS: &[&str] = &[
    "html",
    "base",
    "head",
    "link",
    "meta",
    "style",
    "title",
    "body",
    "address",
    "article",
    "aside",
    "footer",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hgroup",
    "main",
    "nav",
    "section",
    "search",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "figcaption",
    "figure",
    "hr",
    "li",
    "menu",
    "ol",
    "p",
    "pre",
    "ul",
    "a",
    "abbr",
    "b",
    "bdi",
    "bdo",
    "br",
    "cite",
    "code",
    "data",
    "dfn",
    "em",
    "i",
    "kbd",
    "mark",
    "q",
    "rp",
    "rt",
    "ruby",
    "s",
    "samp",
    "small",
    "span",
    "strong",
    "sub",
    "sup",
    "time",
    "u",
    "var",
    "wbr",
    "area",
    "audio",
    "img",
    "map",
    "track",
    "video",
    "embed",
    "iframe",
    "object",
    "param",
    "picture",
    "portal",
    "source",
    "svg",
    "math",
    "canvas",
    "noscript",
    "script",
    "del",
    "ins",
    "caption",
    "col",
    "colgroup",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "button",
    "datalist",
    "fieldset",
    "form",
    "input",
    "label",
    "legend",
    "meter",
    "optgroup",
    "option",
    "output",
    "progress",
    "select",
    "textarea",
    "details",
    "dialog",
    "summary",
    "slot",
    "template",
];

fn known_html_tags() -> &'static HashSet<&'static str> {
    use std::sync::OnceLock;
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| HTML5_TAGS.iter().copied().collect())
}

fn is_known_html_tag(tag_name: &str) -> bool {
    let set = known_html_tags();
    if set.contains(tag_name) {
        return true;
    }
    if tag_name.bytes().any(|b| b.is_ascii_uppercase()) {
        let lower = tag_name.to_ascii_lowercase();
        return set.contains(lower.as_str());
    }
    false
}

// ---------------------------------------------------------------------------
// Placeholder prefix management
// ---------------------------------------------------------------------------

const DEFAULT_PREFIX: &str = "{{RTCO_TAG_";
const PLACEHOLDER_SUFFIX: &str = "}}";

fn pick_placeholder_prefix(text: &str) -> (String, bool) {
    if !text.contains(DEFAULT_PREFIX) {
        return (DEFAULT_PREFIX.to_string(), false);
    }
    for salt in 0u32..16 {
        let candidate = format!("{{{{RTCO_TAG_{salt}_");
        if !text.contains(&candidate) {
            return (candidate, true);
        }
    }
    // Ultimate fallback — extremely unlikely to collide
    static FALLBACK: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let prefix = FALLBACK
        .get_or_init(|| "{{RTCO_TAG_FALLBACK_a4f1c7e2_".to_string())
        .clone();
    (prefix, true)
}

// ---------------------------------------------------------------------------
// Tag parsing
// ---------------------------------------------------------------------------

struct OpenTag {
    name_lower: String,
    open_start: usize,
}

enum TagParse {
    Open {
        name_end: usize,
        tag_end: usize,
        is_self_closing: bool,
    },
    Close {
        name_end: usize,
        tag_end: usize,
    },
    NotTag,
}

fn parse_tag_at(bytes: &[u8], start: usize) -> TagParse {
    debug_assert!(bytes[start] == b'<');
    let mut i = start + 1;
    let n = bytes.len();
    if i >= n {
        return TagParse::NotTag;
    }

    let is_close = bytes[i] == b'/';
    if is_close {
        i += 1;
    }
    if i >= n {
        return TagParse::NotTag;
    }

    let name_start = i;
    if !is_name_start(bytes[i]) {
        return TagParse::NotTag;
    }
    i += 1;
    while i < n && is_name_cont(bytes[i]) {
        i += 1;
    }
    let name_end = i;
    if name_end == name_start {
        return TagParse::NotTag;
    }

    if is_close {
        while i < n && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= n || bytes[i] != b'>' {
            return TagParse::NotTag;
        }
        return TagParse::Close {
            name_end,
            tag_end: i + 1,
        };
    }

    // Opening tag or self-closing
    let mut self_closing = false;
    while i < n {
        match bytes[i] {
            b'>' => {
                return TagParse::Open {
                    name_end,
                    tag_end: i + 1,
                    is_self_closing: self_closing,
                };
            }
            b'/' => {
                self_closing = true;
                i += 1;
            }
            b'"' | b'\'' => {
                let quote = bytes[i];
                i += 1;
                while i < n && bytes[i] != quote {
                    i += 1;
                }
                if i >= n {
                    return TagParse::NotTag;
                }
                i += 1;
                self_closing = false;
            }
            _ => {
                if bytes[i].is_ascii_whitespace() {
                    self_closing = false;
                }
                i += 1;
            }
        }
    }
    TagParse::NotTag
}

fn is_name_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_name_cont(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b':')
}

// ---------------------------------------------------------------------------
// Span identification
// ---------------------------------------------------------------------------

struct Span {
    start: usize,
    end: usize,
}

fn identify_spans(text: &str, compress_tagged_content: bool) -> Vec<Span> {
    let bytes = text.as_bytes();
    let n = bytes.len();
    let mut spans: Vec<Span> = Vec::new();
    let mut stack: Vec<OpenTag> = Vec::new();

    let mut i = 0;
    while i < n {
        let b = bytes[i];
        if b != b'<' {
            // Fast-scan to next '<'
            i += memchr(b'<', &bytes[i..]).unwrap_or(n - i);
            continue;
        }

        match parse_tag_at(bytes, i) {
            TagParse::NotTag => {
                i += 1;
            }
            TagParse::Open {
                name_end,
                tag_end,
                is_self_closing,
            } => {
                let name = &text[i + 1..name_end];
                if is_known_html_tag(name) {
                    i = tag_end;
                    continue;
                }
                if is_self_closing {
                    spans.push(Span {
                        start: i,
                        end: tag_end,
                    });
                    i = tag_end;
                    continue;
                }
                if compress_tagged_content {
                    // Open-marker mode: push only the open tag as a span
                    spans.push(Span {
                        start: i,
                        end: tag_end,
                    });
                }
                stack.push(OpenTag {
                    name_lower: name.to_ascii_lowercase(),
                    open_start: i,
                });
                i = tag_end;
            }
            TagParse::Close { name_end, tag_end } => {
                let close_name = &text[i + 2..name_end];
                if is_known_html_tag(close_name) {
                    i = tag_end;
                    continue;
                }
                let close_name_lower = close_name.to_ascii_lowercase();
                let matching = stack
                    .iter()
                    .rposition(|open| open.name_lower == close_name_lower);

                match matching {
                    Some(stack_idx) => {
                        if compress_tagged_content {
                            // Close-marker mode: push only the close tag as a span
                            stack.truncate(stack_idx);
                            let _ = stack.pop();
                            spans.push(Span {
                                start: i,
                                end: tag_end,
                            });
                        } else {
                            // Block mode: replace entire <custom>...</custom> with one placeholder
                            let open_start = stack[stack_idx].open_start;
                            stack.truncate(stack_idx);
                            // Remove any intermediate spans nested inside this block
                            spans.retain(|s| s.start < open_start);
                            spans.push(Span {
                                start: open_start,
                                end: tag_end,
                            });
                        }
                        i = tag_end;
                    }
                    None => {
                        // Orphan close — emit verbatim
                        i = tag_end;
                    }
                }
            }
        }
    }

    // Unclosed tags at end of input are left as-is (emitted verbatim)
    spans
}

// ---------------------------------------------------------------------------
// Output emission
// ---------------------------------------------------------------------------

fn emit_output(
    text: &str,
    spans: &[Span],
    prefix: &str,
) -> Option<(String, Vec<(String, String)>)> {
    let mut out = String::with_capacity(text.len());
    let mut blocks: Vec<(String, String)> = Vec::new();
    let mut cursor: usize = 0;

    for (counter, span) in (0_u64..).zip(spans.iter()) {
        if span.start < cursor {
            return None; // Overlapping span — safety check
        }
        out.push_str(&text[cursor..span.start]);
        let placeholder = format!("{prefix}{counter}{PLACEHOLDER_SUFFIX}");
        let original = &text[span.start..span.end];
        blocks.push((placeholder.clone(), original.to_string()));
        out.push_str(&placeholder);
        cursor = span.end;
    }
    out.push_str(&text[cursor..]);
    Some((out, blocks))
}

// ---------------------------------------------------------------------------
// Local memchr (no external dep needed)
// ---------------------------------------------------------------------------

fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn protect(text: &str) -> (String, Vec<(String, String)>) {
        protect_tags(text, false)
    }

    #[test]
    fn passthrough_when_no_angle_bracket() {
        let (cleaned, blocks) = protect("Just plain text");
        assert_eq!(cleaned, "Just plain text");
        assert!(blocks.is_empty());
    }

    #[test]
    fn html_tags_emitted_verbatim() {
        let text = "<div>Some content</div>";
        let (cleaned, blocks) = protect(text);
        assert_eq!(cleaned, text);
        assert!(blocks.is_empty());
    }

    #[test]
    fn html_tag_check_case_insensitive() {
        assert!(is_known_html_tag("DIV"));
        assert!(is_known_html_tag("Span"));
        assert!(!is_known_html_tag("system-reminder"));
        assert!(!is_known_html_tag("EXTREMELY_IMPORTANT"));
    }

    #[test]
    fn custom_tag_replaced_with_placeholder() {
        let text = "Before <system-reminder>Important</system-reminder> After";
        let (cleaned, blocks) = protect(text);
        assert!(!cleaned.contains("<system-reminder>"));
        assert!(!cleaned.contains("Important"));
        assert!(cleaned.contains("Before"));
        assert!(cleaned.contains("After"));
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].1, "<system-reminder>Important</system-reminder>");
    }

    #[test]
    fn custom_tag_with_attributes() {
        let text = r#"Before <tool_call name="search" query="test">payload</tool_call> After"#;
        let (_cleaned, blocks) = protect(text);
        assert!(!_cleaned.contains("<tool_call"));
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].1.contains("tool_call"));
    }

    #[test]
    fn self_closing_custom_tag() {
        let text = "Text <marker/> more text";
        let (_, blocks) = protect(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].1, "<marker/>");
    }

    #[test]
    fn self_closing_html_tag_not_protected() {
        let text = "Text <br/> more <hr/> text";
        let (cleaned, blocks) = protect(text);
        assert_eq!(cleaned, text);
        assert!(blocks.is_empty());
    }

    #[test]
    fn nested_custom_tags_collapse_to_outer_span() {
        let text = "<outer><inner>deep</inner></outer>";
        let (cleaned, blocks) = protect(text);
        assert!(!cleaned.contains("<outer>"));
        assert!(!cleaned.contains("<inner>"));
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].1, "<outer><inner>deep</inner></outer>");
    }

    #[test]
    fn mixed_html_and_custom() {
        let text = "<div>HTML</div> <system-reminder>Rule</system-reminder> <p>HTML2</p>";
        let (cleaned, blocks) = protect(text);
        assert!(cleaned.contains("<div>"));
        assert!(cleaned.contains("<p>"));
        assert!(!cleaned.contains("<system-reminder>"));
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn real_workflow_tags() {
        let cases = [
            "<tool_call>search({query: 'test'})</tool_call>",
            "<thinking>Let me analyze this</thinking>",
            "<EXTREMELY_IMPORTANT>Never skip validation</EXTREMELY_IMPORTANT>",
            "<user-prompt-submit-hook>check perms</user-prompt-submit-hook>",
            "<system-reminder>Rules</system-reminder>",
            "<result>Success: 42 items</result>",
        ];
        for tag in cases {
            let text = format!("Before {tag} After");
            let (_, blocks) = protect(&text);
            assert_eq!(blocks.len(), 1, "failed to protect: {tag}");
            assert_eq!(blocks[0].1, tag);
        }
    }

    #[test]
    fn empty_input_returns_empty() {
        let (cleaned, blocks) = protect("");
        assert!(cleaned.is_empty());
        assert!(blocks.is_empty());
    }

    #[test]
    fn compress_tagged_content_true_emits_marker_placeholders() {
        let text = "Before <system-reminder>Compressible content</system-reminder> After";
        let (cleaned, blocks, ..) = {
            let (c, b) = protect_tags(text, true);
            (c, b)
        };
        assert!(!cleaned.contains("<system-reminder>"));
        assert!(!cleaned.contains("</system-reminder>"));
        assert!(cleaned.contains("Compressible content"));
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn restore_basic() {
        let original = "Before <system-reminder>Rule</system-reminder> After";
        let (cleaned, blocks) = protect_tags(original, false);
        let restored = restore_tags(&cleaned, &blocks);
        assert_eq!(restored, original);
    }

    #[test]
    fn restore_empty_blocks_passthrough() {
        assert_eq!(restore_tags("untouched", &[]), "untouched");
    }

    #[test]
    fn restore_lost_placeholder_discards_wrap() {
        let blocks = vec![("{{RTCO_TAG_0}}".to_string(), "<tag>data</tag>".to_string())];
        let compressed = "text without placeholder";
        let restored = restore_tags(compressed, &blocks);
        assert_eq!(restored, compressed);
        assert!(!restored.contains("<tag>"));
    }

    #[test]
    fn restore_partial_loss_keeps_present_drops_lost() {
        let blocks = vec![
            ("{{RTCO_TAG_0}}".to_string(), "<a>1</a>".to_string()),
            ("{{RTCO_TAG_1}}".to_string(), "<lost>x</lost>".to_string()),
        ];
        let compressed = "head {{RTCO_TAG_0}} tail";
        let restored = restore_tags(compressed, &blocks);
        assert_eq!(restored, "head <a>1</a> tail");
        assert!(!restored.contains("<lost"));
    }

    #[test]
    fn restore_roundtrip_preserves_content() {
        let original = "Start <system-reminder>Rule 1: validate</system-reminder> middle \
                        <tool_call>search(q='test')</tool_call> end";
        let (cleaned, blocks) = protect_tags(original, false);
        let restored = restore_tags(&cleaned, &blocks);
        assert_eq!(restored, original);
    }

    #[test]
    fn duplicate_blocks_get_distinct_placeholders() {
        let text = "<system-reminder>same</system-reminder> middle \
                    <system-reminder>same</system-reminder>";
        let (cleaned, blocks) = protect_tags(text, false);
        assert_eq!(blocks.len(), 2);
        assert!(!cleaned.contains("<system-reminder>"));
        assert_ne!(blocks[0].0, blocks[1].0);
        assert_eq!(restore_tags(&cleaned, &blocks), text);
    }

    #[test]
    fn handles_deeply_nested_custom_tags() {
        let depth = 60;
        let mut text = String::new();
        for _ in 0..depth {
            text.push_str("<lvl>");
        }
        text.push_str("core");
        for _ in 0..depth {
            text.push_str("</lvl>");
        }
        let (cleaned, blocks) = protect_tags(&text, false);
        assert!(!cleaned.contains("<lvl>"));
        assert_eq!(blocks.len(), 1);
        assert_eq!(restore_tags(&cleaned, &blocks), text);
    }

    #[test]
    fn orphan_close_tag_emitted_verbatim() {
        let text = "no opener </ghost> here";
        let (cleaned, blocks) = protect_tags(text, false);
        assert_eq!(blocks.len(), 0);
        assert!(cleaned.contains("</ghost>"));
    }

    #[test]
    fn orphan_open_tag_emitted_verbatim() {
        let text = "<ghost>dangling content with no close";
        let (cleaned, blocks) = protect_tags(text, false);
        assert!(blocks.is_empty());
        assert_eq!(cleaned, text);
    }

    #[test]
    fn malformed_lone_lt_emitted_verbatim() {
        let text = "if a < b then c";
        let (cleaned, blocks) = protect_tags(text, false);
        assert_eq!(cleaned, text);
        assert!(blocks.is_empty());
    }

    #[test]
    fn truncated_markers_do_not_panic() {
        for text in ["</", "<", "<a/", "<a", "<a /", "</a"] {
            let (cleaned, blocks) = protect_tags(text, false);
            assert_eq!(cleaned, text);
            assert!(blocks.is_empty());
        }
    }

    #[test]
    fn attribute_with_gt_inside_quotes() {
        let text = r#"<custom payload='a>b'>content</custom>"#;
        let (cleaned, _blocks) = protect_tags(text, false);
        assert!(!cleaned.contains("payload"));
    }

    #[test]
    fn html_close_inside_custom_block_does_not_pop_stack() {
        let text = "<custom>x</div> y</custom>";
        let (cleaned, blocks) = protect_tags(text, false);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].1, "<custom>x</div> y</custom>");
        assert!(!cleaned.contains("<custom>"));
    }
}
