//! Shows users how many tokens RTCO has saved them over time.

use crate::hooks::hook_check;
use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;
use rtco_core::display_helpers::{format_duration, print_period_table};
use rtco_core::tracking::{DayStats, MonthStats, Tracker, WeekStats};
use rtco_core::utils::format_tokens;
use serde::Serialize;
use std::io::IsTerminal;
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub fn run(
    project: bool, // added: per-project scope flag
    graph: bool,
    history: bool,
    quota: bool,
    tier: &str,
    daily: bool,
    weekly: bool,
    monthly: bool,
    all: bool,
    format: &str,
    failures: bool,
    reset: bool,
    yes: bool,
    _verbose: u8,
) -> Result<()> {
    let tracker = Tracker::new().context("Failed to initialize tracking database")?;
    let project_scope = resolve_project_scope(project)?; // resolve project path

    if reset {
        if !yes && !confirm_reset()? {
            println!("Aborted.");
            return Ok(());
        }
        tracker
            .reset_all()
            .context("Failed to reset token savings")?;
        println!("{}", styled("Token savings stats reset to zero.", true));
        return Ok(());
    }

    if failures {
        return show_failures(&tracker);
    }

    // Handle export formats
    match format {
        "json" => {
            return export_json(
                &tracker,
                daily,
                weekly,
                monthly,
                all,
                project_scope.as_deref(), // pass project scope
            );
        }
        "csv" => {
            return export_csv(
                &tracker,
                daily,
                weekly,
                monthly,
                all,
                project_scope.as_deref(), // pass project scope
            );
        }
        _ => {} // Continue with text format
    }

    let summary = tracker
        .get_summary_filtered(project_scope.as_deref()) // use filtered variant
        .context("Failed to load token savings summary from database")?;

    if summary.total_commands == 0 {
        println!("No tracking data yet.");
        println!("Run some rtco commands to start tracking savings.");
        return Ok(());
    }

    // Default view (summary)
    if !daily && !weekly && !monthly && !all {
        // scope-aware styled header // merged upstream styled + project scope
        let title = if project_scope.is_some() {
            "rtco Token Savings (Project Scope)"
        } else {
            "rtco Token Savings (Global Scope)"
        };
        println!("{}", styled(title, true));
        println!("{}", "═".repeat(60));
        // added: show project path when scoped
        if let Some(ref scope) = project_scope {
            println!("Scope: {}", shorten_path(scope));
        }
        println!();

        // KPI-style aligned output
        print_kpi("Total commands", summary.total_commands.to_string());
        print_kpi("Input tokens", format_tokens(summary.total_input));
        print_kpi("Output tokens", format_tokens(summary.total_output));
        print_kpi(
            "Tokens saved",
            format!(
                "{} ({:.1}%)",
                format_tokens(summary.total_saved),
                summary.avg_savings_pct
            ),
        );
        print_kpi(
            "Total exec time",
            format!(
                "{} (avg {})",
                format_duration(summary.total_time_ms),
                format_duration(summary.avg_time_ms)
            ),
        );
        print_efficiency_meter(summary.avg_savings_pct);
        println!();

        // Warn about hook issues that silently kill savings (stderr, not stdout)
        match hook_check::status() {
            hook_check::HookStatus::Missing => {
                eprintln!(
                    "{}",
                    "[warn] No hook installed — run `rtco init -g` for automatic token savings"
                        .yellow()
                );
                eprintln!();
            }
            hook_check::HookStatus::Outdated => {
                eprintln!(
                    "{}",
                    "[warn] Hook outdated — run `rtco init -g` to update".yellow()
                );
                eprintln!();
            }
            hook_check::HookStatus::Ok => {}
        }

        // Lightweight RTCO_DISABLED bypass check (best-effort, silent on failure)
        if let Some(warning) = check_rtco_disabled_bypass() {
            eprintln!("{}", warning.yellow());
            eprintln!();
        }

        if !summary.by_command.is_empty() {
            // added: styled section header
            println!("{}", styled("By Command", true));

            // added: dynamic column widths for clean alignment
            let cmd_width = 24usize;
            let impact_width = 10usize;
            let count_width = summary
                .by_command
                .iter()
                .map(|(_, count, _, _, _)| count.to_string().len())
                .max()
                .unwrap_or(5)
                .max(5);
            let saved_width = summary
                .by_command
                .iter()
                .map(|(_, _, saved, _, _)| format_tokens(*saved).len())
                .max()
                .unwrap_or(5)
                .max(5);
            let time_width = summary
                .by_command
                .iter()
                .map(|(_, _, _, _, avg_time)| format_duration(*avg_time).len())
                .max()
                .unwrap_or(6)
                .max(6);

            let table_width = 3
                + 2
                + cmd_width
                + 2
                + count_width
                + 2
                + saved_width
                + 2
                + 6
                + 2
                + time_width
                + 2
                + impact_width;
            println!("{}", "─".repeat(table_width));
            println!(
                "{:>3}  {:<cmd_width$}  {:>count_width$}  {:>saved_width$}  {:>6}  {:>time_width$}  {:<impact_width$}",
                "#", "Command", "Count", "Saved", "Avg%", "Time", "Impact",
                cmd_width = cmd_width, count_width = count_width,
                saved_width = saved_width, time_width = time_width,
                impact_width = impact_width
            );
            println!("{}", "─".repeat(table_width));

            let max_saved = summary
                .by_command
                .iter()
                .map(|(_, _, saved, _, _)| *saved)
                .max()
                .unwrap_or(1);

            for (idx, (cmd, count, saved, pct, avg_time)) in summary.by_command.iter().enumerate() {
                let row_idx = format!("{:>2}.", idx + 1);
                let cmd_cell = style_command_cell(&truncate_for_column(cmd, cmd_width));
                let count_cell = format!("{:>count_width$}", count, count_width = count_width);
                let saved_cell = format!(
                    "{:>saved_width$}",
                    format_tokens(*saved),
                    saved_width = saved_width
                );
                let pct_plain = format!("{:>6}", format!("{pct:.1}%"));
                let pct_cell = colorize_pct_cell(*pct, &pct_plain); // added: color-coded percentage
                let time_cell = format!(
                    "{:>time_width$}",
                    format_duration(*avg_time),
                    time_width = time_width
                );
                let impact = mini_bar(*saved, max_saved, impact_width); // added: impact bar
                println!(
                    "{}  {}  {}  {}  {}  {}  {}",
                    row_idx, cmd_cell, count_cell, saved_cell, pct_cell, time_cell, impact
                );
            }
            println!("{}", "─".repeat(table_width));
            println!();
        }

        if graph && !summary.by_day.is_empty() {
            println!("{}", styled("Daily Savings (last 30 days)", true)); // styled header
            println!("──────────────────────────────────────────────────────────");
            print_ascii_graph(&summary.by_day);
            println!();
        }

        if history {
            let recent = tracker.get_recent_filtered(10, project_scope.as_deref())?; // changed: filtered
            if !recent.is_empty() {
                println!("{}", styled("Recent Commands", true)); // styled header
                println!("──────────────────────────────────────────────────────────");
                for rec in recent {
                    let time = rec.timestamp.with_timezone(&Local).format("%m-%d %H:%M");
                    // #2318: Use char-safe truncation instead of byte slicing
                    // which panics on CJK characters (byte index mid-multi-byte char)
                    let cmd_short = if rec.rtco_cmd.chars().count() > 25 {
                        let truncated: String = rec.rtco_cmd.chars().take(22).collect();
                        format!("{}...", truncated)
                    } else {
                        rec.rtco_cmd.clone()
                    };

                    let sign = if rec.savings_pct >= 70.0 {
                        "▲"
                    } else if rec.savings_pct >= 30.0 {
                        "■"
                    } else {
                        "•"
                    };
                    println!(
                        "{} {} {:<25} -{:.0}% ({})",
                        time,
                        sign,
                        cmd_short,
                        rec.savings_pct,
                        format_tokens(rec.saved_tokens)
                    );
                }
                println!();
            }
        }

        if quota {
            const ESTIMATED_PRO_MONTHLY: usize = 6_000_000;

            let (quota_tokens, tier_name) = match tier {
                "pro" => (ESTIMATED_PRO_MONTHLY, "Pro ($20/mo)"),
                "5x" => (ESTIMATED_PRO_MONTHLY * 5, "Max 5x ($100/mo)"),
                "20x" => (ESTIMATED_PRO_MONTHLY * 20, "Max 20x ($200/mo)"),
                _ => (ESTIMATED_PRO_MONTHLY, "Pro ($20/mo)"),
            };

            let quota_pct = (summary.total_saved as f64 / quota_tokens as f64) * 100.0;

            println!("{}", styled("Monthly Quota Analysis", true)); // styled header
            println!("──────────────────────────────────────────────────────────");
            print_kpi("Subscription tier", tier_name.to_string()); // added: KPI style
            print_kpi("Estimated monthly quota", format_tokens(quota_tokens));
            print_kpi(
                "Tokens saved (lifetime)",
                format_tokens(summary.total_saved),
            );
            print_kpi("Quota preserved", format!("{:.1}%", quota_pct));
            println!();
            println!("Note: Heuristic estimate based on ~44K tokens/5h (Pro baseline)");
            println!("      Actual limits use rolling 5-hour windows, not monthly caps.");
        }

        return Ok(());
    }

    // Time breakdown views
    if all || daily {
        print_daily_full(&tracker, project_scope.as_deref())?; // changed: pass project scope
    }

    if all || weekly {
        print_weekly(&tracker, project_scope.as_deref())?; // changed: pass project scope
    }

    if all || monthly {
        print_monthly(&tracker, project_scope.as_deref())?; // changed: pass project scope
    }

    Ok(())
}

