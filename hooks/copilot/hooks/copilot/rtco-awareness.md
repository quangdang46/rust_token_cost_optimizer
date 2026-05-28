# RTCO — Copilot Integration (VS Code Copilot Chat + Copilot CLI)

**Usage**: Token-optimized CLI proxy (60-90% savings on dev operations)

## What's automatic

The `.github/copilot-instructions.md` file is loaded at session start by both Copilot CLI and VS Code Copilot Chat.
It instructs Copilot to prefix commands with `rtco` automatically.

The `.github/hooks/rtco-rewrite.json` hook adds a `PreToolUse` safety net via `rtco hook` —
a cross-platform Rust binary that intercepts raw bash tool calls and rewrites them.
No shell scripts, no `jq` dependency, works on Windows natively.

## Meta commands (always use directly)

```bash
rtco gain              # Token savings dashboard for this session
rtco gain --history    # Per-command history with savings %
rtco discover          # Scan session history for missed rtco opportunities
rtco proxy <cmd>       # Run raw (no filtering) but still track it
```

## Installation verification

```bash
rtco --version   # Should print: rtco X.Y.Z
rtco gain        # Should show a dashboard (not "command not found")
which rtco       # Verify correct binary path
```

> ⚠️ **Name collision**: If `rtco gain` fails, you may have `reachingforthejack/rtk`
> (Rust Type Kit) installed instead. Check `which rtco` and reinstall from rtk-ai/rtk.

## How the hook works

`rtco hook` reads `PreToolUse` JSON from stdin, detects the agent format, and responds appropriately:

**VS Code Copilot Chat** (supports `updatedInput` — transparent rewrite, no denial):
1. Agent runs `git status` → `rtco hook` intercepts via `PreToolUse`
2. `rtco hook` detects VS Code format (`tool_name`/`tool_input` keys)
3. Returns `hookSpecificOutput.updatedInput.command = "rtco git status"`
4. Agent runs the rewritten command silently — no denial, no retry

**GitHub Copilot CLI** (deny-with-suggestion — CLI ignores `updatedInput` today, see [issue #2013](https://github.com/github/copilot-cli/issues/2013)):
1. Agent runs `git status` → `rtco hook` intercepts via `PreToolUse`
2. `rtco hook` detects Copilot CLI format (`toolName`/`toolArgs` keys)
3. Returns `permissionDecision: deny` with reason: `"Token savings: use 'rtco git status' instead"`
4. Copilot reads the reason and re-runs `rtco git status`

When Copilot CLI adds `updatedInput` support, only `rtco hook` needs updating — no config changes.

## Integration comparison

| Tool                  | Mechanism                               | Hook output              | File                               |
|-----------------------|-----------------------------------------|--------------------------|------------------------------------|
| Claude Code           | `PreToolUse` hook with `updatedInput`   | Transparent rewrite      | `hooks/rtco-rewrite.sh`             |
| VS Code Copilot Chat  | `PreToolUse` hook with `updatedInput`   | Transparent rewrite      | `.github/hooks/rtco-rewrite.json`   |
| GitHub Copilot CLI    | `PreToolUse` deny-with-suggestion       | Denial + retry           | `.github/hooks/rtco-rewrite.json`   |
| OpenCode              | Plugin `tool.execute.before`            | Transparent rewrite      | `hooks/opencode-rtco.ts`            |
| (any)                 | Custom instructions                     | Prompt-level guidance    | `.github/copilot-instructions.md`  |
