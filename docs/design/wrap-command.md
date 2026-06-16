# Wrap Command — Design Document

## Overview

The `rtco wrap <command>` subcommand wraps an existing CLI command so that its
output is automatically filtered by RTCO before it reaches the terminal (or any
consumer).  Unlike `rtco proxy` which passes output through unchanged, `rtco
wrap` applies the full content-detection and compression pipeline.

This is useful for:
- **Aliasing**: `alias ll='rtco wrap ls -la'` — every invocation is compressed.
- **CI pipelines**: wrapping verbose tool output to keep logs manageable.
- **Agentic workflows**: wrapping commands called by LLM agents so the context
  stays lean.

## Usage

```bash
# Basic usage — run the command through RTCO's compression pipeline
rtco wrap cargo test

# With original exit code propagation
rtco wrap --exit-code ./deploy.sh

# Strip ANSI before compressing
rtco wrap --strip-ansi npm run build

# Write wrapped output to a file instead of stdout
rtco wrap --output build.log make

# Wrapping a piped command
rtco wrap -- ls -la | grep target
```

## Architecture

```
User shell
    │
    ▼
rtco wrap <command> [args...]
    │
    ├── 1. Execute <command> via std::process::Command
    ├── 2. Capture stdout + stderr
    ├── 3. Detect content type (content_detector)
    ├── 4. Route to handler (content_router)
    ├── 5. Apply compression
    ├── 6. Print compressed output
    └── 7. Exit with command's exit code
```

### Integration with existing modules

| Component | Role |
|-----------|------|
| `content_detector` | Classify raw output |
| `content_router` | Dispatch to best handler |
| `core::utils::execute_command` | Run the wrapped command |
| `core::tracking` | Record token savings |

### Exit code handling

By default, the wrap command exits with the wrapped command's exit code,
matching the behavior of existing RTCO filter commands.

- `--exit-code` (default: `true`): propagate exit code.
- `--exit-code false`: always exit 0 (useful when wrapping non-critical tools).
- If the wrapped command cannot be started, exit 127 (command not found).

### Stderr handling

- Stderr from the wrapped command is passed through directly (not compressed)
  so that error messages remain timely and unaltered.
- `--capture-stderr` flag can merge stderr into the compression pipeline.

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--output` / `-o` | (stdout) | Write compressed output to a file |
| `--strip-ansi` | `true` | Strip ANSI escape codes before compression |
| `--exit-code` | `true` | Propagate wrapped command exit code |
| `--capture-stderr` | `false` | Merge stderr into compression pipeline |
| `--append` | `false` | Append to `--output` file instead of overwriting |
| `--quiet` / `-q` | `false` | Suppress RTCO diagnostics on stderr |

## Implementation

### Stub module

File: `crates/rtco-cli/src/cmds/system/wrap_cmd.rs`

```rust
use anyhow::{Context, Result};

pub struct WrapArgs {
    pub command: String,
    pub args: Vec<String>,
    pub output: Option<String>,
    pub strip_ansi: bool,
    pub exit_code: bool,
    pub capture_stderr: bool,
    pub append: bool,
    pub quiet: bool,
}

pub fn run(args: WrapArgs) -> Result<()> {
    // Placeholder — delegates to ContentRouter
    let router = rtco_core::content_router::ContentRouter::new();
    // ... execute command, capture output, route through router, print
    Ok(())
}
```

### Routing in `main.rs`

The `Commands` enum gains a `Wrap` variant:

```rust
#[derive(Debug, Clone)]
pub enum Subcommand {
    // ... existing variants ...
    Wrap {
        command: String,
        args: Vec<String>,
    },
}
```

When the `wrap` subcommand is matched, `cmds::system::wrap_cmd::run()` is
called with `WrapArgs` parsed from the remaining CLI arguments.

## Milestones

### M1 — Stub + integration
- [ ] Add `Wrap` variant to `Commands` enum
- [ ] Create `wrap_cmd.rs` with `run()` entry point
- [ ] Register in `main.rs` match dispatch
- [ ] Basic argument parsing (command + args from positional)

### M2 — Core logic
- [ ] Execute wrapped command, capture output
- [ ] Pipe through `ContentRouter`
- [ ] Print compressed output
- [ ] Propagate exit code

### M3 — Options
- [ ] `--output` / `--append` file support
- [ ] `--strip-ansi` flag
- [ ] `--capture-stderr` flag
- [ ] Stderr passthrough (default) vs capture

### M4 — Polish
- [ ] Token savings tracking via `core::tracking`
- [ ] Colored feedback with `--quiet` suppression
- [ ] Handle shell metacharacters in command name
- [ ] Integration tests

## Relationship to other commands

| Command | Description | Key difference |
|---------|-------------|----------------|
| `rtco <cmd>` | Direct filter (e.g. `rtco git log`) | Only works for known commands |
| `rtco proxy <cmd>` | Execute without filtering | No compression, tracking only |
| `rtco wrap <cmd>` | Execute with auto-detection + compression | Universal, content-aware |
