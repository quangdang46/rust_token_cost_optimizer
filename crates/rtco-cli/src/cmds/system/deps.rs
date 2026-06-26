//! Summarizes project dependencies from lock files and manifests.

use rtco_core::tracking;
use rtco_core::truncate::{reduced, CAP_WARNINGS};
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

const MAX_DEPS: usize = CAP_WARNINGS;
// dev deps are secondary to prod — show fewer.
const MAX_DEV_DEPS: usize = reduced(CAP_WARNINGS, 5);

/// Summarize project dependencies
pub fn run(path: &Path, verbose: u8) -> Result<()> {
    let timer = tracking::TimedExecution::start();

    let dir = if path.is_file() {
        path.parent().unwrap_or(Path::new("."))
    } else {
        path
    };

    if verbose > 0 {
        eprintln!("Scanning dependencies in: {}", dir.display());
    }

    let mut found = false;
    let mut deps_output = String::new();
    let mut raw = String::new();

    let cargo_path = dir.join("Cargo.toml");
    if cargo_path.exists() {
        found = true;
        raw.push_str(&fs::read_to_string(&cargo_path).unwrap_or_default());
        deps_output.push_str("Rust (Cargo.toml):\n");
        deps_output.push_str(&summarize_cargo_str(&cargo_path)?);
    }

    let package_path = dir.join("package.json");
    if package_path.exists() {
        found = true;
        raw.push_str(&fs::read_to_string(&package_path).unwrap_or_default());
        deps_output.push_str("Node.js (package.json):\n");
        deps_output.push_str(&summarize_package_json_str(&package_path)?);
    }

    let requirements_path = dir.join("requirements.txt");
    if requirements_path.exists() {
        found = true;
        raw.push_str(&fs::read_to_string(&requirements_path).unwrap_or_default());
        deps_output.push_str("Python (requirements.txt):\n");
        deps_output.push_str(&summarize_requirements_str(&requirements_path)?);
    }

    let pyproject_path = dir.join("pyproject.toml");
    if pyproject_path.exists() {
        found = true;
        raw.push_str(&fs::read_to_string(&pyproject_path).unwrap_or_default());
        deps_output.push_str("Python (pyproject.toml):\n");
        deps_output.push_str(&summarize_pyproject_str(&pyproject_path)?);
    }

    let gomod_path = dir.join("go.mod");
    if gomod_path.exists() {
        found = true;
        raw.push_str(&fs::read_to_string(&gomod_path).unwrap_or_default());
        deps_output.push_str("Go (go.mod):\n");
        deps_output.push_str(&summarize_gomod_str(&gomod_path)?);
    }

    if !found {
        deps_output.push_str(&format!("No dependency files found in {}", dir.display()));
    }

    print!("{}", deps_output);
    timer.track("cat */deps", "rtco deps", &raw, &deps_output);
    Ok(())
}

/// Extract workspace member paths from Cargo.toml (#46)
#[allow(dead_code)]
fn extract_workspace_members(content: &str) -> Option<Vec<String>> {
    let member_re = Regex::new(r#"^\s*"([^"]+)"\s*$"#).unwrap();
    let mut in_workspace = false;
    let mut members = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace]" {
            in_workspace = true;
            continue;
        }
        if in_workspace {
            if trimmed.starts_with('[') {
                break;
            }
            if trimmed == "members = [" || trimmed.ends_with('[') {
                continue;
            }
            if trimmed == "]" || trimmed.starts_with("]") {
                continue;
            }
            if let Some(caps) = member_re.captures(trimmed) {
                if let Some(m) = caps.get(1) {
                    members.push(m.as_str().to_string());
                }
            }
        }
    }

    if members.is_empty() { None } else { Some(members) }
}

