# rtco — Rust Token Killer

<p align="center">
  <img src="https://avatars.githubusercontent.com/u/258253854?v=4" alt="rtco - Rust Token Killer" width="500">
</p>

<p align="center">
  <strong>High-performance CLI proxy that reduces LLM token consumption by 60–90%</strong>
</p>

<p align="center">
  <a href="https://github.com/rtco-ai/rtco/actions"><img src="https://github.com/rtco-ai/rtco/workflows/Security%20Check/badge.svg" alt="CI"></a>
  <a href="https://github.com/rtco-ai/rtco/releases"><img src="https://img.shields.io/github/v/release/rtco-ai/rtco" alt="Release"></a>
  <a href="https://opensource.org/licenses/Apache-2.0"><img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="License: Apache 2.0"></a>
  <a href="https://discord.gg/rtco"><img src="https://img.shields.io/discord/1470188214710046894?label=Discord&logo=discord" alt="Discord"></a>
  <a href="https://formulae.brew.sh/formula/rtco"><img src="https://img.shields.io/homebrew/v/rtco" alt="Homebrew"></a>
</p>

<p align="center">
  <a href="https://www.rtco-ai.app">Website</a> &bull;
  <a href="#installation">Install</a> &bull;
  <a href="https://www.rtco-ai.app/guide/troubleshooting">Troubleshooting</a> &bull;
  <a href="docs/contributing/ARCHITECTURE.md">Architecture</a> &bull;
  <a href="https://discord.gg/rtco">Discord</a>
</p>

---

**rtco** filters and compresses command outputs before they reach your LLM context. Single Rust binary, 100+ supported commands, <10ms overhead.

## Token Savings (30-min Claude Code Session)

| Operation | Frequency | Standard | rtco | Savings |
|-----------|-----------|----------|------|---------|
| `ls` / `tree` | 10x | 2,000 | 400 | -80% |
| `cat` / `read` | 20x | 40,000 | 12,000 | -70% |
| `grep` / `rg` | 8x | 16,000 | 3,200 | -80% |
| `git status` | 10x | 3,000 | 600 | -80% |
| `git diff` | 5x | 10,000 | 2,500 | -75% |
| `git log` | 5x | 2,500 | 500 | -80% |
| `git add/commit/push` | 8x | 1,600 | 120 | -92% |
| `cargo test` / `npm test` | 5x | 25,000 | 2,500 | -90% |
| `ruff check` | 3x | 3,000 | 600 | -80% |
| `pytest` | 4x | 8,000 | 800 | -90% |
| `go test` | 3x | 6,000 | 600 | -90% |
| `docker ps` | 3x | 900 | 180 | -80% |
| **Total** | | **~118,000** | **~23,900** | **-80%** |

> Estimates based on medium-sized TypeScript/Rust projects. Actual savings vary by project size.

## The Problem

AI coding assistants (Claude Code, Cursor, Copilot, etc.) execute hundreds of shell commands per session. Each command's output — stderr, stdout, compiler warnings, test failures — fills your context window with noise you don't need.

A single `cargo test` run can generate 200+ lines when you only need "2 failed". A `git status` is 15 lines when 1 line suffices. An `ls -la` is 800 tokens when 150 would do.

**That's 60–90% of every token budget wasted on machine output.**

## The Solution

rtco intercepts shell commands and applies four strategies before output reaches your LLM:

1. **Smart Filtering** — removes boilerplate, comments, whitespace noise
2. **Grouping** — aggregates similar items (files by directory, errors by type)
3. **Truncation** — keeps relevant context, cuts redundancy
4. **Deduplication** — collapses repeated log lines with counts

```
Without rtco:                              With rtco:
Claude  --git status-->  shell  -->  git    Claude  --git status-->  rtco  -->  git
  ^                                  |       ^                        |          |
  |        ~2,000 tokens (raw)       |       |   ~200 tokens         | filter   |
  +----------------------------------+       +------- (filtered) -----+----------+
```

## Why Use rtco?

| Feature | Description |
|---------|-------------|
| **60–90% token savings** | Real numbers on real workloads |
| **Single binary** | No Node.js, no Python, no dependencies |
| **<10ms overhead** | Transparent to your workflow |
| **100+ commands** | git, cargo, npm, pnpm, pytest, go test, docker, kubectl, aws, and more |
| **Auto-rewrite hook** | Zero effort — commands rewritten automatically |
| **Analytics** | See exactly how much you've saved with `rtco gain` |
| **SQLite tracking** | Local-only, private, no cloud dependency |
| **17 AI agents** | Claude Code, Copilot, Cursor, Gemini CLI, Codex, and more |

