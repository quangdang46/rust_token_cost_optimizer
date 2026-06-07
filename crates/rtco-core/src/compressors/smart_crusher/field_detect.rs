//! Statistical detectors for ID-like and score-like fields.
//!
//! Ported from headroom's `transforms/smart_crusher/field_detect.rs`.
//! Detects whether a field carries meaningful ranking signals (score)
//! or is just a unique identifier (ID) that shouldn't drive compression.

use serde_json::Value;

/// Internal field stats required for statistical detection.
#[derive(Debug, Clone)]
pub struct DetectFieldStats {
    pub name: String,
    pub field_type: String,
    pub unique_ratio: f64,
    pub min_val: Option<f64>,
    pub max_val: Option<f64>,
}

/// Check if a string looks like a UUID (8-4-4-4-12 hex pattern).
fn is_uuid_format(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    let bytes = s.as_bytes();
    // xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    bytes.len() == 36
        && bytes[8] == b'-'
        && bytes[13] == b'-'
        && bytes[18] == b'-'
        && bytes[23] == b'-'
        && bytes.iter().all(|&b| b == b'-' || b.is_ascii_hexdigit())
}

/// Calculate entropy of a string (0.0–1.0 normalized).
fn calculate_string_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let len = s.len() as f64;
    let mut counts: std::collections::HashMap<char, usize> = std::collections::HashMap::new();
    for c in s.chars() {
        *counts.entry(c).or_insert(0) += 1;
    }
    let entropy: f64 = counts.values().fold(0.0, |acc, &c| {
        let p = c as f64 / len;
        if p > 0.0 {
            acc - p * p.log2()
        } else {
            acc
        }
    });
    // Normalize to 0.0–1.0 (max entropy = log2(distinct chars))
    let distinct = counts.len() as f64;
    if distinct <= 1.0 {
        0.0
    } else {
        entropy / distinct.log2()
    }
}

/// Detect if numeric values follow a sequential pattern.
fn detect_sequential_pattern(values: &[Value], _allow_gaps: bool) -> bool {
    let nums: Vec<f64> = values
        .iter()
        .filter_map(|v| v.as_f64())
        .filter(|f| f.is_finite())
        .collect();
    if nums.len() < 4 {
        return false;
    }

    // Check if values are monotonically increasing (with possible gaps)
    let increasing = nums.windows(2).all(|w| w[1] >= w[0]);
    if !increasing {
        return false;
    }

    // Check approximate arithmetic progression: diff should be relatively constant
    let diffs: Vec<f64> = nums.windows(2).map(|w| w[1] - w[0]).collect();
    if diffs.is_empty() {
        return true;
    }
    let avg_diff = diffs.iter().sum::<f64>() / diffs.len() as f64;
    if avg_diff <= 0.0 {
        return false;
    }

    // Allow small variance: most diffs should be within 2x of avg
    let tolerant_count = diffs
        .iter()
        .filter(|&&d| d >= avg_diff * 0.5 && d <= avg_diff * 2.0)
        .count();
    tolerant_count as f64 >= diffs.len() as f64 * 0.7
}

/// Detect whether a field is an ID field (high uniqueness, sequential or UUID).
pub fn detect_id_field_statistically(stats: &DetectFieldStats, values: &[Value]) -> (bool, f64) {
    if stats.unique_ratio < 0.9 {
        return (false, 0.0);
    }

    // String field detection
    if stats.field_type == "string" {
        let sample_values: Vec<&str> = values.iter().take(20).filter_map(|v| v.as_str()).collect();

        if !sample_values.is_empty() {
            let uuid_count = sample_values.iter().filter(|s| is_uuid_format(s)).count();
            if (uuid_count as f64 / sample_values.len() as f64) > 0.8 {
                return (true, 0.95);
            }

            let avg_entropy = sample_values
                .iter()
                .map(|s| calculate_string_entropy(s))
                .sum::<f64>()
                / sample_values.len() as f64;
            if avg_entropy > 0.7 && stats.unique_ratio > 0.95 {
                return (true, 0.8);
            }
        }
    }

    // Numeric field detection
    if stats.field_type == "numeric" {
        if detect_sequential_pattern(values, true) && stats.unique_ratio > 0.95 {
            return (true, 0.9);
        }

        if let (Some(min_v), Some(max_v)) = (stats.min_val, stats.max_val) {
            if (max_v - min_v) > 0.0 && stats.unique_ratio > 0.95 {
                return (true, 0.85);
            }
        }
    }

    if stats.unique_ratio > 0.98 {
        return (true, 0.7);
    }

    (false, 0.0)
}