fn summarize_cargo_str(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let dep_re =
        Regex::new(r#"^([a-zA-Z0-9_-]+)\s*=\s*(?:"([^"]+)"|.*version\s*=\s*"([^"]+)")"#).unwrap();
    let section_re = Regex::new(r"^\[([^\]]+)\]").unwrap();
    let mut current_section = String::new();
    let mut deps = Vec::new();
    let mut dev_deps = Vec::new();
    let mut out = String::new();

    for line in content.lines() {
        if let Some(caps) = section_re.captures(line) {
            current_section = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
        } else if let Some(caps) = dep_re.captures(line) {
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let version = caps
                .get(2)
                .or(caps.get(3))
                .map(|m| m.as_str())
                .unwrap_or("*");
            let dep = format!("{} ({})", name, version);
            match current_section.as_str() {
                "dependencies" => deps.push(dep),
                "dev-dependencies" => dev_deps.push(dep),
                _ => {}
            }
        }
    }

    if !deps.is_empty() {
        out.push_str(&format!("  Dependencies ({}):\n", deps.len()));
        for d in deps.iter().take(MAX_DEPS) {
            out.push_str(&format!("    {}\n", d));
        }
        if deps.len() > MAX_DEPS {
            out.push_str(&format!("    ... +{} more\n", deps.len() - MAX_DEPS));
        }
    }
    if !dev_deps.is_empty() {
        out.push_str(&format!("  Dev ({}):\n", dev_deps.len()));
        for d in dev_deps.iter().take(MAX_DEV_DEPS) {
            out.push_str(&format!("    {}\n", d));
        }
        if dev_deps.len() > MAX_DEV_DEPS {
            out.push_str(&format!("    ... +{} more\n", dev_deps.len() - MAX_DEV_DEPS));
        }
    }
    Ok(out)
}

fn summarize_package_json_str(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    let mut out = String::new();

    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
        let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("?");
        out.push_str(&format!("  {} @ {}\n", name, version));
    }
    if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
        out.push_str(&format!("  Dependencies ({}):\n", deps.len()));
        for (i, (name, version)) in deps.iter().enumerate() {
            if i >= MAX_DEPS {
                out.push_str(&format!("    ... +{} more\n", deps.len() - MAX_DEPS));
                break;
            }
            out.push_str(&format!(
                "    {} ({})\n",
                name,
                version.as_str().unwrap_or("*")
            ));
        }
    }
    if let Some(dev_deps) = json.get("devDependencies").and_then(|v| v.as_object()) {
        out.push_str(&format!("  Dev Dependencies ({}):\n", dev_deps.len()));
        for (i, (name, _)) in dev_deps.iter().enumerate() {
            if i >= MAX_DEV_DEPS {
                out.push_str(&format!("    ... +{} more\n", dev_deps.len() - MAX_DEV_DEPS));
                break;
            }
            out.push_str(&format!("    {}\n", name));
        }
    }
    Ok(out)
}

fn summarize_requirements_str(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let dep_re = Regex::new(r"^([a-zA-Z0-9_-]+)([=<>!~]+.*)?$").unwrap();
    let mut deps = Vec::new();
    let mut out = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(caps) = dep_re.captures(line) {
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let version = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            deps.push(format!("{}{}", name, version));
        }
    }

    out.push_str(&format!("  Packages ({}):\n", deps.len()));
    for d in deps.iter().take(MAX_DEPS) {
        out.push_str(&format!("    {}\n", d));
    }
    if deps.len() > MAX_DEPS {
        out.push_str(&format!("    ... +{} more\n", deps.len() - MAX_DEPS));
    }
    Ok(out)
}