/// Compute cost-savings estimates from token counts.
/// Uses a simple input/output cost model: $3/M input, $15/M output.
#[allow(dead_code)]
fn calculate_economics(input_tokens: usize, output_tokens: usize) -> (f64, f64) {
    let assumed_cost_per_input_token = 3.0 / 1_000_000.0;
    let assumed_cost_per_output_token = 15.0 / 1_000_000.0;
    let input_cost = input_tokens as f64 * assumed_cost_per_input_token;
    let output_cost = output_tokens as f64 * assumed_cost_per_output_token;
    let total_savings = input_cost + output_cost;
    let denominator = input_cost + output_cost + 1.0; // avoid div by zero
    (total_savings, 100.0 * total_savings / denominator)
}

// ── Display helpers (TTY-aware) ── // added: entire section

/// Format text with bold styling (TTY-aware). // added
fn styled(text: &str, strong: bool) -> String {
    if !std::io::stdout().is_terminal() {
        return text.to_string();
    }
    if strong {
        text.bold().green().to_string()
    } else {
        text.to_string()
    }
}

/// Print a key-value pair in KPI layout. // added
fn print_kpi(label: &str, value: String) {
    println!("{:<18} {}", format!("{label}:"), value);
}

/// Colorize percentage based on savings tier (TTY-aware). // added
fn colorize_pct_cell(pct: f64, padded: &str) -> String {
    if !std::io::stdout().is_terminal() {
        return padded.to_string();
    }
    if pct >= 70.0 {
        padded.green().bold().to_string()
    } else if pct >= 40.0 {
        padded.yellow().bold().to_string()
    } else {
        padded.red().bold().to_string()
    }
}

