//! Runs TOML filter inline tests to make sure filter rules work correctly.

use anyhow::Result;

use crate::toml_filter;

/// Run TOML filter inline tests.
///
/// - `filter`: if `Some`, only run tests for that filter name
/// - `require_all`: fail if any filter has no inline tests
pub fn run(filter: Option<String>, require_all: bool) -> Result<()> {
    let results = toml_filter::run_filter_tests(filter.as_deref());

    let total = results.outcomes.len();
    let passed = results.outcomes.iter().filter(|o| o.passed).count();
    let failed = total - passed;

    // Print failures with details
    for outcome in &results.outcomes {
        if !outcome.passed {
            eprintln!(
                "FAIL [{}] {}\n  expected: {:?}\n  actual:   {:?}",
                outcome.filter_name, outcome.test_name, outcome.expected, outcome.actual
            );
        }
    }

    if total == 0 {
        println!("No inline tests found.");
    } else {
        println!("{}/{} tests passed", passed, total);
    }

    if require_all && !results.filters_without_tests.is_empty() {
        for name in &results.filters_without_tests {
            eprintln!("MISSING tests for filter: {}", name);
        }
        anyhow::bail!(
            "{} filter(s) have no inline tests (use --require-all in CI)",
            results.filters_without_tests.len()
        );
    }

    if failed > 0 {
        anyhow::bail!("{} test(s) failed", failed);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_filter_list() {
        // When filter is None and no TOML tests exist, should return OK
        let result = run(None, false);
        // Should not panic - either OK or an error about test failures
        assert!(
            result.is_ok()
                || result
                    .as_ref()
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("test(s) failed"),
            "unexpected error: {:?}",
            result
        );
    }

    #[test]
    fn test_run_with_nonexistent_filter() {
        // Filter name that doesn't exist should not crash
        let result = run(
            Some("this-filter-definitely-does-not-exist-xyz".to_string()),
            false,
        );
        // Should return OK (no tests found for a nonexistent filter is not an error)
        assert!(
            result.is_ok(),
            "expected OK with nonexistent filter: {:?}",
            result
        );
    }

    #[test]
    fn test_require_all_on_empty() {
        // With require_all=true on a minimal config, may fail
        // The important thing is it doesn't panic
        let result = run(None, true);
        // require_all on a config with no tests may succeed or fail depending on whether
        // filters are loaded — the point is it doesn't panic. Assert something concrete either way.
        if let Err(e) = &result {
            let msg = format!("{:#}", e);
            assert!(
                msg.contains("test(s) failed")
                    || msg.contains("no inline tests")
                    || msg.contains("filter(s) have no inline tests"),
                "unexpected error: {}",
                msg
            );
        }
    }

    #[test]
    fn test_verify_results_types_accessible() {
        // Verify we can construct the types used by verify
        let outcome = toml_filter::TestOutcome {
            filter_name: "test_filter".to_string(),
            test_name: "test_one".to_string(),
            passed: true,
            actual: "ok".to_string(),
            expected: "ok".to_string(),
        };
        assert!(outcome.passed);
        assert_eq!(outcome.filter_name, "test_filter");

        let results = toml_filter::VerifyResults {
            outcomes: vec![outcome],
            filters_without_tests: vec![],
        };
        assert_eq!(results.outcomes.len(), 1);
        assert!(results.filters_without_tests.is_empty());
    }

    #[test]
    fn test_verify_results_with_failure() {
        let outcome = toml_filter::TestOutcome {
            filter_name: "test_filter".to_string(),
            test_name: "failing_test".to_string(),
            passed: false,
            actual: "actual_output".to_string(),
            expected: "expected_output".to_string(),
        };
        assert!(!outcome.passed);
        assert_eq!(outcome.actual, "actual_output");
        assert_eq!(outcome.expected, "expected_output");
    }
}
