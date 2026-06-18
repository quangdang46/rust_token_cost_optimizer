# RTCO Installation Guide - For AI Coding Assistants

## ⚠️ Name Collision Warning

**There are TWO completely different projects named "rtco":**

1. ✅ **Rust Token Killer** (this project) - LLM token optimizer
   - Repos: `rtco-ai/rtco`
   - Has `rtco gain` command for token savings stats

2. ❌ **Rust Type Kit** (reachingforthejack/rtco) - DIFFERENT PROJECT
   - Rust codebase query tool and type generator
   - **DO NOT install if you want token optimization**

## Pre-Installation Check (REQUIRED)

**AI assistants should ALWAYS verify if RTCO is already installed before attempting installation.**

```bash
# Check if RTCO is installed
rtco --version

# CRITICAL: Verify it's the Token Killer (not Type Kit)
rtco gain    # Should show token savings stats, NOT "command not found"

# Check installation path
which rtco
```

If `rtco gain` works, you have the **correct** RTCO installed. **DO NOT reinstall**. Skip to "Project Initialization".

If `rtco gain` fails but `rtco --version` succeeds, you have the **wrong** RTCO (Type Kit). Uninstall and reinstall the correct one (see below).

## Installation (only if RTCO not available or wrong RTCO installed)

### Step 0: Uninstall Wrong RTCO (if needed)

If you accidentally installed Rust Type Kit:

```bash
cargo uninstall rtco
```

### Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/master/install.sh | sh
```

After installation, **verify you have the correct rtco**:
```bash
rtco gain  # Must show token savings stats (not "command not found")
```

### Auto-Configure MCP + Hooks in Every Detected Provider

The installer can register `rtco` as an MCP server and install the
`rtco-rewrite` hook into every AI provider whose config file already
exists on disk. This is opt-in — pass `--with-mcp` and/or `--with-hooks`
to enable.

```bash
# Probe every known provider (Claude, Cursor, Cline, Windsurf, Copilot,
# OpenCode, Codex, Gemini, Amazon Q, Warp) and register MCP+hooks
# wherever a config file is present.
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/master/install.sh \
  | bash -s -- --with-mcp --with-hooks --all-providers
```

| Flag | Effect |
|---|---|
| `--with-mcp` | Register `rtco` as an MCP server in every detected provider config. |
| `--no-mcp` | Skip the MCP auto-config step (default if neither `--with-mcp` nor `--all-providers` is set). |
| `--with-hooks` | Install `rtco-rewrite` hooks in every detected provider config. |
| `--no-hooks` | Skip the hooks auto-config step. |
| `--provider claude,cursor` | Restrict the set to a comma-separated provider list. |
| `--all-providers` | Probe every known provider regardless of `--provider`. |
| `--dry-run` | Print the actions that would be taken; do not modify any files. |

**Per-provider config file paths** the installer probes:

| Provider | File | Format | Key |
|---|---|---|---|
| Claude Code | `~/.claude.json` | JSON | `mcpServers` |
| Cursor | `~/.cursor/mcp.json` | JSON | `mcpServers` |
| Cline | `~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json` (macOS) | JSON | `mcpServers` |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` | JSON | `mcpServers` |
| VS Code Copilot | `~/.vscode/mcp.json` | JSON | `servers` |
| OpenCode | `~/.opencode.json` (or `~/.config/opencode/.opencode.json`) | JSON | `mcpServers` |
| Codex CLI | `~/.codex/config.toml` | TOML | `[mcp_servers.rtco]` |
| Gemini CLI | `~/.gemini/settings.json` | JSON | `mcpServers` |
| Amazon Q | `~/.aws/amazonq/mcp.json` | JSON | `mcpServers` |
| Warp | `./.warp/.mcp.json` | JSON | `mcpServers` |

Each file is backed up to `<file>.rtco.bak` before any change. Re-run the
installer with `--uninstall` to remove the rtco entry from every
detected provider.

The same flags exist on the PowerShell installer:

```powershell
irm https://raw.githubusercontent.com/rtco-ai/rtco/master/install.ps1 | iex
# then re-run with:
.\install.ps1 -WithMcp -WithHooks -AllProviders
```


### Alternative: Manual Installation

```bash
# From rtco-ai repository (NOT reachingforthejack!)
cargo install --git https://github.com/rtco-ai/rtco

# OR (if published and correct on crates.io)
cargo install rtco

# ALWAYS VERIFY after installation
rtco gain  # MUST show token savings, not "command not found"
```

⚠️ **WARNING**: `cargo install rtco` from crates.io might install the wrong package. Always verify with `rtco gain`.

