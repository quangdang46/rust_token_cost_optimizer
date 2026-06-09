//! Filters Next.js build output down to route metrics and bundle sizes.

use rtco_core::runner;
use rtco_core::truncate::CAP_WARNINGS;
use rtco_core::utils::{resolved_command, strip_ansi, tool_exists, truncate};
use anyhow::Result;
use regex::Regex;

pub fn run(args: &[String], verbose: u8) -> Result<i32> {
    // Try next directly first, fallback to npx if not found
    let next_exists = tool_exists("next");

    let mut cmd = if next_exists {
        resolved_command("next")
    } else {
        let mut c = resolved_command("npx");
        c.arg("next");
        c
    };

    cmd.arg("build");

    for arg in args {
        cmd.arg(arg);
    }

    if verbose > 0 {
        let tool = if next_exists { "next" } else { "npx next" };
        eprintln!("Running: {} build", tool);
    }

    runner::run_filtered(
        cmd,
        "next build",
        &args.join(" "),
        filter_next_build,
        runner::RunOptions::default(),
    )
}

/// Filter Next.js build output - extract routes, bundles, warnings
fn filter_next_build(output: &str) -> String {
    lazy_static::lazy_static! {
        // Route line pattern: ○ /dashboard    1.2 kB  132 kB
        static ref ROUTE_PATTERN: Regex = Regex::new(
            r"^[○●◐λ✓]\s+(/[^\s]*)\s+(\d+(?:\.\d+)?)\s*(kB|B)"
        ).unwrap();

        // Bundle size pattern
        static ref BUNDLE_PATTERN: Regex = Regex::new(
            r"^[○●◐λ✓]\s+([\w/\-\.]+)\s+(\d+(?:\.\d+)?)\s*(kB|B)\s+(\d+(?:\.\d+)?)\s*(kB|B)"
        ).unwrap();
    }

    let mut routes_static = 0;
    let mut routes_dynamic = 0;
    let mut routes_total = 0;
    let mut bundles: Vec<(String, f64, Option<f64>)> = Vec::new();
    let mut warnings = 0;
    let mut errors = 0;
    let mut build_time = String::new();

    // Strip ANSI codes
    let clean_output = strip_ansi(output);

    for line in clean_output.lines() {
        // Count route types by symbol
        if line.starts_with("○") {
            routes_static += 1;
            routes_total += 1;
        } else if line.starts_with("●") || line.starts_with("◐") {
            routes_dynamic += 1;
            routes_total += 1;
        } else if line.starts_with("λ") {
            routes_total += 1;
        }

        // Extract bundle information (route + size + total size)
        if let Some(caps) = BUNDLE_PATTERN.captures(line) {
            let route = caps[1].to_string();
            let size: f64 = caps[2].parse().unwrap_or(0.0);
            let total: f64 = caps[4].parse().unwrap_or(0.0);

            // Calculate percentage increase if both sizes present
            let pct_change = if total > 0.0 {
                Some(((total - size) / size) * 100.0)
            } else {
                None
            };

            bundles.push((route, total, pct_change));
        }

        // Count warnings and errors
        if line.to_lowercase().contains("warning") {
            warnings += 1;
        }
        if line.to_lowercase().contains("error") && !line.contains("0 error") {
            errors += 1;
        }

        // Extract build time
        if line.contains("Compiled") || line.contains("in") {
            if let Some(time_match) = extract_time(line) {
                build_time = time_match;
            }
        }
    }

    // Detect if build was skipped (already built)
    let already_built = clean_output.contains("already optimized")
        || clean_output.contains("Cache")
        || (routes_total == 0 && clean_output.contains("Ready"));

    // Build filtered output
    let mut result = String::new();
    result.push_str("Next.js Build\n");
    result.push_str("═══════════════════════════════════════\n");

    if already_built && routes_total == 0 {
        result.push_str("Already built (using cache)\n\n");
    } else if routes_total > 0 {
        result.push_str(&format!(
            "{} routes ({} static, {} dynamic)\n\n",
            routes_total, routes_static, routes_dynamic
        ));
    }

    if !bundles.is_empty() {
        result.push_str("Bundles:\n");

        // Sort by size (descending) and show top 10
        bundles.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        const MAX_BUNDLES: usize = CAP_WARNINGS;
        for (route, size, pct_change) in bundles.iter().take(MAX_BUNDLES) {
            let warning_marker = if let Some(pct) = pct_change {
                if *pct > 10.0 {
                    format!(" [warn] (+{:.0}%)", pct)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            result.push_str(&format!(
                "  {:<30} {:>6.0} kB{}\n",
                truncate(route, 30),
                size,
                warning_marker
            ));
        }

        if bundles.len() > MAX_BUNDLES {
            result.push_str(&format!(
                "\n  ... +{} more routes\n",
                bundles.len() - MAX_BUNDLES
            ));
        }

        result.push('\n');
    }

    // Show build time and status
    if !build_time.is_empty() {
        result.push_str(&format!("Time: {} | ", build_time));
    }

    result.push_str(&format!("Errors: {} | Warnings: {}\n", errors, warnings));

    result.trim().to_string()
}

/// Extract time from build output (e.g., "Compiled in 34.2s")
fn extract_time(line: &str) -> Option<String> {
    lazy_static::lazy_static! {
        static ref TIME_RE: Regex = Regex::new(r"(\d+(?:\.\d+)?)\s*(s|ms)").unwrap();
    }

    TIME_RE
        .captures(line)
        .map(|caps| format!("{}{}", &caps[1], &caps[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_next_build() {
        let output = r#"
   ▲ Next.js 15.2.0

   Creating an optimized production build ...
✓ Compiled successfully
✓ Linting and checking validity of types
✓ Collecting page data
○ /                            1.2 kB        132 kB
● /dashboard                   2.5 kB        156 kB
○ /api/auth                    0.5 kB         89 kB

Route (app)                    Size     First Load JS
┌ ○ /                          1.2 kB        132 kB
├ ● /dashboard                 2.5 kB        156 kB
└ ○ /api/auth                  0.5 kB         89 kB

○  (Static)  prerendered as static content
●  (SSG)     prerendered as static HTML
λ  (Server)  server-side renders at runtime

✓ Built in 34.2s
"#;
        let result = filter_next_build(output);
        assert!(result.contains("Next.js Build"));
        assert!(result.contains("routes"));
        assert!(!result.contains("Creating an optimized")); // Should filter verbose logs
    }

    #[test]
    fn test_extract_time() {
        assert_eq!(extract_time("Built in 34.2s"), Some("34.2s".to_string()));
        assert_eq!(
            extract_time("Compiled in 1250ms"),
            Some("1250ms".to_string())
        );
        assert_eq!(extract_time("No time here"), None);
    }

    fn count_tokens(s: &str) -> usize {
        s.split_whitespace().count()
    }

    #[test]
    fn test_filter_next_build_savings() {
        let input = r#"
   ▲ Next.js 15.2.0

   Creating an optimized production build ...
✓ Compiled successfully
✓ Linting and checking validity of types

✓ Collecting page data
   Generating static pages (6/6)
   Finalizing page generation

info  - Need to disable some ESLint rules? https://nextjs.org/docs/basic-features/eslint
info  - Creating an optimized production build...
warn  - You have enabled experimental feature (appDir) in next.config.js.
warn  - Experimental features are not covered by semver, and may cause unexpected or broken application behavior. Proceed with caution.

○ /                            1.2 kB        132 kB
● /dashboard                   2.5 kB        156 kB
○ /api/auth                    0.5 kB         89 kB
○ /users                       1.8 kB        201 kB
● /settings                    3.2 kB        245 kB
○ /api/health                  0.3 kB         45 kB

Route (app)                    Size     First Load JS
┌ ○ /                          1.2 kB        132 kB
├ ● /dashboard                 2.5 kB        156 kB
├ ○ /api/auth                  0.5 kB         89 kB
├ ○ /users                     1.8 kB        201 kB
├ ● /settings                  3.2 kB        245 kB
└ ○ /api/health                0.3 kB         45 kB

○  (Static)  prerendered as static content
●  (SSG)     prerendered as static HTML
λ  (Server)  server-side renders at runtime

✓ Built in 34.2s
"#;
        let output = filter_next_build(input);
        let raw_tokens = count_tokens(input);
        let filtered_tokens = count_tokens(&output);
        let raw_bytes = input.len();
        let filtered_bytes = output.len();
        let token_savings =
            100.0 - (filtered_tokens as f64 / raw_tokens as f64 * 100.0);
        let byte_savings =
            100.0 - (filtered_bytes as f64 / raw_bytes as f64 * 100.0);
        assert!(
            token_savings >= 60.0,
            "Expected ≥60% token savings, got {:.1}%",
            token_savings
        );
        assert!(
            byte_savings >= 60.0,
            "Expected ≥60% byte savings, got {:.1}%",
            byte_savings
        );
    }
}
