# RTCO - Rust Token Killer (Kilo Code)

**Usage**: Token-optimized CLI proxy for shell commands.

## Rule

Always prefix shell commands with `rtco` to minimize token consumption.

Examples:

```bash
rtco git status
rtco cargo test
rtco ls src/
rtco grep "pattern" src/
rtco find "*.rs" .
rtco docker ps
rtco gh pr list
```

## Meta Commands

```bash
rtco gain              # Show token savings
rtco gain --history    # Command history with savings
rtco discover          # Find missed RTCO opportunities
rtco proxy <cmd>       # Run raw (no filtering, for debugging)
```

## Why

RTCO filters and compresses command output before it reaches the LLM context, saving 60-90% tokens on common operations. Always use `rtco <cmd>` instead of raw commands.
