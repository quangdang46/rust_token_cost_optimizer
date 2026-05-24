# RTCO - Rust Token Killer

**Usage**: Token-optimized CLI proxy (60-90% savings on dev operations)

## Meta Commands (always use rtco directly)

```bash
rtco gain              # Show token savings analytics
rtco gain --history    # Show command usage history with savings
rtco discover          # Analyze Claude Code history for missed opportunities
rtco proxy <cmd>       # Execute raw command without filtering (for debugging)
```

## Installation Verification

```bash
rtco --version         # Should show: rtco X.Y.Z
rtco gain              # Should work (not "command not found")
which rtco             # Verify correct binary
```

⚠️ **Name collision**: If `rtco gain` fails, you may have reachingforthejack/rtco (Rust Type Kit) installed instead.

## Hook-Based Usage

All other commands are automatically rewritten by the Claude Code hook.
Example: `git status` → `rtco git status` (transparent, 0 tokens overhead)

Refer to CLAUDE.md for full command reference.