## Quick Example

```bash
# Commands are transparently rewritten — you don't change anything
git status        # → rtco git status (1 line, not 15)
cargo test        # → rtco cargo test (failures only, not 200+ lines)
npm install       # → rtco pnpm install (compact output)

# Or call rtco directly
rtco git log -n 5
rtco pytest
rtco docker ps

# See your savings
rtco gain         # Token savings summary
rtco gain --graph # ASCII graph (last 30 days)
```

## Installation

### Homebrew (recommended)

```bash
brew install rtco
```

### Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/refs/heads/master/install.sh | sh
```

> Installs to `~/.local/bin`. Add to PATH if needed:
> ```bash
> echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc
> ```

### Cargo

```bash
cargo install --git https://github.com/rtco-ai/rtco
```

### Pre-built Binaries

Download from [releases](https://github.com/rtco-ai/rtco/releases):
- macOS: `rtco-x86_64-apple-darwin.tar.gz` / `rtco-aarch64-apple-darwin.tar.gz`
- Linux: `rtco-x86_64-unknown-linux-musl.tar.gz` / `rtco-aarch64-unknown-linux-gnu.tar.gz`
- Windows: `rtco-x86_64-pc-windows-msvc.zip`

> **Windows users**: Extract the zip and place `rtco.exe` somewhere in your PATH (e.g. `C:\Users\<you>\.local\bin`). Run rtco from **Command Prompt**, **PowerShell**, or **Windows Terminal** — do not double-click the `.exe` (it will flash and close). For the best experience, use [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) where the full hook system works natively. See [Windows setup](#windows) below for details.

### Verify Installation

```bash
rtco --version   # Should show "rtco 0.28.2" or newer
rtco gain        # Should show token savings stats
```

## Quick Start

```bash
# 1. Install the auto-rewrite hook for your AI tool
rtco init -g                     # Claude Code / Copilot (default)
rtco init -g --gemini            # Gemini CLI
rtco init -g --codex             # Codex (OpenAI)
rtco init -g --agent cursor      # Cursor
rtco init --agent windsurf       # Windsurf
rtco init --agent cline          # Cline / Roo Code
rtco init --agent kilocode       # Kilo Code
rtco init --agent hermes         # Hermes

