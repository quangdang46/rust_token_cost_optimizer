//! JSON array compression execution.
//!
//! Applies selected compression strategies to JSON arrays.
//! Ported from headroom's SmartCrusher.

use serde_json::Value;

use crate::compressors::{CompressionStrategy, CrushResult, CrushedField};
use crate::utils::count_tokens;

use super::classifier::classify_array;
use super::planner::{plan_compression, SmartCrusherConfig};

/// Maximum depth for recursive crushing of nested values.
const MAX_RECURSION_DEPTH: usize = 20;

/// Compress a JSON value using SmartCrusher strategies.
///
/// Returns the compressed JSON string. If the input is not valid JSON
/// or crushing fails, returns the original input unchanged.
pub fn compress_json(input: &str) -> String {
    match try_compress(input) {
        Ok(result) => result,
        Err(_) => input.to_string(),
    }
}

/// Compress JSON with detailed statistics.
///
/// Returns a `CrushResult` with compression stats alongside the output string.
pub fn compress_json_with_stats(input: &str) -> Result<(String, CrushResult), String> {
    try_compress_with_stats(input)
}

/// Internal compression — returns `Ok` on success, `Err` on failure.
fn try_compress(input: &str) -> Result<String, String> {
    try_compress_with_stats(input).map(|(output, _)| output)
}

fn try_compress_with_stats(input: &str) -> Result<(String, CrushResult), String> {
    let mut value: Value =
        serde_json::from_str(input).map_err(|e| format!("JSON parse error: {}", e))?;

    let config = SmartCrusherConfig::default();
    let original_tokens = count_tokens(input);

    let mut crushed_fields = Vec::new();
    crush_value(&mut value, &[], &config, &mut crushed_fields, 0);

    let output =
        serde_json::to_string_pretty(&value).map_err(|e| format!("JSON serialize error: {}", e))?;
    let compressed_tokens = count_tokens(&output);

    Ok((
        output,
        CrushResult {
            original_tokens,
            compressed_tokens,
            crushed_fields,
        },
    ))
}

/// Recursively crush a JSON value in-place.
fn crush_value(
    value: &mut Value,
    path: &[String],
    config: &SmartCrusherConfig,
    crushed: &mut Vec<CrushedField>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        return;
    }

    match value {
        Value::Array(arr) => {
            let field_path = path.to_vec();
            let original_len = arr.len();

            // Classify and plan
            let json_val = Value::Array(arr.clone());
            if let Some(analysis) = classify_array(&json_val, field_path.clone()) {
                let plan = plan_compression(&analysis, config);

                match &plan.strategy {
                    CompressionStrategy::None => {
                        // Still recurse into nested objects
                        for element in arr.iter_mut() {
                            crush_value(element, path, config, crushed, depth + 1);
                        }
                    }
                    CompressionStrategy::Skip => {
                        // Replace with empty array
                        arr.clear();
                        crushed.push(CrushedField {
                            field_path: path.join("."),
                            original_count: original_len,
                            compressed_count: 0,
                            strategy: plan.strategy,
                        });
                    }
                    CompressionStrategy::TopN(n) => {
                        let keep = (*n).min(arr.len());
                        let elements: Vec<Value> = arr.drain(..keep).collect();
                        for element in &elements {
                            let mut e = element.clone();
                            crush_value(&mut e, path, config, crushed, depth + 1);
                        }
                        *arr = elements;
                        crushed.push(CrushedField {
                            field_path: path.join("."),
                            original_count: original_len,
                            compressed_count: keep,
                            strategy: plan.strategy,
                        });
                    }
                    CompressionStrategy::Sample(frac) => {
                        let keep_count = ((arr.len() as f64) * frac).round() as usize;
                        let keep_count = keep_count.max(1).min(arr.len());
                        let sampled = sample_elements(arr, keep_count);
                        crushed.push(CrushedField {
                            field_path: path.join("."),
                            original_count: original_len,
                            compressed_count: sampled.len(),
                            strategy: plan.strategy,
                        });
                        *arr = sampled;
                    }
                    CompressionStrategy::ClusterSample(n) => {
                        let keep = (*n).min(arr.len());
                        let sampled = cluster_sample(arr, keep);
                        crushed.push(CrushedField {
                            field_path: path.join("."),
                            original_count: original_len,
                            compressed_count: sampled.len(),
                            strategy: plan.strategy,
                        });
                        *arr = sampled;
                    }
                }
            }
        }
        Value::Object(map) => {
            // Recurse into each field
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in &keys {
                if let Some(val) = map.get_mut(key) {
                    let mut new_path = path.to_vec();
                    new_path.push(key.clone());
                    crush_value(val, &new_path, config, crushed, depth + 1);
                }
            }
        }
        _ => {} // Primitives — nothing to crush
    }
}