/// Truncate text to fit column width with ellipsis. // added
fn truncate_for_column(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let char_count = text.chars().count();
    if char_count <= width {
        return format!("{:<width$}", text, width = width);
    }
    if width <= 3 {
        return text.chars().take(width).collect();
    }
    let mut out: String = text.chars().take(width - 3).collect();
    out.push_str("...");
    out
}

/// Style command names with cyan+bold (TTY-aware). // added
fn style_command_cell(cmd: &str) -> String {
    if !std::io::stdout().is_terminal() {
        return cmd.to_string();
    }
    cmd.bright_cyan().bold().to_string()
}

/// Render a proportional bar chart segment (TTY-aware). // added
fn mini_bar(value: usize, max: usize, width: usize) -> String {
    if max == 0 || width == 0 {
        return String::new();
    }
    let filled = ((value as f64 / max as f64) * width as f64).round() as usize;
    let filled = filled.min(width);
    let mut bar = "█".repeat(filled);
    bar.push_str(&"░".repeat(width - filled));
    if std::io::stdout().is_terminal() {
        bar.cyan().to_string()
    } else {
        bar
    }
}

/// Print an efficiency meter with colored progress bar (TTY-aware). // added
fn print_efficiency_meter(pct: f64) {
    let width = 24usize;
    let filled = (((pct / 100.0) * width as f64).round() as usize).min(width);
    let meter = format!("{}{}", "█".repeat(filled), "░".repeat(width - filled));
    if std::io::stdout().is_terminal() {
        let pct_str = format!("{pct:.1}%");
        let colored_pct = if pct >= 70.0 {
            pct_str.green().bold().to_string()
        } else if pct >= 40.0 {
            pct_str.yellow().bold().to_string()
        } else {
            pct_str.red().bold().to_string()
        };
        println!("Efficiency meter: {} {}", meter.green(), colored_pct);
    } else {
        println!("Efficiency meter: {} {:.1}%", meter, pct);
    }
}