## Project Initialization

### Which mode to choose?

```
  Do you want RTCO active across ALL Claude Code projects?
  │
  ├─ YES → rtco init -g              (recommended)
  │         Hook + RTCO.md (~10 tokens in context)
  │         Commands auto-rewritten transparently
  │
  ├─ YES, minimal → rtco init -g --hook-only
  │         Hook only, nothing added to CLAUDE.md
  │         Zero tokens in context
  │
  └─ NO, single project → rtco init
            Local CLAUDE.md only (137 lines)
            No hook, no global effect
```

### Recommended: Global Hook-First Setup

**Best for: All projects, automatic RTCO usage**

```bash
rtco init -g
# → Installs hook to ~/.claude/hooks/rtco-rewrite.sh
# → Creates ~/.claude/RTCO.md (10 lines, meta commands only)
# → Adds @RTCO.md reference to ~/.claude/CLAUDE.md
# → Prompts: "Patch settings.json? [y/N]"
# → If yes: patches + creates backup (~/.claude/settings.json.bak)

# Automated alternatives:
rtco init -g --auto-patch    # Patch without prompting
rtco init -g --no-patch      # Print manual instructions instead

# Verify installation
rtco init --show  # Check hook is installed and executable
```

**Token savings**: ~99.5% reduction (2000 tokens → 10 tokens in context)

**What is settings.json?**
Claude Code's hook registry. RTCO adds a PreToolUse hook that rewrites commands transparently. Without this, Claude won't invoke the hook automatically.

```
  Claude Code          settings.json        rtco-rewrite.sh        RTCO binary
       │                    │                     │                    │
       │  "git status"      │                     │                    │
       │ ──────────────────►│                     │                    │
       │                    │  PreToolUse trigger  │                    │
       │                    │ ───────────────────►│                    │
       │                    │                     │  rewrite command   │
       │                    │                     │  → rtco git status  │
       │                    │◄────────────────────│                    │
       │                    │  updated command     │                    │
       │                    │                                          │
       │  execute: rtco git status                                      │
       │ ─────────────────────────────────────────────────────────────►│
       │                                                               │  filter
       │  "3 modified, 1 untracked ✓"                                  │
       │◄──────────────────────────────────────────────────────────────│
```

**Backup Safety**:
RTCO backs up existing settings.json before changes. Restore if needed:
```bash
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

### Alternative: Local Project Setup

**Best for: Single project without hook**

```bash
cd /path/to/your/project
rtco init  # Creates ./CLAUDE.md with full RTCO instructions (137 lines)
```

**Token savings**: Instructions loaded only for this project

### Upgrading from Previous Version

#### From old 137-line CLAUDE.md injection (pre-0.22)

```bash
rtco init -g  # Automatically migrates to hook-first mode
# → Removes old 137-line block
# → Installs hook + RTCO.md
# → Adds @RTCO.md reference
```

#### From old hook with inline logic (pre-0.24) — ⚠️ Breaking Change

RTCO 0.24.0 replaced the inline command-detection hook (~200 lines) with a **thin delegator** that calls `rtco rewrite`. The binary now contains the rewrite logic, so adding new commands no longer requires a hook update.

The old hook still works but won't benefit from new rules added in future releases.

```bash
# Upgrade hook to thin delegator
rtco init --global

# Verify the new hook is active
rtco init --show
# Should show: ✅ Hook: ... (thin delegator, up to date)
```

## Common User Flows

### First-Time User (Recommended)
```bash
# 1. Install RTCO
cargo install --git https://github.com/rtco-ai/rtco
rtco gain  # Verify (must show token stats)

# 2. Setup with prompts
rtco init -g
# → Answer 'y' when prompted to patch settings.json
# → Creates backup automatically

# 3. Restart Claude Code
# 4. Test: git status (should use rtco)
```

### CI/CD or Automation
```bash
# Non-interactive setup (no prompts)
rtco init -g --auto-patch

# Verify in scripts
rtco init --show | grep "Hook:"
```

### Conservative User (Manual Control)
```bash
# Get manual instructions without patching
rtco init -g --no-patch

# Review printed JSON snippet
# Manually edit ~/.claude/settings.json
# Restart Claude Code
```

### Temporary Trial
```bash
# Install hook
rtco init -g --auto-patch

# Later: remove everything
rtco init -g --uninstall

# Restore backup if needed
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

## Installation Verification

```bash
# Basic test
rtco ls .

# Test with git
rtco git status

# Test with pnpm
rtco pnpm list

# Test with Vitest
rtco vitest
```

## Uninstalling

### Complete Removal (Global Installations Only)

