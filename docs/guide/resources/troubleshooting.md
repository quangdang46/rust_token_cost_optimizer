---
title: Troubleshooting
description: Common RTCO issues and how to fix them
sidebar:
  order: 2
---

# Troubleshooting

## `rtco gain` says "not a rtco command"

**Symptom:**
```bash
$ rtco gain
rtco: 'gain' is not a rtco command. See 'rtco --help'.
```

**Cause:** You installed **Rust Type Kit** (`reachingforthejack/rtco`) instead of **Rust Token Killer** (`rtco-ai/rtco`). They share the same binary name.

**Fix:**
```bash
cargo uninstall rtco
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/master/install.sh | sh
rtco gain    # should now show token savings stats
```

## How to tell which rtco you have

| If `rtco gain`... | You have |
|------------------|----------|
| Shows token savings dashboard | Rust Token Killer ✅ |
| Returns "not a rtco command" | Rust Type Kit ❌ |

## AI assistant not using RTCO

**Symptom:** Claude Code (or another agent) runs `cargo test` instead of `rtco cargo test`.

**Checklist:**

1. Verify RTCO is installed:
   ```bash
   rtco --version
   rtco gain
   ```

2. Initialize the hook:
   ```bash
   rtco init --global    # Claude Code
   rtco init --global --cursor    # Cursor
   rtco init --global --opencode  # OpenCode
   ```

3. Restart your AI assistant.

4. Verify hook status:
   ```bash
   rtco init --show
   ```

5. Check `settings.json` has the hook registered (Claude Code):
   ```bash
   cat ~/.claude/settings.json | grep rtco
   ```

## RTCO not found after `cargo install`

**Symptom:**
```bash
$ rtco --version
zsh: command not found: rtco
```

**Cause:** `~/.cargo/bin` is not in your PATH.

**Fix:**

For bash (`~/.bashrc`) or zsh (`~/.zshrc`):
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

For fish (`~/.config/fish/config.fish`):
```fish
set -gx PATH $HOME/.cargo/bin $PATH
```

Then reload:
```bash
source ~/.zshrc    # or ~/.bashrc
rtco --version
```

## RTCO on Windows

### Double-clicking rtco.exe does nothing

**Symptom:** You double-click `rtco.exe`, a terminal flashes and closes instantly.

**Cause:** RTCO is a command-line tool. With no arguments, it prints usage and exits. The console window opens and closes before you can read anything.

**Fix:** Open a terminal first, then run RTCO from there:
- Press `Win+R`, type `cmd`, press Enter
- Or open PowerShell or Windows Terminal
- Then run: `rtco --version`

### Hook not working (no auto-rewrite)

**Symptom:** `rtco init -g` shows "Falling back to --claude-md mode" on Windows.

**Cause:** The auto-rewrite hook (`rtco-rewrite.sh`) requires a Unix shell. Native Windows doesn't have one.

**Fix:** Use [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) for full hook support:
```bash
# Inside WSL
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/refs/heads/master/install.sh | sh
rtco init -g    # full hook mode works in WSL
```

On native Windows, RTCO falls back to CLAUDE.md injection. Your AI assistant gets RTCO instructions but won't auto-rewrite commands. It can still use RTCO manually: `rtco cargo test`, `rtco git status`, etc.

### Node.js tools not found

**Symptom:**
```
rtco vitest --run
Error: program not found
```

**Cause:** On Windows, Node.js tools are installed as `.CMD`/`.BAT` wrappers. Older RTCO versions couldn't find them.

**Fix:** Update to RTCO v0.23.1+:
```bash
cargo install --git https://github.com/rtco-ai/rtco
rtco --version    # should be 0.23.1+
```

## Compilation error during installation

```bash
rustup update stable
rustup default stable
cargo clean
cargo build --release
cargo install --path . --force
```

Minimum required Rust version: 1.70+.

## OpenCode not using RTCO

```bash
rtco init --global --opencode
# restart OpenCode
rtco init --show    # should show "OpenCode: plugin installed"
```

## `cargo install rtco` installs the wrong package

If Rust Type Kit is published to crates.io under the name `rtco`, `cargo install rtco` may install the wrong one.

Always use the explicit URL:

```bash
cargo install --git https://github.com/rtco-ai/rtco
```

## Run the diagnostic script

From the RTCO repository root:

```bash
bash scripts/check-installation.sh
```

Checks:
- RTCO installed and in PATH
- Correct version (Token Killer, not Type Kit)
- Available features
- Claude Code integration
- Hook status

## Still stuck?

Open an issue: https://github.com/rtco-ai/rtco/issues