/// Resolve project scope from --project flag. // added
fn resolve_project_scope(project: bool) -> Result<Option<String>> {
    if !project {
        return Ok(None);
    }
    let cwd = std::env::current_dir().context("Failed to resolve current working directory")?;
    let canonical = cwd.canonicalize().unwrap_or(cwd);
    Ok(Some(canonical.to_string_lossy().to_string()))
}

/// Shorten long absolute paths for display. // added
fn shorten_path(path: &str) -> String {
    let path_buf = PathBuf::from(path);
    let comps: Vec<String> = path_buf
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    if comps.len() <= 4 {
        return path.to_string();
    }
    let root = comps[0].as_str();
    if root == "/" || root.is_empty() {
        format!("/.../{}/{}", comps[comps.len() - 2], comps[comps.len() - 1])
    } else {
        format!(
            "{}/.../{}/{}",
            root,
            comps[comps.len() - 2],
            comps[comps.len() - 1]
        )
    }
}

fn print_ascii_graph(data: &[(String, usize)]) {
    if data.is_empty() {
        return;
    }

    let max_val = data.iter().map(|(_, v)| *v).max().unwrap_or(1);
    let width = 40;

    for (date, value) in data {
        let date_short = if date.len() >= 10 { &date[5..10] } else { date };

        let bar_len = if max_val > 0 {
            ((*value as f64 / max_val as f64) * width as f64) as usize
        } else {
            0
        };

        let bar: String = "█".repeat(bar_len);
        let spaces: String = " ".repeat(width - bar_len);

        println!(
            "{} │{}{} {}",
            date_short,
            bar,
            spaces,
            format_tokens(*value)
        );
    }
}

fn print_daily_full(tracker: &Tracker, project_scope: Option<&str>) -> Result<()> {
    // changed: add project scope
    let days = tracker.get_all_days_filtered(project_scope)?; // use filtered variant
    print_period_table(&days);
    Ok(())
}

fn print_weekly(tracker: &Tracker, project_scope: Option<&str>) -> Result<()> {
    // changed: add project scope
    let weeks = tracker.get_by_week_filtered(project_scope)?; // use filtered variant
    print_period_table(&weeks);
    Ok(())
}

fn print_monthly(tracker: &Tracker, project_scope: Option<&str>) -> Result<()> {
    // changed: add project scope
    let months = tracker.get_by_month_filtered(project_scope)?; // use filtered variant
    print_period_table(&months);
    Ok(())
}

