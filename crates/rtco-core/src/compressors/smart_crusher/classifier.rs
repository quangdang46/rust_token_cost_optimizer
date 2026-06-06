//! JSON array classification logic.
//!
//! Classifies JSON arrays into types (DictArray, StringArray, etc.)
//! by inspecting their elements. Ported from headroom's SmartCrusher classifier.

use serde_json::Value;

use crate::compressors::{ArrayAnalysis, ArrayType, FieldStats};

/// Classify a JSON array and produce an analysis.
///
/// Returns `None` if the value is not an array.
pub fn classify_array(value: &Value, field_path: Vec<String>) -> Option<ArrayAnalysis> {
    let array = value.as_array()?;
    let array_length = array.len();

    if array_length == 0 {
        return Some(ArrayAnalysis {
            array_type: ArrayType::Empty,
            field_stats: FieldStats {
                field_path,
                array_length: 0,
                unique_values: None,
                estimated_token_savings: 0,
            },
            crushable: false,
        });
    }

    let array_type = determine_array_type(array);
    let unique_values = estimate_unique_values(array, &array_type);

    // Estimate token savings: approximate as (1 - keep_ratio) * array_length * avg_element_tokens
    let keep_ratio = match array_type.default_strategy(array_length) {
        crate::compressors::CompressionStrategy::None => 1.0,
        crate::compressors::CompressionStrategy::Skip => 0.0,
        crate::compressors::CompressionStrategy::TopN(n) => n as f64 / array_length as f64,
        crate::compressors::CompressionStrategy::Sample(f) => f,
        crate::compressors::CompressionStrategy::ClusterSample(n) => n as f64 / array_length as f64,
    };

    let avg_element_len = array
        .iter()
        .map(|v| serde_json::to_string(v).unwrap_or_default().len())
        .sum::<usize>()
        .max(1)
        / array_length;

    let estimated_savings =
        ((1.0 - keep_ratio) * array_length as f64 * avg_element_len as f64) as usize;

    let crushable = matches!(
        array_type,
        ArrayType::DictArray
            | ArrayType::StringArray
            | ArrayType::NumberArray
            | ArrayType::NestedArray
    );

    Some(ArrayAnalysis {
        array_type,
        field_stats: FieldStats {
            field_path,
            array_length,
            unique_values,
            estimated_token_savings: estimated_savings,
        },
        crushable,
    })
}

/// Determine the `ArrayType` of a JSON array by inspecting its first
/// `SAMPLE_SIZE` elements.
fn determine_array_type(array: &[Value]) -> ArrayType {
    const SAMPLE_SIZE: usize = 100;
    let sample = if array.len() <= SAMPLE_SIZE {
        array
    } else {
        &array[..SAMPLE_SIZE]
    };

    if sample.is_empty() {
        return ArrayType::Empty;
    }

    let mut has_obj = false;
    let mut has_string = false;
    let mut has_number = false;
    let mut has_bool = false;
    let mut has_array = false;
    let mut has_null = false;

    for element in sample {
        match element {
            Value::Object(_) => has_obj = true,
            Value::String(_) => has_string = true,
            Value::Number(_) => has_number = true,
            Value::Bool(_) => has_bool = true,
            Value::Array(_) => has_array = true,
            Value::Null => has_null = true,
        }
    }

    let types_found = [has_obj, has_string, has_number, has_bool, has_array]
        .iter()
        .filter(|&&x| x)
        .count();

    if has_null && types_found == 0 {
        // All nulls
        ArrayType::MixedArray
    } else if types_found == 1 {
        if has_obj {
            ArrayType::DictArray
        } else if has_string {
            ArrayType::StringArray
        } else if has_number {
            ArrayType::NumberArray
        } else if has_bool {
            ArrayType::BoolArray
        } else if has_array {
            ArrayType::NestedArray
        } else {
            ArrayType::MixedArray
        }
    } else {
        ArrayType::MixedArray
    }
}

/// Estimate unique values in the array by sampling.
fn estimate_unique_values(array: &[Value], array_type: &ArrayType) -> Option<usize> {
    const UNIQUE_SAMPLE: usize = 50;
    let sample: Vec<&Value> = if array.len() <= UNIQUE_SAMPLE {
        array.iter().collect()
    } else {
        let step = array.len() / UNIQUE_SAMPLE;
        (0..UNIQUE_SAMPLE)
            .filter_map(|i| array.get(i * step))
            .collect()
    };

    use std::collections::HashSet;
    let mut seen = HashSet::new();

    for element in sample {
        let key = match (array_type, element) {
            (_, Value::String(s)) => s.as_str().to_string(),
            (_, Value::Number(n)) => n.to_string(),
            (ArrayType::DictArray, Value::Object(m)) => {
                // Use first key as fingerprint
                m.keys().next().unwrap_or(&"".to_string()).clone()
            }
            _ => continue,
        };
        seen.insert(key);
    }

    if seen.is_empty() {
        None
    } else {
        Some(seen.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_classify_empty_array() {
        let result = classify_array(&json!([]), vec!["empty".into()]).unwrap();
        assert_eq!(result.array_type, ArrayType::Empty);
        assert!(!result.crushable);
    }

    #[test]
    fn test_classify_dict_array() {
        let arr = json!([
            {"name": "alice", "age": 30},
            {"name": "bob", "age": 25},
            {"name": "charlie", "age": 35},
        ]);
        let result = classify_array(&arr, vec!["users".into()]).unwrap();
        assert_eq!(result.array_type, ArrayType::DictArray);
        assert!(result.crushable);
    }

    #[test]
    fn test_classify_string_array() {
        let arr = json!(["apple", "banana", "cherry"]);
        let result = classify_array(&arr, vec!["fruits".into()]).unwrap();
        assert_eq!(result.array_type, ArrayType::StringArray);
    }

    #[test]
    fn test_classify_number_array() {
        let arr = json!([1, 2, 3, 4, 5]);
        let result = classify_array(&arr, vec!["scores".into()]).unwrap();
        assert_eq!(result.array_type, ArrayType::NumberArray);
    }

    #[test]
    fn test_classify_mixed_array() {
        let arr = json!([1, "hello", true, {"key": "val"}]);
        let result = classify_array(&arr, vec!["mixed".into()]).unwrap();
        assert_eq!(result.array_type, ArrayType::MixedArray);
    }

    #[test]
    fn test_non_array_returns_none() {
        let result = classify_array(&json!("not an array"), vec![]);
        assert!(result.is_none());
    }

    #[test]
    fn test_field_stats_populated() {
        let arr = json!([{"a": 1}, {"a": 2}]);
        let result = classify_array(&arr, vec!["data".to_string(), "items".into()]).unwrap();
        assert_eq!(result.field_stats.field_path, vec!["data", "items"]);
        assert_eq!(result.field_stats.array_length, 2);
    }
}