# 2. Restart your AI tool, then use it normally
git status  # Automatically rewritten to rtco git status
```

Hook-based agents rewrite Bash commands (e.g., `git status` → `rtco git status`) before execution. Plugin-based agents, including Hermes, use their plugin API to rewrite commands before execution. The agent receives compact output without needing to call `rtco` explicitly.

**Important:** the hook only runs on Bash tool calls. Claude Code built-in tools like `Read`, `Grep`, and `Glob` do not pass through the Bash hook, so they are not auto-rewritten. To get rtco's compact output for those workflows, use shell commands (`cat`/`head`/`tail`, `rg`/`grep`, `find`) or call `rtco read`, `rtco grep`, or `rtco find` directly.

## Commands

### Files
```bash
rtco ls .                        # Token-optimized directory tree
rtco read file.rs                # Smart file reading
rtco read file.rs -l aggressive  # Signatures only (strips bodies)
rtco smart file.rs               # 2-line heuristic code summary
rtco find "*.rs" .               # Compact find results
rtco grep "pattern" .            # Grouped search results
rtco diff file1 file2            # Condensed diff
```

### Git
```bash
rtco git status                  # Compact status
rtco git log -n 10               # One-line commits
rtco git diff                    # Condensed diff
rtco git add                     # -> "ok"
rtco git commit -m "msg"         # -> "ok abc1234"
rtco git push                    # -> "ok main"
rtco git pull                    # -> "ok 3 files +10 -2"
```

### GitHub CLI
```bash
rtco gh pr list                  # Compact PR listing
rtco gh pr view 42               # PR details + checks
rtco gh issue list               # Compact issue listing
rtco gh run list                 # Workflow run status
```

### Test Runners
```bash
rtco jest                        # Jest compact (failures only)
rtco vitest                      # Vitest compact (failures only)
rtco playwright test             # E2E results (failures only)
rtco pytest                      # Python tests (-90%)
rtco go test                     # Go tests (NDJSON, -90%)
rtco cargo test                  # Cargo tests (-90%)
rtco rake test                   # Ruby minitest (-90%)
rtco rspec                       # RSpec tests (JSON, -60%+)
rtco err <cmd>                   # Filter errors only from any command
rtco test <cmd>                  # Generic test wrapper - failures only (-90%)
```

### Build & Lint
```bash
rtco lint                        # ESLint grouped by rule/file
rtco lint biome                  # Supports other linters
rtco tsc                         # TypeScript errors grouped by file
rtco next build                  # Next.js build compact
rtco prettier --check .          # Files needing formatting
rtco cargo build                 # Cargo build (-80%)
rtco cargo clippy                # Cargo clippy (-80%)
rtco ruff check                  # Python linting (JSON, -80%)
rtco golangci-lint run           # Go linting (JSON, -85%)
rtco rubocop                     # Ruby linting (JSON, -60%+)
```

### Package Managers
```bash
rtco pnpm list                   # Compact dependency tree
rtco pip list                    # Python packages (auto-detect uv)
rtco pip outdated                # Outdated packages
rtco bundle install              # Ruby gems (strip Using lines)
rtco prisma generate             # Schema generation (no ASCII art)
rtco fnm use 20                  # Node version switch (ANSI/progress stripped)
```

### AWS
```bash
rtco aws sts get-caller-identity # One-line identity
rtco aws ec2 describe-instances  # Compact instance list
rtco aws lambda list-functions   # Name/runtime/memory (strips secrets)
rtco aws logs get-log-events     # Timestamped messages only
rtco aws cloudformation describe-stack-events  # Failures first
rtco aws dynamodb scan           # Unwraps type annotations
rtco aws iam list-roles          # Strips policy documents
rtco aws s3 ls                   # Truncated with tee recovery
```

### Containers
```bash
rtco docker ps                   # Compact container list
rtco docker images               # Compact image list
rtco docker logs <container>     # Deduplicated logs
rtco docker compose ps           # Compose services
rtco kubectl pods                # Compact pod list
rtco kubectl logs <pod>          # Deduplicated logs
rtco kubectl services            # Compact service list
```

### Data & Analytics
```bash
rtco json config.json            # Structure without values
rtco deps                        # Dependencies summary
rtco env -f AWS                  # Filtered env vars
rtco log app.log                 # Deduplicated logs
rtco curl <url>                  # Truncate + save full output
rtco wget <url>                  # Download, strip progress bars
rtco summary <long command>      # Heuristic summary
rtco proxy <command>             # Raw passthrough + tracking
```

### Token Savings Analytics
```bash
rtco gain                        # Summary stats
rtco gain --graph                # ASCII graph (last 30 days)
rtco gain --history              # Recent command history
rtco gain --daily                # Day-by-day breakdown
rtco gain --all --format json    # JSON export for dashboards

rtco discover                    # Find missed savings opportunities
rtco discover --all --since 7    # All projects, last 7 days

