# RTCO — Rust Token Killer (Agent Instructions)

This repository contains **rtco**, a high-performance CLI proxy that minimizes LLM token consumption by filtering and compressing command outputs (60-90% savings).

## Usage Rule

Always prefix shell commands with `rtco` to save tokens.

```bash
rtco git status
rtco cargo test
rtco cargo build
rtco npm run build
rtco pnpm install
rtco docker ps
rtco gh pr list
rtco curl https://api.example.com
```

## Meta Commands (use rtco directly)

```bash
rtco gain              # Token savings analytics
rtco gain --history    # Recent command savings history
rtco proxy <cmd>       # Execute raw command without filtering (bypass mode)
rtco discover          # Analyze history for missed savings opportunities
```

## Rewrite System

rtco includes a `rtco rewrite` command that translates raw commands to optimized equivalents. This is used by the Claude Code and OpenCode hooks.

```bash
rtco rewrite "git status"   # → "rtco git status"
rtco rewrite "cargo test"   # → "rtco cargo test"
```

## Verification

```bash
rtco --version         # Should show rtco X.Y.Z
rtco gain              # Should work (not "command not found")
which rtco             # Verify correct binary
```

> ⚠️ **Name collision**: If `rtco gain` fails, you may have reachingforthejack/rtco installed instead.