```bash
# Complete removal (global installations only)
rtco init -g --uninstall

# What gets removed:
#   - Hook: ~/.claude/hooks/rtco-rewrite.sh
#   - Context: ~/.claude/RTCO.md
#   - Reference: @RTCO.md line from ~/.claude/CLAUDE.md
#   - Registration: RTCO hook entry from settings.json

# Restart Claude Code after uninstall
```

**For Local Projects**: Manually remove RTCO block from `./CLAUDE.md`

### Binary Removal

```bash
# If installed via cargo
cargo uninstall rtco

# If installed via package manager
brew uninstall rtco          # macOS Homebrew
sudo apt remove rtco         # Debian/Ubuntu
sudo dnf remove rtco         # Fedora/RHEL
```

### Restore from Backup (if needed)

```bash
cp ~/.claude/settings.json.bak ~/.claude/settings.json
```

## Essential Commands

### Files
```bash
rtco ls .              # Compact tree view
rtco read file.rs      # Optimized reading
rtco grep "pattern" .  # Grouped search results
```

### Git
```bash
rtco git status        # Compact status
rtco git log -n 10     # Condensed logs
rtco git diff          # Optimized diff
rtco git add .         # → "ok ✓"
rtco git commit -m "msg"  # → "ok ✓ abc1234"
rtco git push          # → "ok ✓ main"
```

### Pnpm (fork only)
```bash
rtco pnpm list     # Dependency tree (-70% tokens)
rtco pnpm outdated # Available updates (-80-90%)
rtco pnpm install  # Silent installation
```

### Tests
```bash
rtco cargo test      # Filtered Cargo test output (-90%)
rtco go test         # Filtered Go tests (NDJSON, -90%)
rtco jest            # Filtered Jest output (-99.6%)
rtco vitest          # Filtered Vitest output (-99.6%)
rtco playwright test # Filtered Playwright output (-94%)
rtco pytest          # Filtered Python tests (-90%)
rtco rake test       # Filtered Ruby tests (-90%)
rtco rspec           # Filtered RSpec tests (-60%)
rtco test <cmd>      # Generic test wrapper - failures only (-90%)
```

### Statistics
```bash
rtco gain              # Token savings
rtco gain --graph      # With ASCII graph
rtco gain --history    # With command history
```

## Validated Token Savings

### Production T3 Stack Project
| Operation | Standard | RTCO | Reduction |
|-----------|----------|-----|-----------|
| `vitest` | 102,199 chars | 377 chars | **-99.6%** |
| `git status` | 529 chars | 217 chars | **-59%** |
| `pnpm list` | ~8,000 tokens | ~2,400 | **-70%** |
| `pnpm outdated` | ~12,000 tokens | ~1,200-2,400 | **-80-90%** |

### Typical Claude Code Session (30 min)
- **Without RTCO**: ~150,000 tokens
- **With RTCO**: ~45,000 tokens
- **Savings**: **70% reduction**

## Troubleshooting

### RTCO command not found after installation
```bash
# Check PATH
echo $PATH | grep -o '[^:]*\.cargo[^:]*'

# Add to PATH if needed (~/.bashrc or ~/.zshrc)
export PATH="$HOME/.cargo/bin:$PATH"

# Reload shell
source ~/.bashrc  # or source ~/.zshrc
```

### RTCO command not available (e.g., vitest)
```bash
# Check branch
cd /path/to/rtco
git branch

# Switch to feat/vitest-support if needed
git checkout feat/vitest-support

# Reinstall
cargo install --path . --force
```

### Compilation error
```bash
# Update Rust
rustup update stable

# Clean and recompile
cargo clean
cargo build --release
cargo install --path . --force
```

## Support and Contributing

- **Website**: https://www.rtco-ai.app
- **Contact**: contact@rtco-ai.app
- **Troubleshooting**: See [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) for common issues
- **GitHub issues**: https://github.com/rtco-ai/rtco/issues
- **Pull Requests**: https://github.com/rtco-ai/rtco/pulls

⚠️ **If you installed the wrong rtco (Type Kit)**, see [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md#problem-rtco-gain-command-not-found)

## AI Assistant Checklist

Before each session:

- [ ] Verify RTCO is installed: `rtco --version`
- [ ] If not installed → follow "Install from fork"
- [ ] If project not initialized → `rtco init`
- [ ] Use `rtco` for ALL git/pnpm/test/vitest commands
- [ ] Check savings: `rtco gain`

**Golden Rule**: AI coding assistants should ALWAYS use `rtco` as a proxy for shell commands that generate verbose output (git, pnpm, npm, cargo test, vitest, docker, kubectl).