rtco session                     # Show rtco adoption across recent sessions
```

## Global Flags

```bash
-u, --ultra-compact    # ASCII icons, inline format (extra token savings)
-v, --verbose          # Increase verbosity (-v, -vv, -vvv)
```

## Auto-Rewrite Hook

The most effective way to use rtco. The hook transparently intercepts Bash commands and rewrites them to rtco equivalents before execution.

**Result**: 100% rtco adoption across all conversations and subagents, zero token overhead.

**Scope note:** this only applies to Bash tool calls. Claude Code built-in tools such as `Read`, `Grep`, and `Glob` bypass the hook, so use shell commands or explicit `rtco` commands when you want rtco filtering there.

### Setup

```bash
rtco init -g                 # Install hook + rtco.md (recommended)
rtco init -g --opencode     # OpenCode plugin (instead of Claude Code)
rtco init -g --auto-patch   # Non-interactive (CI/CD)
rtco init -g --hook-only    # Hook only, no rtco.md
rtco init --show            # Verify installation
```

After install, **restart your AI tool**.

## Windows

rtco works on Windows with some limitations. The auto-rewrite hook requires a Unix shell, so on native Windows rtco falls back to **CLAUDE.md injection mode** — your AI assistant receives rtco instructions but commands are not rewritten automatically.

### Recommended: WSL (full support)

For the best experience, use [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) (Windows Subsystem for Linux). Inside WSL, rtco works exactly like Linux — full hook support, auto-rewrite, everything:

```bash
# Inside WSL
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/refs/heads/master/install.sh | sh
rtco init -g
```

### Native Windows (limited support)

On native Windows (cmd.exe / PowerShell), rtco filters work but the hook does not auto-rewrite commands:

```powershell
# 1. Download and extract rtco-x86_64-pc-windows-msvc.zip from releases
# 2. Add rtco.exe to your PATH
# 3. Initialize (falls back to CLAUDE.md injection)
rtco init -g
# 4. Use rtco explicitly
rtco cargo test
rtco git status
```

**Important**: Do not double-click `rtco.exe` — it is a CLI tool that prints usage and exits immediately. Always run it from a terminal (Command Prompt, PowerShell, or Windows Terminal).

| Feature | WSL | Native Windows |
|---------|-----|----------------|
| Filters (cargo, git, etc.) | Full | Full |
| Auto-rewrite hook | Yes | No (CLAUDE.md fallback) |
| `rtco init -g` | Hook mode | CLAUDE.md mode |
| `rtco gain` / analytics | Full | Full |

## Supported AI Tools

rtco supports 17 AI coding tools. Each integration rewrites shell commands to `rtco` equivalents for 60–90% token savings where the agent supports command interception.

| Tool | Install | Method |
|------|---------|--------|
| **Claude Code** | `rtco init -g` | PreToolUse hook (bash) |
| **GitHub Copilot (VS Code)** | `rtco init -g --copilot` | PreToolUse hook — transparent rewrite |
| **GitHub Copilot CLI** | `rtco init -g --copilot` | PreToolUse deny-with-suggestion (CLI limitation) |
| **Cursor** | `rtco init -g --agent cursor` | preToolUse hook (hooks.json) |
| **Gemini CLI** | `rtco init -g --gemini` | BeforeTool hook |
| **Google Antigravity CLI (`agy`)** | `rtco init -g --agent agy` | PreToolUse hook — deny-with-reason (global) |
| **Codex** | `rtco init -g --codex` | AGENTS.md + rtco.md instructions |
| **Windsurf** | `rtco init --agent windsurf` | .windsurfrules (project-scoped) |
| **Cline / Roo Code** | `rtco init --agent cline` | .clinerules (project-scoped) |
| **OpenCode** | `rtco init -g --opencode` | Plugin TS (tool.execute.before) |
| **OpenClaw** | `openclaw plugins install ./openclaw` | Plugin TS (before_tool_call) |
| **Hermes** | `rtco init --agent hermes` | Python plugin adapter (terminal command mutation via `rtco rewrite`) |
| **Kilo Code** | `rtco init --agent kilocode` | .kilocode/rules/rtco-rules.md (project-scoped) |

For per-agent setup details, override controls, and graceful degradation, see the [Supported Agents guide](https://www.rtco-ai.app/guide/getting-started/supported-agents). The Hermes plugin source and tests live in `hooks/hermes/`; installed Hermes runtime files still live under `~/.hermes/plugins/rtco-rewrite/`.

## Configuration

`~/.config/rtco/config.toml` (macOS: `~/Library/Application Support/rtco/config.toml`):

```toml
[hooks]
exclude_commands = ["curl", "playwright"]  # skip rewrite for these

