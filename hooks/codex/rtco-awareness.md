# RTCO - Rust Token Killer (Codex CLI)

**Usage**: Token-optimized CLI proxy for shell commands.

## Rule

Always prefix shell commands with `rtco`.

Examples:

```bash
rtco git status
rtco cargo test
rtco npm run build
rtco pytest -q
```

## Meta Commands

```bash
rtco gain            # Token savings analytics
rtco gain --history  # Recent command savings history
rtco proxy <cmd>     # Run raw command without filtering
```

## Verification

```bash
rtco --version
rtco gain
which rtco
```
