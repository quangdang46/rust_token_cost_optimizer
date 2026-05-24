# Copilot Instructions for rtco

**rtco (Rust Token Killer)** is a CLI proxy that filters and compresses command outputs before they reach an LLM context, saving 60-90% of tokens. It wraps common tools (`git`, `cargo`, `grep`, `pnpm`, `go`, etc.) and outputs condensed summaries instead of raw output.

## Using rtco in this session

**Always prefix commands with `rtco` when running shell commands** — this reduces token consumption for every operation you perform.

```bash
# Instead of:              Use:
git status                 rtco git status
git log -10                rtco git log -10
cargo test                 rtco cargo test
cargo clippy --all-targets rtco cargo clippy --all-targets
grep -r "pattern" src/     rtco grep -r "pattern" src/
```

**rtco meta-commands** (always use these directly, no prefix needed):
```bash
rtco gain              # Show token savings analytics
rtco gain --history    # Full command history with per-command savings
rtco discover          # Scan session history for missed rtco opportunities
rtco proxy <cmd>       # Run a command raw (no filtering) but still track it
```

**Verify rtco is installed before starting:**
```bash
rtco --version   # Should print: rtco X.Y.Z
rtco gain        # Should show a dashboard (not "command not found")
```

> Name collision: `rtco gain` failing means you have `reachingforthejack/rtco` (Rust Type Kit) installed instead. Run `which rtco` to check.

## Build, Test & Lint

```bash
cargo build                    # Development build
cargo test                     # All tests
cargo test test_name           # Single test
cargo test module::tests::     # Module tests
cargo test -- --nocapture      # With stdout

# Pre-commit gate (must all pass before any PR)
cargo fmt --all --check && cargo clippy --all-targets && cargo test

bash scripts/test-all.sh       # Smoke tests (requires installed binary)
```

PRs target the **`develop`** branch, not `main`. All commits require a DCO sign-off (`git commit -s`).

## Architecture

rtco routes CLI commands via a Clap `Commands` enum in `main.rs` to specialized filter modules in `src/cmds/*/`, each executing the underlying command and compressing output. Token savings are tracked in SQLite via `src/core/tracking.rs`.

For full details see [ARCHITECTURE.md](../docs/contributing/ARCHITECTURE.md) and [docs/contributing/TECHNICAL.md](../docs/contributing/TECHNICAL.md). Module responsibilities are documented in each folder's `README.md` and each file's `//!` doc header.

## Key Conventions

- **Error handling**: `anyhow::Result` with `.context("description")?` — no bare `?`, no `unwrap()` in production. Filters must fall back to raw command on error.
- **Regex**: Always `lazy_static!`, never compile inside a function body.
- **Testing**: Unit tests inside modules (`#[cfg(test)] mod tests`). Fixtures in `tests/fixtures/`. Token savings assertions with `count_tokens()`.
- **Exit codes**: Preserve the underlying command's exit code via `std::process::exit(code)`.
- **Performance**: Startup <10ms (no async runtime), binary <5MB stripped.
- **Branch naming**: `fix(scope):`, `feat(scope):`, `chore(scope):` where scope is the affected component.

For the full contribution workflow, design philosophy, and new-filter checklist, see [CONTRIBUTING.md](../CONTRIBUTING.md).