/// Detect whether a field is a score field (bounded numeric, higher = more relevant).
pub fn detect_score_field_statistically(stats: &DetectFieldStats, items: &[Value]) -> (bool, f64) {
    if stats.field_type != "numeric" {
        return (false, 0.0);
    }

    let (min_val, max_val) = match (stats.min_val, stats.max_val) {
        (Some(mn), Some(mx)) => (mn, mx),
        _ => return (false, 0.0),
    };

    let mut confidence: f64 = 0.0;

    let is_bounded = if (0.0..=1.0).contains(&min_val) && (0.0..=1.0).contains(&max_val) {
        confidence += 0.4;
        true
    } else if (0.0..=10.0).contains(&min_val) && (0.0..=10.0).contains(&max_val) {
        confidence += 0.3;
        true
    } else if (0.0..=100.0).contains(&min_val) && (0.0..=100.0).contains(&max_val) {
        confidence += 0.25;
        true
    } else if min_val >= -1.0 && max_val <= 1.0 {
        confidence += 0.35;
        true
    } else {
        false
    };

    if !is_bounded {
        return (false, 0.0);
    }

    // Reject if sequential
    let sample_values: Vec<Value> = items
        .iter()
        .take(50)
        .filter_map(|item| item.as_object())
        .filter_map(|m| m.get(&stats.name))
        .cloned()
        .collect();

    if detect_sequential_pattern(&sample_values, true) {
        return (false, 0.0);
    }

    // Check descending sort (higher score = better)
    let values_in_order: Vec<f64> = items
        .iter()
        .filter_map(|item| item.as_object())
        .filter_map(|m| m.get(&stats.name))
        .filter_map(|v| v.as_f64())
        .filter(|f| f.is_finite())
        .collect();

    if values_in_order.len() >= 5 {
        let num_pairs = values_in_order.len() - 1;
        let descending_count = values_in_order.windows(2).filter(|w| w[0] >= w[1]).count();
        if num_pairs > 0 && (descending_count as f64 / num_pairs as f64) > 0.7 {
            confidence += 0.3;
        }
    }

    // Check for non-integer floats
    let first_20: &[f64] = if values_in_order.len() > 20 {
        &values_in_order[..20]
    } else {
        &values_in_order
    };
    let float_count = first_20
        .iter()
        .filter(|&&v| v.is_finite() && v != v.trunc())
        .count();
    if !first_20.is_empty() && (float_count as f64) > (first_20.len() as f64 * 0.3) {
        confidence += 0.1;
    }

    let is_score = confidence >= 0.4;
    (is_score, confidence.min(0.95))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_stats(name: &str, field_type: &str, unique_ratio: f64) -> DetectFieldStats {
        DetectFieldStats {
            name: name.to_string(),
            field_type: field_type.to_string(),
            unique_ratio,
            min_val: None,
            max_val: None,
        }
    }

    fn make_stats_range(name: &str, min_v: f64, max_v: f64) -> DetectFieldStats {
        DetectFieldStats {
            name: name.to_string(),
            field_type: "numeric".to_string(),
            unique_ratio: 1.0,
            min_val: Some(min_v),
            max_val: Some(max_v),
        }
    }

    mod id_field_tests {
        use super::*;

        #[test]
        fn test_low_uniqueness_rejected() {
            let s = make_stats("status", "string", 0.5);
            let values = vec![json!("ok"), json!("error"), json!("ok")];
            assert_eq!(detect_id_field_statistically(&s, &values), (false, 0.0));
        }

        #[test]
        fn test_uuid_strings_high_confidence() {
            let s = make_stats("uid", "string", 1.0);
            let values: Vec<Value> = (0..20)
                .map(|i| json!(format!("550e8400-e29b-41d4-a716-{:012x}", i)))
                .collect();
            let (is_id, conf) = detect_id_field_statistically(&s, &values);
            assert!(is_id);
            assert!((conf - 0.95).abs() < 1e-9);
        }

        #[test]
        fn test_high_entropy_strings() {
            let mut s = make_stats("uid", "string", 1.0);
            s.unique_ratio = 0.96;
            let values: Vec<Value> = (0..20)
                .map(|i| json!(format!("a3f7b2c{:06x}d8e1f4a7", i)))
                .collect();
            let (is_id, conf) = detect_id_field_statistically(&s, &values);
            assert!(is_id);
            assert!((conf - 0.8).abs() < 1e-9);
        }

        #[test]
        fn test_sequential_numeric() {
            let mut s = make_stats("id", "numeric", 1.0);
            s.unique_ratio = 0.96;
            s.min_val = Some(1.0);
            s.max_val = Some(100.0);
            let values: Vec<Value> = (1..=100).map(|i| json!(i)).collect();
            let (is_id, conf) = detect_id_field_statistically(&s, &values);
            assert!(is_id);
            assert!((conf - 0.9).abs() < 1e-9);
        }

        #[test]
        fn test_high_uniqueness_catchall() {
            let s = make_stats("misc", "numeric", 0.99);
            let values: Vec<Value> = (0..100).map(|_| json!(0)).collect();
            let (is_id, conf) = detect_id_field_statistically(&s, &values);
            assert!(is_id);
            assert!((conf - 0.7).abs() < 1e-9);
        }
    }

    mod score_field_tests {
        use super::*;

        #[test]
        fn test_unit_range_descending() {
            let s = make_stats_range("score", 0.0, 1.0);
            let items: Vec<Value> = (0..10)
                .rev()
                .map(|i| json!({"score": (i as f64) / 10.0}))
                .collect();
            let (is_score, conf) = detect_score_field_statistically(&s, &items);
            assert!(is_score);
            assert!(conf >= 0.7);
            assert!(conf <= 0.95);
        }

        #[test]
        fn test_sequential_rejected() {
            let s = make_stats_range("score", 1.0, 10.0);
            let items: Vec<Value> = (1..=10).map(|i| json!({"score": i})).collect();
            let (is_score, _) = detect_score_field_statistically(&s, &items);
            assert!(!is_score);
        }

        #[test]
        fn test_unbounded_range_rejected() {
            let s = make_stats_range("metric", 0.0, 1000.0);
            let items: Vec<Value> = (0..10).map(|i| json!({"metric": i * 100})).collect();
            let (is_score, _) = detect_score_field_statistically(&s, &items);
            assert!(!is_score);
        }

        #[test]
        fn test_signed_similarity_range() {
            let s = make_stats_range("similarity", -0.9, 0.95);
            let items: Vec<Value> = (0..10)
                .rev()
                .map(|i| json!({"similarity": (i as f64) / 10.0 - 0.5}))
                .collect();
            let (is_score, _) = detect_score_field_statistically(&s, &items);
            assert!(is_score);
        }

        #[test]
        fn test_below_threshold_rejected() {
            let s = make_stats_range("metric", 0.0, 100.0);
            let items: Vec<Value> = vec![
                json!({"metric": 50}),
                json!({"metric": 10}),
                json!({"metric": 80}),
                json!({"metric": 20}),
                json!({"metric": 90}),
            ];
            let (is_score, _) = detect_score_field_statistically(&s, &items);
            assert!(!is_score);
        }

        #[test]
        fn test_non_numeric_rejected() {
            let s = make_stats("name", "string", 0.5);
            let items = vec![json!({"name": "alice"}), json!({"name": "bob"})];
            let (is_score, _) = detect_score_field_statistically(&s, &items);
            assert!(!is_score);
        }

        #[test]
        fn test_confidence_capped() {
            let s = make_stats_range("score", 0.0, 1.0);
            let items: Vec<Value> = (0..50)
                .rev()
                .map(|i| json!({"score": (i as f64) / 50.0}))
                .collect();
            let (_, conf) = detect_score_field_statistically(&s, &items);
            assert!(conf <= 0.95);
        }

        #[test]
        fn test_is_uuid_format() {
            assert!(is_uuid_format("550e8400-e29b-41d4-a716-446655440000"));
            assert!(!is_uuid_format("not-a-uuid"));
            assert!(!is_uuid_format(""));
        }

        #[test]
        fn test_entropy_calculation() {
            let e1 = calculate_string_entropy("aaaa"); // low entropy
            let e2 = calculate_string_entropy("abcd"); // higher entropy
            assert!(e2 > e1, "more varied chars should have higher entropy");
            assert_eq!(calculate_string_entropy(""), 0.0);
        }

        #[test]
        fn test_sequential_pattern() {
            let values: Vec<Value> = (1..=10).map(|i| json!(i)).collect();
            assert!(detect_sequential_pattern(&values, true));

            let scattered: Vec<Value> = vec![json!(1), json!(100), json!(2), json!(200)];
            assert!(!detect_sequential_pattern(&scattered, true));
        }
    }
}
