---
title: Quick Start
description: Get RTCO running in 5 minutes and see your first token savings
sidebar:
  order: 2
---

# Quick Start

This guide walks you through your first RTCO commands after installation.

## Prerequisites

RTCO is installed and verified:

```bash
rtco --version   # rtco x.y.z
rtco gain        # shows token savings dashboard
```

If not, see [Installation](./installation.md).

## Step 1: Initialize for your AI assistant

```bash
# For Claude Code (global — applies to all projects)
rtco init --global

# For a single project only
cd /your/project && rtco init
```

This installs the hook that automatically rewrites commands. Restart your AI assistant after this step.

### Preview without writing: `--dry-run`

To see exactly what `init` would change before it touches anything, add `--dry-run`:

```bash
rtco init --global --dry-run
```

Every would-be file create/update/patch is printed with a `[dry-run] would ...` prefix, then a `[dry-run] Nothing written.` footer. Nothing on disk is modified, no settings.json is patched, and the telemetry consent prompt is skipped. Combine with `-v` to also print the full content RTCO would write:

```bash
rtco init --global --dry-run -v
```

`--dry-run` works for every init flavour (`--agent cursor`, `--gemini`, `--codex`, `--copilot`, `--uninstall`, ...). It cannot be combined with `--show`.

## Step 2: Use your tools normally

Once the hook is installed, nothing changes in how you work. Your AI assistant runs commands as usual — the hook intercepts them transparently and rewrites them before execution.

For example, when Claude Code runs `cargo test`, the hook rewrites it to `rtco cargo test` before it executes. The LLM receives filtered output with only the failures — not 500 lines of passing tests. You never see or type `rtco`.

RTCO covers all major ecosystems — Git, Cargo/Rust, JavaScript, Python, Go, Ruby, .NET, Docker/Kubernetes, and more. See [What RTCO Optimizes](../resources/what-rtco-covers.md) for the full list.

## Step 3: Check your savings

After a few commands, see how much was saved:

```bash
rtco gain
```

```
Total commands : 12
Input tokens   : 45,230
Output tokens  : 4,890
Saved          : 40,340  (89.2%)
```

## Step 4: Unsupported commands

Commands RTCO doesn't recognize run through passthrough — output is unchanged, usage is tracked:

```bash
rtco proxy make install
```

## Next steps

- [What RTCO Optimizes](../resources/what-rtco-covers.md) — all supported commands and savings by ecosystem
- [Supported agents](./supported-agents.md) — Claude Code, Cursor, Copilot, and more
- [Configuration](./configuration.md) — customize RTCO behavior