#[derive(Serialize)]
struct ExportData {
    summary: ExportSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    daily: Option<Vec<DayStats>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weekly: Option<Vec<WeekStats>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    monthly: Option<Vec<MonthStats>>,
}

#[derive(Serialize)]
struct ExportSummary {
    total_commands: usize,
    total_input: usize,
    total_output: usize,
    total_saved: usize,
    avg_savings_pct: f64,
    total_time_ms: u64,
    avg_time_ms: u64,
}

fn export_json(
    tracker: &Tracker,
    daily: bool,
    weekly: bool,
    monthly: bool,
    all: bool,
    project_scope: Option<&str>, // project scope
) -> Result<()> {
    let summary = tracker
        .get_summary_filtered(project_scope) // use filtered variant
        .context("Failed to load token savings summary from database")?;

    let export = ExportData {
        summary: ExportSummary {
            total_commands: summary.total_commands,
            total_input: summary.total_input,
            total_output: summary.total_output,
            total_saved: summary.total_saved,
            avg_savings_pct: summary.avg_savings_pct,
            total_time_ms: summary.total_time_ms,
            avg_time_ms: summary.avg_time_ms,
        },
        daily: if all || daily {
            Some(tracker.get_all_days_filtered(project_scope)?) // changed: use filtered
        } else {
            None
        },
        weekly: if all || weekly {
            Some(tracker.get_by_week_filtered(project_scope)?) // changed: use filtered
        } else {
            None
        },
        monthly: if all || monthly {
            Some(tracker.get_by_month_filtered(project_scope)?) // changed: use filtered
        } else {
            None
        },
    };

    let json = serde_json::to_string_pretty(&export)?;
    println!("{}", json);

    Ok(())
}

fn export_csv(
    tracker: &Tracker,
    daily: bool,
    weekly: bool,
    monthly: bool,
    all: bool,
    project_scope: Option<&str>, // project scope
) -> Result<()> {
    if all || daily {
        let days = tracker.get_all_days_filtered(project_scope)?; // changed: use filtered
        println!("# Daily Data");
        println!("date,commands,input_tokens,output_tokens,saved_tokens,savings_pct,total_time_ms,avg_time_ms");
        for day in days {
            println!(
                "{},{},{},{},{},{:.2},{},{}",
                day.date,
                day.commands,
                day.input_tokens,
                day.output_tokens,
                day.saved_tokens,
                day.savings_pct,
                day.total_time_ms,
                day.avg_time_ms
            );
        }
        println!();
    }

    if all || weekly {
        let weeks = tracker.get_by_week_filtered(project_scope)?; // changed: use filtered
        println!("# Weekly Data");
        println!(
            "week_start,week_end,commands,input_tokens,output_tokens,saved_tokens,savings_pct,total_time_ms,avg_time_ms"
        );
        for week in weeks {
            println!(
                "{},{},{},{},{},{},{:.2},{},{}",
                week.week_start,
                week.week_end,
                week.commands,
                week.input_tokens,
                week.output_tokens,
                week.saved_tokens,
                week.savings_pct,
                week.total_time_ms,
                week.avg_time_ms
            );
        }
        println!();
    }

    if all || monthly {
        let months = tracker.get_by_month_filtered(project_scope)?; // changed: use filtered
        println!("# Monthly Data");
        println!("month,commands,input_tokens,output_tokens,saved_tokens,savings_pct,total_time_ms,avg_time_ms");
        for month in months {
            println!(
                "{},{},{},{},{},{:.2},{},{}",
                month.month,
                month.commands,
                month.input_tokens,
                month.output_tokens,
                month.saved_tokens,
                month.savings_pct,
                month.total_time_ms,
                month.avg_time_ms
            );
        }
    }

    Ok(())
}

