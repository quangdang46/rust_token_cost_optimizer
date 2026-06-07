//! Regex-based query anchor extraction from user text.
//!
//! Ported from headroom's `transforms/smart_crusher/anchors.rs`.
//! Extracts anchors (UUIDs, IDs, hostnames, quoted strings, emails)
//! from user queries and checks if JSON items match any anchor.

use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::LazyLock;

static UUID_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b")
        .expect("UUID_PATTERN")
});

static NUMERIC_ID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d{4,}\b").expect("NUMERIC_ID_PATTERN"));

static HOSTNAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[a-zA-Z0-9][-a-zA-Z0-9]*\.[a-zA-Z0-9][-a-zA-Z0-9]*(?:\.[a-zA-Z]{2,})?\b")
        .expect("HOSTNAME_PATTERN")
});

static QUOTED_STRING_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"['"]([^'"]{1,50})['"]"#).expect("QUOTED_STRING_PATTERN"));

static EMAIL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").expect("EMAIL_PATTERN")
});

const HOSTNAME_FALSE_POSITIVES: &[&str] = &["e.g", "i.e", "etc."];

/// Extract query anchors from user text.
///
/// Returns a set of lowercased anchor strings.
pub fn extract_query_anchors(text: &str) -> HashSet<String> {
    let mut anchors = HashSet::new();
    if text.is_empty() {
        return anchors;
    }

    // UUIDs
    for m in UUID_PATTERN.find_iter(text) {
        anchors.insert(m.as_str().to_lowercase());
    }

    // Numeric IDs (4+ digits)
    for m in NUMERIC_ID_PATTERN.find_iter(text) {
        anchors.insert(m.as_str().to_string());
    }

    // Hostnames
    for m in HOSTNAME_PATTERN.find_iter(text) {
        let lc = m.as_str().to_lowercase();
        if !HOSTNAME_FALSE_POSITIVES.contains(&lc.as_str()) {
            anchors.insert(lc);
        }
    }

    // Quoted strings
    for caps in QUOTED_STRING_PATTERN.captures_iter(text) {
        if let Some(inner) = caps.get(1) {
            if inner.as_str().trim().len() >= 2 {
                anchors.insert(inner.as_str().to_lowercase());
            }
        }
    }

    // Emails
    for m in EMAIL_PATTERN.find_iter(text) {
        anchors.insert(m.as_str().to_lowercase());
    }

    anchors
}

/// Serialize a serde_json::Value matching Python's str() representation.
///
/// Python str(dict) differs from json.dumps():
/// - Single quotes instead of double
/// - True/False/None instead of true/false/null
/// - Spaces after commas and colons (`, `, `: `)
fn python_repr(value: &Value) -> String {
    let mut out = String::new();
    write_python_repr(&mut out, value);
    out
}

fn write_python_repr(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("None"),
        Value::Bool(true) => out.push_str("True"),
        Value::Bool(false) => out.push_str("False"),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => {
            out.push('\'');
            out.push_str(s);
            out.push('\'');
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_python_repr(out, item);
            }
            out.push(']');
        }
        Value::Object(map) => {
            out.push('{');
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push('\'');
                out.push_str(k);
                out.push('\'');
                out.push_str(": ");
                write_python_repr(out, v);
            }
            out.push('}');
        }
    }
}

/// Check if a JSON value matches any query anchors.
///
/// Uses python_repr() to match Python's str() behavior for parity.
pub fn item_matches_anchors(item: &Value, anchors: &HashSet<String>) -> bool {
    if anchors.is_empty() {
        return false;
    }
    let item_str = python_repr(item).to_lowercase();
    anchors.iter().any(|a| item_str.contains(a))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_text_no_anchors() {
        assert!(extract_query_anchors("").is_empty());
    }

    #[test]
    fn test_extracts_uuid_lowercased() {
        let anchors = extract_query_anchors("see id 550E8400-E29B-41D4-A716-446655440000 plz");
        assert!(anchors.contains("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_extracts_numeric_id() {
        let anchors = extract_query_anchors("user 12345 reported issue");
        assert!(anchors.contains("12345"));
    }

    #[test]
    fn test_three_digit_not_anchor() {
        assert!(!extract_query_anchors("user 123 reported issue")
            .iter()
            .any(|a| a == "123"));
    }

    #[test]
    fn test_extracts_hostname() {
        let anchors = extract_query_anchors("connect to api.example.com asap");
        assert!(anchors.contains("api.example.com"));
    }

    #[test]
    fn test_hostname_false_positive_filtered() {
        let anchors = extract_query_anchors("test e.g endpoint");
        assert!(
            !anchors.contains("e.g"),
            "false positive should be filtered"
        );
    }

    #[test]
    fn test_extracts_quoted_string_double() {
        let anchors = extract_query_anchors(r#"find "user_name" field"#);
        assert!(anchors.contains("user_name"));
    }

    #[test]
    fn test_extracts_quoted_string_single() {
        let anchors = extract_query_anchors("find the 'user_name' field");
        assert!(anchors.contains("user_name"));
    }

    #[test]
    fn test_very_short_quoted_skipped() {
        let anchors = extract_query_anchors(r#"find "x" value"#);
        assert!(!anchors.contains("x"));
    }

    #[test]
    fn test_extracts_email() {
        let anchors = extract_query_anchors("contact USER@example.COM please");
        assert!(anchors.contains("user@example.com"));
    }

    #[test]
    fn test_item_matches_empty_set() {
        let empty = HashSet::new();
        assert!(!item_matches_anchors(&json!({"a": 1}), &empty));
    }

    #[test]
    fn test_item_matches_anchor_in_value() {
        let anchors: HashSet<String> = ["alice".to_string()].into_iter().collect();
        assert!(item_matches_anchors(&json!({"name": "Alice"}), &anchors));
    }

    #[test]
    fn test_item_matches_anchor_in_key() {
        let anchors: HashSet<String> = ["status".to_string()].into_iter().collect();
        assert!(item_matches_anchors(&json!({"status": "ok"}), &anchors));
    }

    #[test]
    fn test_item_no_match_unrelated() {
        let anchors: HashSet<String> = ["xyz123".to_string()].into_iter().collect();
        assert!(!item_matches_anchors(&json!({"a": "b"}), &anchors));
    }

    #[test]
    fn test_python_repr_dict() {
        let v = json!({"name": "Alice", "ok": true, "count": 5, "val": null});
        assert_eq!(
            python_repr(&v),
            "{'name': 'Alice', 'ok': True, 'count': 5, 'val': None}"
        );
    }

    #[test]
    fn test_python_repr_list() {
        let v = json!([1, 2, "abc", true]);
        assert_eq!(python_repr(&v), "[1, 2, 'abc', True]");
    }

    #[test]
    fn test_python_repr_nested() {
        let v = json!({"a": [1, {"b": "c"}]});
        assert_eq!(python_repr(&v), "{'a': [1, {'b': 'c'}]}");
    }

    #[test]
    fn test_item_matches_with_python_none() {
        let anchors: HashSet<String> = ["none".to_string()].into_iter().collect();
        assert!(item_matches_anchors(&json!({"val": null}), &anchors));
    }

    #[test]
    fn test_item_avoids_json_null_token() {
        let anchors: HashSet<String> = ["null".to_string()].into_iter().collect();
        assert!(!item_matches_anchors(&json!({"val": null}), &anchors));
    }
}
