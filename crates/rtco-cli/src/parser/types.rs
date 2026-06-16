/// Canonical types for tool outputs
/// These provide a unified interface across different tool versions
use serde::{Deserialize, Serialize};

/// Test execution result (vitest, playwright, jest, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: Option<u64>,
    pub failures: Vec<TestFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    pub test_name: String,
    pub file_path: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
}

/// Dependency state (pnpm, npm, cargo, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyState {
    pub total_packages: usize,
    pub outdated_count: usize,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub wanted_version: Option<String>,
    pub dev_dependency: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_result_serialization() {
        let result = TestResult {
            total: 10,
            passed: 8,
            failed: 1,
            skipped: 1,
            duration_ms: Some(5000),
            failures: vec![TestFailure {
                test_name: "test_foo".to_string(),
                file_path: "src/lib.rs".to_string(),
                error_message: "assertion failed".to_string(),
                stack_trace: Some("at src/lib.rs:42".to_string()),
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"total\":10"));
        assert!(json.contains("\"passed\":8"));
        assert!(json.contains("\"test_foo\""));
    }

    #[test]
    fn test_test_result_no_stack_trace() {
        let result = TestResult {
            total: 5,
            passed: 5,
            failed: 0,
            skipped: 0,
            duration_ms: None,
            failures: vec![TestFailure {
                test_name: "test_bar".to_string(),
                file_path: "tests/integration.rs".to_string(),
                error_message: "timeout".to_string(),
                stack_trace: None,
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"test_bar\""));
        // stack_trace is Option<String> which serializes as null when None
        // (no skip_serializing_if annotation)
    }

    #[test]
    fn test_dependency_state_serialization() {
        let state = DependencyState {
            total_packages: 42,
            outdated_count: 3,
            dependencies: vec![Dependency {
                name: "serde".to_string(),
                current_version: "1.0.0".to_string(),
                latest_version: Some("1.0.2".to_string()),
                wanted_version: Some("1.0.1".to_string()),
                dev_dependency: false,
            }],
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"serde\""));
        assert!(json.contains("\"total_packages\":42"));
    }

    #[test]
    fn test_dependency_no_latest_version() {
        let dep = Dependency {
            name: "stdlib".to_string(),
            current_version: "1.0.0".to_string(),
            latest_version: None,
            wanted_version: None,
            dev_dependency: true,
        };
        let json = serde_json::to_string(&dep).unwrap();
        assert!(json.contains("\"stdlib\""));
        assert!(json.contains("\"dev_dependency\":true"));
        // latest_version None serializes as null (no skip_serializing_if annotation)
        assert!(json.contains("latest_version"));
    }
}