/// Lightweight scan of recent Claude Code sessions for RTCO_DISABLED= overuse.
/// Returns a warning string if bypass rate exceeds 10%, None otherwise.
/// Silently returns None on any error (missing dirs, permission issues, etc.).
fn check_rtco_disabled_bypass() -> Option<String> {
    use crate::discover::provider::{ClaudeProvider, SessionProvider};
    use crate::discover::registry::cmd_has_rtco_disabled_prefix;

    let provider = ClaudeProvider;

    // Quick scan: last 7 days only
    let sessions = provider.discover_sessions(None, Some(7)).ok()?;

    // Early bail if no sessions or too many (avoid slow scan)
    if sessions.is_empty() || sessions.len() > 200 {
        return None;
    }

    let mut total_bash: usize = 0;
    let mut bypassed: usize = 0;

    for session_path in &sessions {
        let extracted = match provider.extract_commands(session_path) {
            Ok(cmds) => cmds,
            Err(_) => continue,
        };

        for ext_cmd in &extracted {
            total_bash += 1;
            if cmd_has_rtco_disabled_prefix(&ext_cmd.command) {
                bypassed += 1;
            }
        }
    }

    if total_bash == 0 {
        return None;
    }

    let pct = (bypassed as f64 / total_bash as f64) * 100.0;
    if pct > 10.0 {
        Some(format!(
            "[warn] {} commands ({:.0}%) used RTCO_DISABLED=1 unnecessarily — run `rtco discover` for details",
            bypassed, pct
        ))
    } else {
        None
    }
}

fn show_failures(tracker: &Tracker) -> Result<()> {
    let summary = tracker
        .get_parse_failure_summary()
        .context("Failed to load parse failure data")?;

    if summary.total == 0 {
        println!("No parse failures recorded.");
        println!("This means all commands parsed successfully (or fallback hasn't triggered yet).");
        return Ok(());
    }

    println!("{}", styled("rtco Parse Failures", true));
    println!("{}", "═".repeat(60));
    println!();

    print_kpi("Total failures", summary.total.to_string());
    print_kpi("Recovery rate", format!("{:.1}%", summary.recovery_rate));
    println!();

    if !summary.top_commands.is_empty() {
        println!("{}", styled("Top Commands (by frequency)", true));
        println!("{}", "─".repeat(60));
        for (cmd, count) in &summary.top_commands {
            let cmd_display = if cmd.len() > 50 {
                format!("{}...", &cmd[..47])
            } else {
                cmd.clone()
            };
            println!("  {:>4}x  {}", count, cmd_display);
        }
        println!();
    }

    if !summary.recent.is_empty() {
        println!("{}", styled("Recent Failures (last 10)", true));
        println!("{}", "─".repeat(60));
        for rec in &summary.recent {
            let ts_short = if rec.timestamp.len() >= 16 {
                &rec.timestamp[..16]
            } else {
                &rec.timestamp
            };
            let status = if rec.fallback_succeeded { "ok" } else { "FAIL" };
            let cmd_display = if rec.raw_command.len() > 40 {
                format!("{}...", &rec.raw_command[..37])
            } else {
                rec.raw_command.clone()
            };
            println!("  {} [{}] {}", ts_short, status, cmd_display);
        }
        println!();
    }

    Ok(())
}

