# Ecosystem Filters — Design Document

## Overview

RTCO provides ecosystem-specific filter modules that understand the output
format of each tool and apply targeted compression strategies.  This document
describes the architecture, how to add new ecosystems, and the current
coverage.

## Architecture

Each ecosystem filter lives in `crates/rtco-cli/src/cmds/<ecosystem>/` and
typically has:

- `<cmd>_cmd.rs` — the filter implementation (or a shared `mod.rs`).
- `snapshots/` — insta snapshot test data.
- `README.md` — module-specific documentation.

Routing is handled in `main.rs` via the `Commands` enum:

```rust
pub enum Commands {
    Git(GitArgs),
    Cargo(CargoArgs),
    Npm(NpmArgs),
    // ...
}
```

Which dispatches to the appropriate `<ecosystem>::run(args)` function.

### Base pattern for a filter

```rust
pub fn run(args: MyArgs) -> Result<()> {
    let output = execute_command("tool", &args.to_cmd_args())?;
    let filtered = filter_output(&output.stdout)
        .unwrap_or_else(|e| {
            eprintln!("rtco: {}: warning: {}", NAME, e);
            output.stdout.clone()
        });
    print!("{filtered}");
    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }
    Ok(())
}
```

## Current Ecosystem Coverage

| Ecosystem | Filters | Est. Savings |
|-----------|---------|--------------|
| git | log, diff, status, branch, gh pr | 80-90% |
| rust | cargo build, test, clippy, check | 85-95% |
| js | npm, pnpm, vitest, tsc, lint, next, prettier, playwright, prisma | 70-90% |
| python | ruff, pytest, mypy, pip | 75-90% |
| go | go, golangci-lint | 70-85% |
| dotnet | dotnet build, test, binlog | 75-90% |
| cloud | aws, docker, kubectl, curl, psql | 60-80% |
| system | ls, tree, grep, find, read, wc, env, json, log, deps, summary | 70-85% |
| ruby | rake, rspec, rubocop | 70-85% |
| jvm | maven, gradle | 70-85% |

## Adding a New Ecosystem Filter

1. **Create directory**: `src/cmds/<ecosystem>/`
2. **Create mod.rs**: If using `automod`, add a `mod.rs` with `automod::dir!(pub "src/cmds/<ecosystem>")`. Otherwise, declare each module explicitly.
3. **Implement filter**: Follow the base pattern above.  Use `lazy_static!`
   regexes and the fallback pattern.
4. **Register in `main.rs`**: Add variant to `Commands` enum and match arm.
5. **Add tests**: Snapshot test + token savings test with real fixtures.
6. **Update this document**: Add the new ecosystem to the coverage table.

## Design Principles

1. **Real fixtures only** — no synthetic test data.  Capture real command output.
2. **Token savings assertions** — every filter must have a test verifying >= 60%
   savings (preferably >= 80%).
3. **Fallback always** — if compression fails, pass output through unchanged.
4. **Exit code propagation** — command failure exit codes must be preserved.
5. **ANSI stripping** — strip colors before applying content filters.
6. **Lazy regexes** — all `Regex::new()` in `lazy_static!` blocks.