[tee]
enabled = true          # save raw output on failure (default: true)
mode = "failures"       # "failures", "always", or "never"
```

When a command fails, rtco saves the full unfiltered output so the LLM can read it without re-executing:

```
FAILED: 2/15 tests
[full output: ~/.local/share/rtco/tee/1707753600_cargo_test.log]
```

For the full config reference (all sections, env vars, per-project filters), see the [Configuration guide](https://www.rtco-ai.app/guide/getting-started/configuration).

### Uninstall

```bash
rtco init -g --uninstall     # Remove hook, rtco.md, settings.json entry
cargo uninstall rtco          # Remove binary
brew uninstall rtco           # If installed via Homebrew
```

## Design Philosophy

**1. Transparency above all** — The hook should be invisible. You use your AI tool exactly as before; rtco compresses the output. If something breaks, the original command runs unchanged.

**2. Fail open** — When rtco can't parse or filter a command's output, it passes the raw output through unchanged. You never lose information; you just don't save tokens on that command.

**3. Local-only by default** — No telemetry, no cloud sync, no account required. Your data stays on your machine. Telemetry is opt-in only.

**4. Speed is a feature** — <10ms overhead means the hook never slows your workflow. If filtering takes longer than the command itself, something is wrong.

**5. One binary, no dependencies** — rtco is a single Rust binary that runs everywhere. No Node.js, no Python, no Docker. If you can run shell commands, you can run rtco.

## Comparison vs Alternatives

| Feature | rtco | tldr | glow | 
|---------|------|------|------|
| Token savings | 60–90% | N/A | N/A |
| Auto-rewrite hook | ✅ | ❌ | ❌ |
| Analytics (`rtco gain`) | ✅ | ❌ | ❌ |
| SQLite tracking | ✅ | ❌ | ❌ |
| 100+ commands | ✅ | ~15 | ~5 |
| Single binary | ✅ | ❌ | ❌ |
| <10ms overhead | ✅ | ✅ | ✅ |
| Claude/Copilot/Cursor integration | ✅ | ❌ | ❌ |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Claude / Cursor / etc.                        │
└─────────────────────────────┬───────────────────────────────────────┘
                              │ Bash command (e.g. "git status")
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      rtco auto-rewrite hook                         │
│  ┌─────────────┐    ┌──────────────┐    ┌────────────────────────┐  │
│  │ PreToolUse  │ →  │ rewrite.sh   │ →  │ "git status" →        │  │
│  │ (intercept) │    │ (jq + sed)   │    │ "rtco git status"     │  │
│  └─────────────┘    └──────────────┘    └────────────────────────┘  │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  shell + git   │
                    └────────┬────────┘
                             │ raw output (~2000 tokens)
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         rtco filter chain                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────┐  │
│  │   filter     │→ │   group      │→ │  truncate    │→ │ dedup  │  │
│  │ (remove noise)│  │ (aggregate) │  │ (cut tail)   │  │(counts)│  │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────┘  │
│                                                                  │
│                          ~200 tokens                              │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         SQLite tracking                             │
│                     ~/.local/share/rtco/rtco.db                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Troubleshooting

### rtco command not found after install

```bash
# Check if rtco is installed
rtco --version

# If not, find where it was installed
which rtco
ls ~/.local/bin/rtco

# Add to PATH if needed
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Hook not rewriting commands

```bash
# Verify hook is installed
rtco init --show

# Check Claude Code's hook log
cat ~/.local/share/rtco/hooks/hook.log

# Reinstall if needed
rtco init -g --uninstall
rtco init -g
```

### rtco gain shows 0 savings

```bash
# Check that tracking DB exists
ls ~/.local/share/rtco/

# Check command history
rtco gain --history

# Verify commands are being rewritten (run with -vvv)
RTK_HOOK_AUDIT=1 git status
```

### Name collision: "rtk" vs "rtco"

> **Warning**: Another package named "rtk" (Rust Type Kit) exists on crates.io. If `rtk gain` fails or shows unexpected behavior, you have the wrong package installed.

```bash
# Verify you're using the correct binary
which rtco
rtco --version   # Should be >= 0.28.2

# If wrong package, uninstall and reinstall via git
cargo uninstall rtk 2>/dev/null
cargo install --git https://github.com/rtco-ai/rtco
```

### Build fails on Windows

- Use **WSL** for full hook support
- On native Windows, the hook falls back to CLAUDE.md mode — filtering works, auto-rewrite does not
- Do not double-click `rtco.exe` — always run from a terminal

## Limitations

**1. Bash tool calls only** — Claude Code's built-in tools (Read, Grep, Glob) bypass the Bash hook and are not rewritten. Use shell commands (`cat`, `rg`, `find`) or call `rtco` directly for those workflows.

**2. Hook requires Unix shell** — On native Windows (cmd.exe/PowerShell), the hook doesn't work. Use WSL for full functionality, or fall back to CLAUDE.md injection mode.

**3. Some commands pass through unfiltered** — If rtco can't parse a command's output format, it passes the raw output unchanged. You never lose information; you just don't save tokens on that specific command.

**4. Projects with non-standard output formats** — Heavily customized build tools or internal CLIs may produce output formats rtco doesn't recognize. Use `rtco proxy <cmd>` to bypass filtering while still tracking usage.

**5. Telemetry is opt-in** — rtco collects no data by default. If you want to help improve rtco, run `rtco telemetry enable` — but this is entirely voluntary.

## FAQ

**Q: Does rtco work with Windows?**  
A: Yes, but with limitations. WSL (Windows Subsystem for Linux) gives full functionality. Native Windows gets filtering and CLI tools but no auto-rewrite hook.