fn summarize_pyproject_str(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let mut in_deps = false;
    let mut deps = Vec::new();
    let mut out = String::new();

    for line in content.lines() {
        if line.contains("dependencies") && line.contains("[") {
            in_deps = true;
            continue;
        }
        if in_deps {
            if line.trim() == "]" {
                break;
            }
            let line = line
                .trim()
                .trim_matches(|c| c == '"' || c == '\'' || c == ',');
            if !line.is_empty() {
                deps.push(line.to_string());
            }
        }
    }

    if !deps.is_empty() {
        out.push_str(&format!("  Dependencies ({}):\n", deps.len()));
        for d in deps.iter().take(MAX_DEPS) {
            out.push_str(&format!("    {}\n", d));
        }
        if deps.len() > MAX_DEPS {
            out.push_str(&format!("    ... +{} more\n", deps.len() - MAX_DEPS));
        }
    }
    Ok(out)
}

fn summarize_gomod_str(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let mut module_name = String::new();
    let mut go_version = String::new();
    let mut deps = Vec::new();
    let mut in_require = false;
    let mut out = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("module ") {
            module_name = line.trim_start_matches("module ").to_string();
        } else if line.starts_with("go ") {
            go_version = line.trim_start_matches("go ").to_string();
        } else if line == "require (" {
            in_require = true;
        } else if line == ")" {
            in_require = false;
        } else if in_require && !line.starts_with("//") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                deps.push(format!("{} {}", parts[0], parts[1]));
            }
        } else if line.starts_with("require ") && !line.contains("(") {
            deps.push(line.trim_start_matches("require ").to_string());
        }
    }

    if !module_name.is_empty() {
        out.push_str(&format!("  {} (go {})\n", module_name, go_version));
    }
    if !deps.is_empty() {
        out.push_str(&format!("  Dependencies ({}):\n", deps.len()));
        for d in deps.iter().take(MAX_DEPS) {
            out.push_str(&format!("    {}\n", d));
        }
        if deps.len() > MAX_DEPS {
            out.push_str(&format!("    ... +{} more\n", deps.len() - MAX_DEPS));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_empty_dir() {
        // Creating a temp dir with no manifest files should handle gracefully
        let dir = std::env::temp_dir().join("rtco_deps_test_empty");
        std::fs::create_dir_all(&dir).unwrap_or_default();
        let result = run(&dir, 0);
        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_summarize_package_json_basic() {
        let dir = std::env::temp_dir().join("rtco_deps_test_pj");
        std::fs::create_dir_all(&dir).unwrap_or_default();
        let pj_path = dir.join("package.json");
        std::fs::write(&pj_path, r#"{
            "name": "test-pkg",
            "version": "1.0.0",
            "dependencies": {
                "react": "^18.0.0",
                "lodash": "^4.17.0"
            },
            "devDependencies": {
                "typescript": "^5.0.0"
            }
        }"#).unwrap();
        let result = run(&dir, 0);
        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cargo_toml_deps() {
        let dir = std::env::temp_dir().join("rtco_deps_test_cargo");
        std::fs::create_dir_all(&dir).unwrap_or_default();
        let cargo_path = dir.join("Cargo.toml");
        std::fs::write(&cargo_path, r#"[package]
name = "test-pkg"
version = "0.1.0"

[dependencies]
serde = "1.0"
regex = "1.5"

[dev-dependencies]
criterion = "0.5"
"#).unwrap();
        let result = run(&dir, 0);
        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_requirements_file() {
        let dir = std::env::temp_dir().join("rtco_deps_test_req");
        std::fs::create_dir_all(&dir).unwrap_or_default();
        let req_path = dir.join("requirements.txt");
        std::fs::write(&req_path, "requests==2.28.0\nflask==2.3.0\n").unwrap();
        let result = run(&dir, 0);
        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_gomod() {
        let dir = std::env::temp_dir().join("rtco_deps_test_go");
        std::fs::create_dir_all(&dir).unwrap_or_default();
        let gomod_path = dir.join("go.mod");
        std::fs::write(&gomod_path, r#"module github.com/test/pkg
go 1.21
require (
    github.com/pkg/errors v0.9.1
    golang.org/x/text v0.14.0
)
"#).unwrap();
        let result = run(&dir, 0);
        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