/// Prompt the user to confirm a destructive reset operation.
/// Defaults to No in non-interactive (piped) environments.
fn confirm_reset() -> Result<bool> {
    use std::io::{self, BufRead, IsTerminal, Write};

    eprint!("This will permanently delete all tracking data. Continue? [y/N] ");
    io::stderr().flush().ok();

    if !io::stdin().is_terminal() {
        eprintln!("(non-interactive mode, defaulting to N)");
        return Ok(false);
    }

    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .context("Failed to read confirmation")?;

    Ok(matches!(line.trim().to_lowercase().as_str(), "y" | "yes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── print_daily_stats equivalent: test print_ascii_graph ──

    #[test]
    fn test_print_ascii_graph_empty() {
        // Should not panic with empty data
        let data: Vec<(String, usize)> = vec![];
        print_ascii_graph(&data);
    }

    #[test]
    fn test_print_ascii_graph_single_entry() {
        let data = vec![("2026-01-01".to_string(), 100)];
        // Should not panic
        print_ascii_graph(&data);
    }

    #[test]
    fn test_print_ascii_graph_multiple_entries() {
        let data = vec![
            ("2026-01-01".to_string(), 50),
            ("2026-01-02".to_string(), 200),
            ("2026-01-03".to_string(), 0),
        ];
        // Should not panic with multiple entries including zero
        print_ascii_graph(&data);
    }

    // ── summary stats helpers ──

    #[test]
    fn test_truncate_for_column() {
        assert_eq!(truncate_for_column("hello", 10), "hello     ");
        assert_eq!(truncate_for_column("hello world", 5), "he...");
        assert_eq!(truncate_for_column("hello", 3), "hel");
        assert_eq!(truncate_for_column("hello", 0), "");
    }

    #[test]
    fn test_shorten_path() {
        let short = "/home/user/proj";
        assert_eq!(shorten_path(short), short);

        let long = "/home/user/projects/rust/myproject";
        let result = shorten_path(long);
        assert!(result.contains("/.../"));

        let root_short = "/a/b";
        assert_eq!(shorten_path(root_short), "/a/b");
    }

    #[test]
    fn test_calculate_economics() {
        use super::*;

        let (savings, pct) = calculate_economics(1_000_000, 100_000);
        assert!(savings > 0.0);
        assert!(pct > 0.0);

        // Zero tokens should not panic
        let (savings, _) = calculate_economics(0, 0);
        assert!(savings == 0.0);
    }

    // ── mini_bar ──

    #[test]
    fn test_mini_bar() {
        let bar = mini_bar(50, 100, 10);
        assert_eq!(bar.chars().filter(|&c| c == '█').count(), 5);
        assert_eq!(bar.chars().filter(|&c| c == '░').count(), 5);

        let zero_bar = mini_bar(0, 100, 10);
        assert_eq!(zero_bar.chars().filter(|&c| c == '░').count(), 10);

        let full_bar = mini_bar(100, 100, 10);
        assert_eq!(full_bar.chars().filter(|&c| c == '█').count(), 10);
    }

    #[test]
    fn test_mini_bar_zero_max() {
        let bar = mini_bar(0, 0, 10);
        assert!(bar.is_empty());
    }

    #[test]
    fn test_mini_bar_zero_width() {
        let bar = mini_bar(100, 100, 0);
        assert!(bar.is_empty());
    }

    // ── resolve_project_scope ──

    #[test]
    fn test_resolve_project_scope_disabled() {
        let result = resolve_project_scope(false).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_project_scope_enabled() {
        // When --project is passed, should return Some with current dir
        let result = resolve_project_scope(true).unwrap();
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(!path.is_empty());
    }

    // ── colorize_pct_cell ──

    #[test]
    fn test_colorize_pct_cell() {
        let padded = format!("{:>6.1}%", 75.0);
        let result = colorize_pct_cell(75.0, &padded);
        assert_eq!(result, padded); // Same content (no ANSI in non-TTY test)

        let result_low = colorize_pct_cell(20.0, &padded);
        assert_eq!(result_low, padded);
    }

    // ── ExportData types ──

    #[test]
    fn test_export_summary_serializable() {
        // Verify the struct is constructable and serializable
        let summary = ExportSummary {
            total_commands: 100,
            total_input: 5000,
            total_output: 3000,
            total_saved: 2000,
            avg_savings_pct: 50.0,
            total_time_ms: 10000,
            avg_time_ms: 100,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"total_commands\":100"));
        assert!(json.contains("\"avg_savings_pct\":50.0"));
    }

    #[test]
    fn test_serialize_empty_export_data() {
        let export = ExportData {
            summary: ExportSummary {
                total_commands: 0,
                total_input: 0,
                total_output: 0,
                total_saved: 0,
                avg_savings_pct: 0.0,
                total_time_ms: 0,
                avg_time_ms: 0,
            },
            daily: None,
            weekly: None,
            monthly: None,
        };
        let json = serde_json::to_string(&export).unwrap();
        assert!(json.contains("\"summary\""));
        // Optional fields should be skipped
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!v.as_object().unwrap().contains_key("daily"));
    }
}