**Q: How does rtco know which commands to filter?**  
A: rtco ships with 100+ built-in command definitions (git, cargo, npm, pytest, etc.). It matches your command against these patterns and applies the appropriate filter. You can also add custom filters in `~/.config/rtco/filters.toml`.

**Q: Can I use rtco without the hook?**  
A: Yes. Call `rtco` directly (e.g., `rtco git status`) or use `rtco proxy <cmd>` to run commands through rtco's filter without modifying the original command.

**Q: Does rtco work with Claude Code's Read/Grep/Glob tools?**  
A: No — those are built-in tools that bypass Bash. Use shell commands or explicit `rtco read`/`rtco grep`/`rtco find` for those workflows.

**Q: How much faster is rtco than using tldr or glow?**  
A: rtco focuses on token savings, not just simplification. The auto-rewrite hook means zero behavior change for you while saving 60–90% tokens. tldr/glow require changing how you invoke commands.

**Q: Can I see how much I've saved?**  
A: Yes — `rtco gain` shows total savings, `rtco gain --graph` shows a 30-day chart, and `rtco gain --all --format json` exports data for dashboards.

## Privacy & Telemetry

rtco can collect **anonymous, aggregate usage metrics** once per day. Telemetry is **disabled by default** and requires **explicit opt-in consent** (GDPR Art. 6, 7) during `rtco init` or via `rtco telemetry enable`. This data helps us build a better product: identifying which commands need filters, which filters need improvement, and how much value rtco delivers. For the full list of fields, data handling, and contributor guidelines, see **[docs/TELEMETRY.md](docs/TELEMETRY.md)**.

**What is collected and why:**

| Category | Data | Why |
|----------|------|-----|
| Identity | Salted device hash (SHA-256, not reversible) | Count unique installations without tracking individuals |
| Environment | rtco version, OS, architecture, install method | Know which platforms to support and test |
| Usage volume | Command count (24h), total commands, tokens saved (24h/30d/total) | Measure adoption and value delivered |
| Quality | Top 5 passthrough commands (0% savings), parse failure count, commands with <30% savings | Identify missing filters and weak ones to improve |
| Ecosystem | Command category distribution (e.g. git 45%, cargo 20%, js 15%) | Prioritize filter development for popular ecosystems |
| Retention | Days since first use, active days in last 30 | Understand engagement and detect churn |
| Adoption | AI agent hook type (claude/gemini/codex), custom TOML filter count | Track integration coverage and DSL adoption |
| Configuration | Whether config.toml exists, number of excluded commands, project count | Understand user maturity and customization patterns |
| Features | Usage counts for meta-commands (gain, discover, proxy, verify) | Know which rtco features are valued vs unused |
| Economics | Estimated USD savings (based on API token pricing) | Quantify the value rtco provides to users |

All data is **aggregate counts or anonymized command names** (first 3 words, no arguments). Top commands report only tool names (e.g. "git", "cargo"), never full command lines.

**What is NOT collected:** source code, file paths, command arguments, secrets, environment variables, personal data, or repository contents.

**Manage telemetry:**
```bash
rtco telemetry status     # Check current consent state
rtco telemetry enable    # Give consent (interactive prompt)
rtco telemetry disable   # Withdraw consent — stops all collection immediately
rtco telemetry forget    # Withdraw consent + delete all local data + request server-side erasure
```

**Override via environment:**
```bash
export RTCO_TELEMETRY_DISABLED=1   # Blocks telemetry regardless of consent
```

## Star History

<a href="https://www.star-history.com/?repos=rtco-ai%2Frtco&type=date&legend=top-left">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=rtco-ai%2Frtco&type=date&theme=dark&legend=top-left" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=rtco-ai%2Frtco&type=date&theme=light&legend=top-left" />
    <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=rtco-ai%2Frtco&type=date&legend=top-left" />
  </picture>
</a>

## Core team

- **Patrick Szymkowiak** — Founder
  [GitHub](https://github.com/pszymkowiak) · [LinkedIn](https://www.linkedin.com/in/patrick-szymkowiak/)
- **Florian Bruniaux** — Core contributor
  [GitHub](https://github.com/FlorianBruniaux) · [LinkedIn](https://www.linkedin.com/in/florian-bruniaux-43408b83/)
- **Adrien Eppling** — Core contributor
  [GitHub](https://github.com/aeppling) · [LinkedIn](https://www.linkedin.com/in/adrien-eppling/)

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

## Disclaimer

See [DISCLAIMER.md](DISCLAIMER.md).