/// Sample elements from an array evenly.
fn sample_elements(arr: &[Value], keep: usize) -> Vec<Value> {
    if arr.len() <= keep {
        return arr.to_vec();
    }
    let step = arr.len() / keep;
    (0..keep)
        .filter_map(|i| arr.get(i * step))
        .cloned()
        .collect()
}

/// Stratified sampling by grouping similar elements.
///
/// Uses a simple hash-bucket approach: compute a string key for each element,
/// distribute into `keep` buckets, pick one from each.
fn cluster_sample(arr: &[Value], keep: usize) -> Vec<Value> {
    if arr.len() <= keep {
        return arr.to_vec();
    }

    let mut buckets: Vec<Vec<&Value>> = (0..keep).map(|_| Vec::new()).collect();

    for element in arr.iter() {
        let bucket = simple_hash(element) % keep;
        buckets[bucket].push(element);
    }

    buckets
        .iter()
        .filter_map(|b| b.first())
        .cloned()
        .cloned()
        .collect()
}

/// Simple hash of a JSON value for bucketing.
fn simple_hash(value: &Value) -> usize {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let key = match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(a) => format!("[{}]", a.len()),
        Value::Object(o) => {
            let keys: Vec<&String> = o.keys().collect();
            keys.iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(",")
        }
    };

    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_non_json_passthrough() {
        let input = "not valid json at all";
        assert_eq!(compress_json(input), input);
    }

    #[test]
    fn test_simple_object_no_crushable_arrays() {
        let input = r#"{"name": "test", "value": 42}"#;
        let output = compress_json(input);
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["name"], "test");
        assert_eq!(parsed["value"], 42);
    }

    #[test]
    fn test_large_dict_array_truncated() {
        let items: Vec<Value> = (0..100)
            .map(|i| json!({"id": i, "name": format!("item_{}", i)}))
            .collect();
        let input = serde_json::to_string(&json!({"results": items})).unwrap();
        let output = compress_json(&input);
        let parsed: Value = serde_json::from_str(&output).unwrap();
        let arr = parsed["results"].as_array().unwrap();
        assert!(
            arr.len() < 100,
            "Should have been truncated, got {}",
            arr.len()
        );
        assert!(
            arr.len() >= 10,
            "Should keep at least some elements: {}",
            arr.len()
        );
    }

    #[test]
    fn test_string_array_sampled() {
        let strings: Vec<Value> = (0..200).map(|i| json!(format!("item_{}", i))).collect();
        let input = serde_json::to_string(&json!({"data": strings})).unwrap();
        let output = compress_json(&input);
        let parsed: Value = serde_json::from_str(&output).unwrap();
        let arr = parsed["data"].as_array().unwrap();
        assert!(arr.len() < 200, "String array should be sampled");
        assert!(!arr.is_empty(), "Should keep at least 1 element");
    }

    #[test]
    fn test_compress_with_stats() {
        // Use pretty-printed input so whitespace token count is meaningful
        let items: Vec<Value> = (0..50).map(|i| json!({"id": i})).collect();
        let input = serde_json::to_string_pretty(&json!({"items": items})).unwrap();
        let (output, stats) = compress_json_with_stats(&input).unwrap();
        // Array of 50 dicts should be crushed to TopN(10) producing fewer tokens
        assert!(
            stats.original_tokens >= stats.compressed_tokens,
            "original={} compressed={}",
            stats.original_tokens,
            stats.compressed_tokens
        );
        assert!(!stats.crushed_fields.is_empty());
        assert!(!output.is_empty());
    }

    #[test]
    fn test_empty_array_skipped() {
        let input = r#"{"data": []}"#;
        let output = compress_json(input);
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["data"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_nested_object_crushing() {
        // NumberArray must be > 50 to trigger ClusterSample
        let items: Vec<i32> = (0..60).collect();
        let input = serde_json::to_string(&json!({
            "level1": {
                "level2": {
                    "large_array": items
                }
            }
        }))
        .unwrap();
        let output = compress_json(&input);
        let parsed: Value = serde_json::from_str(&output).unwrap();
        let arr = parsed["level1"]["level2"]["large_array"]
            .as_array()
            .unwrap();
        assert!(arr.len() < 60, "Nested array should be crushed");
    }

    #[test]
    fn test_count_tokens_approximate() {
        // Verify our tokenizer gives reasonable estimates
        let input = r#"{"a": [1, 2, 3, 4, 5]}"#;
        let tokens = crate::utils::count_tokens_with_tokenizer(input);
        // At minimum, should be > 0
        assert!(tokens > 0);
    }
}
