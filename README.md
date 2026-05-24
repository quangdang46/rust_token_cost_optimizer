<p align="center">
  <img src="https://avatars.githubusercontent.com/u/258253854?v=4" alt="RTCO - Rust Token Killer" width="500">
</p>

<p align="center">
  <strong>High-performance CLI proxy that reduces LLM token consumption by 60-90%</strong>
</p>

<p align="center">
  <a href="https://github.com/rtco-ai/rtco/actions"><img src="https://github.com/rtco-ai/rtco/workflows/Security%20Check/badge.svg" alt="CI"></a>
  <a href="https://github.com/rtco-ai/rtco/releases"><img src="https://img.shields.io/github/v/release/rtco-ai/rtco" alt="Release"></a>
  <a href="https://opensource.org/licenses/Apache-2.0"><img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="License: Apache 2.0"></a>
  <a href="https://discord.gg/RySmvNF5kF"><img src="https://img.shields.io/discord/1470188214710046894?label=Discord&logo=discord" alt="Discord"></a>
  <a href="https://formulae.brew.sh/formula/rtco"><img src="https://img.shields.io/homebrew/v/rtco" alt="Homebrew"></a>
</p>

<p align="center">
  <a href="https://www.rtco-ai.app">Website</a> &bull;
  <a href="#installation">Install</a> &bull;
  <a href="https://www.rtco-ai.app/guide/troubleshooting">Troubleshooting</a> &bull;
  <a href="docs/contributing/ARCHITECTURE.md">Architecture</a> &bull;
  <a href="https://discord.gg/RySmvNF5kF">Discord</a>
</p>

<p align="center">
  <a href="README.md">English</a> &bull;
  <a href="README_fr.md">Francais</a> &bull;
  <a href="README_zh.md">中文</a> &bull;
  <a href="README_ja.md">日本語</a> &bull;
  <a href="README_ko.md">한국어</a> &bull;
  <a href="README_es.md">Espanol</a>
</p>

---

rtco filters and compresses command outputs before they reach your LLM context. Single Rust binary, 100+ supported commands, <10ms overhead.

## Token Savings (30-min Claude Code Session)

| Operation | Frequency | Standard | rtco | Savings |
|-----------|-----------|----------|-----|---------|
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

> **Windows users**: Extract the zip and place `rtco.exe` somewhere in your PATH (e.g. `C:\Users\<you>\.local\bin`). Run RTCO from **Command Prompt**, **PowerShell**, or **Windows Terminal** — do not double-click the `.exe` (it will flash and close). For the best experience, use [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) where the full hook system works natively. See [Windows setup](#windows) below for details.

### Verify Installation

```bash
rtco --version   # Should show "rtco 0.28.2"
rtco gain        # Should show token savings stats
```

> **Name collision warning**: Another project named "rtco" (Rust Type Kit) exists on crates.io. If `rtco gain` fails, you have the wrong package. Use `cargo install --git` above instead.

## Quick Start

```bash
# 1. Install for your AI tool
rtco init -g                     # Claude Code / Copilot (default)
rtco init -g --gemini            # Gemini CLI
rtco init -g --codex             # Codex (OpenAI)
rtco init -g --agent cursor      # Cursor
rtco init --agent windsurf       # Windsurf
rtco init --agent cline          # Cline / Roo Code
rtco init --agent kilocode       # Kilo Code
rtco init --agent antigravity    # Google Antigravity
rtco init --agent hermes         # Hermes

# 2. Restart your AI tool, then test
git status  # Automatically rewritten to rtco git status
```

Hook-based agents rewrite Bash commands (e.g., `git status` -> `rtco git status`) before execution. Plugin-based agents, including Hermes, use their plugin API to rewrite commands before execution. The agent receives compact output without needing to call `rtco` explicitly.

**Important:** the hook only runs on Bash tool calls. Claude Code built-in tools like `Read`, `Grep`, and `Glob` do not pass through the Bash hook, so they are not auto-rewritten. To get RTCO's compact output for those workflows, use shell commands (`cat`/`head`/`tail`, `rg`/`grep`, `find`) or call `rtco read`, `rtco grep`, or `rtco find` directly.

## How It Works

```
  Without rtco:                                    With rtco:

  Claude  --git status-->  shell  -->  git         Claude  --git status-->  RTCO  -->  git
    ^                                   |            ^                      |          |
    |        ~2,000 tokens (raw)        |            |   ~200 tokens        | filter   |
    +-----------------------------------+            +------- (filtered) ---+----------+
```

Four strategies applied per command type:

1. **Smart Filtering** - Removes noise (comments, whitespace, boilerplate)
2. **Grouping** - Aggregates similar items (files by directory, errors by type)
3. **Truncation** - Keeps relevant context, cuts redundancy
4. **Deduplication** - Collapses repeated log lines with counts

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

rtco session                     # Show RTCO adoption across recent sessions
```

## Global Flags

```bash
-u, --ultra-compact    # ASCII icons, inline format (extra token savings)
-v, --verbose          # Increase verbosity (-v, -vv, -vvv)
```

## Examples

**Directory listing:**
```
# ls -la (45 lines, ~800 tokens)        # rtco ls (12 lines, ~150 tokens)
drwxr-xr-x  15 user staff 480 ...       my-project/
-rw-r--r--   1 user staff 1234 ...       +-- src/ (8 files)
...                                      |   +-- main.rs
                                         +-- Cargo.toml
```

**Git operations:**
```
# git push (15 lines, ~200 tokens)       # rtco git push (1 line, ~10 tokens)
Enumerating objects: 5, done.             ok main
Counting objects: 100% (5/5), done.
Delta compression using up to 8 threads
...
```

**Test output:**
```
# cargo test (200+ lines on failure)     # rtco test cargo test (~20 lines)
running 15 tests                          FAILED: 2/15 tests
test utils::test_parse ... ok               test_edge_case: assertion failed
test utils::test_format ... ok              test_overflow: panic at utils.rs:18
...
```

## Auto-Rewrite Hook

The most effective way to use rtco. The hook transparently intercepts Bash commands and rewrites them to rtco equivalents before execution.

**Result**: 100% rtco adoption across all conversations and subagents, zero token overhead.

**Scope note:** this only applies to Bash tool calls. Claude Code built-in tools such as `Read`, `Grep`, and `Glob` bypass the hook, so use shell commands or explicit `rtco` commands when you want RTCO filtering there.

### Setup

```bash
rtco init -g                 # Install hook + RTCO.md (recommended)
rtco init -g --opencode      # OpenCode plugin (instead of Claude Code)
rtco init -g --auto-patch    # Non-interactive (CI/CD)
rtco init -g --hook-only     # Hook only, no RTCO.md
rtco init --show             # Verify installation
```

After install, **restart Claude Code**.

## Windows

RTCO works on Windows with some limitations. The auto-rewrite hook (`rtco-rewrite.sh`) requires a Unix shell, so on native Windows RTCO falls back to **CLAUDE.md injection mode** — your AI assistant receives RTCO instructions but commands are not rewritten automatically.

### Recommended: WSL (full support)

For the best experience, use [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) (Windows Subsystem for Linux). Inside WSL, RTCO works exactly like Linux — full hook support, auto-rewrite, everything:

```bash
# Inside WSL
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/refs/heads/master/install.sh | sh
rtco init -g
```

### Native Windows (limited support)

On native Windows (cmd.exe / PowerShell), RTCO filters work but the hook does not auto-rewrite commands:

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

RTCO supports 13 AI coding tools. Each integration rewrites shell commands to `rtco` equivalents for 60-90% token savings where the agent supports command interception.

| Tool | Install | Method |
|------|---------|--------|
| **Claude Code** | `rtco init -g` | PreToolUse hook (bash) |
| **GitHub Copilot (VS Code)** | `rtco init -g --copilot` | PreToolUse hook — transparent rewrite |
| **GitHub Copilot CLI** | `rtco init -g --copilot` | PreToolUse deny-with-suggestion (CLI limitation) |
| **Cursor** | `rtco init -g --agent cursor` | preToolUse hook (hooks.json) |
| **Gemini CLI** | `rtco init -g --gemini` | BeforeTool hook |
| **Codex** | `rtco init -g --codex` | AGENTS.md + RTCO.md instructions |
| **Windsurf** | `rtco init --agent windsurf` | .windsurfrules (project-scoped) |
| **Cline / Roo Code** | `rtco init --agent cline` | .clinerules (project-scoped) |
| **OpenCode** | `rtco init -g --opencode` | Plugin TS (tool.execute.before) |
| **OpenClaw** | `openclaw plugins install ./openclaw` | Plugin TS (before_tool_call) |
| **Hermes** | `rtco init --agent hermes` | Python plugin adapter (terminal command mutation via `rtco rewrite`) |
| **Mistral Vibe** | Planned ([#800](https://github.com/rtco-ai/rtco/issues/800)) | Blocked on upstream |
| **Kilo Code** | `rtco init --agent kilocode` | .kilocode/rules/rtco-rules.md (project-scoped) |
| **Google Antigravity** | `rtco init --agent antigravity` | .agents/rules/antigravity-rtco-rules.md (project-scoped) |

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

When a command fails, RTCO saves the full unfiltered output so the LLM can read it without re-executing:

```
FAILED: 2/15 tests
[full output: ~/.local/share/rtco/tee/1707753600_cargo_test.log]
```

For the full config reference (all sections, env vars, per-project filters), see the [Configuration guide](https://www.rtco-ai.app/guide/getting-started/configuration).

### Uninstall

```bash
rtco init -g --uninstall     # Remove hook, RTCO.md, settings.json entry
cargo uninstall rtco          # Remove binary
brew uninstall rtco           # If installed via Homebrew
```

## Documentation

- **[rtco-ai.app/guide](https://www.rtco-ai.app/guide)** — full user guide (installation, supported agents, what gets optimized, analytics, configuration, troubleshooting)
- **[INSTALL.md](INSTALL.md)** — detailed installation reference
- **[ARCHITECTURE.md](docs/contributing/ARCHITECTURE.md)** — system design and technical decisions
- **[CONTRIBUTING.md](CONTRIBUTING.md)** — contribution guide
- **[SECURITY.md](SECURITY.md)** — security policy

## Privacy & Telemetry

RTCO can collect **anonymous, aggregate usage metrics** once per day. Telemetry is **disabled by default** and requires **explicit opt-in consent** (GDPR Art. 6, 7) during `rtco init` or via `rtco telemetry enable`. This data helps us build a better product: identifying which commands need filters, which filters need improvement, and how much value RTCO delivers. For the full list of fields, data handling, and contributor guidelines, see **[docs/TELEMETRY.md](docs/TELEMETRY.md)**.

**What is collected and why:**

| Category | Data | Why |
|----------|------|-----|
| Identity | Salted device hash (SHA-256, not reversible) | Count unique installations without tracking individuals |
| Environment | RTCO version, OS, architecture, install method | Know which platforms to support and test |
| Usage volume | Command count (24h), total commands, tokens saved (24h/30d/total) | Measure adoption and value delivered |
| Quality | Top 5 passthrough commands (0% savings), parse failure count, commands with <30% savings | Identify missing filters and weak ones to improve |
| Ecosystem | Command category distribution (e.g. git 45%, cargo 20%, js 15%) | Prioritize filter development for popular ecosystems |
| Retention | Days since first use, active days in last 30 | Understand engagement and detect churn |
| Adoption | AI agent hook type (claude/gemini/codex), custom TOML filter count | Track integration coverage and DSL adoption |
| Configuration | Whether config.toml exists, number of excluded commands, project count | Understand user maturity and customization patterns |
| Features | Usage counts for meta-commands (gain, discover, proxy, verify) | Know which RTCO features are valued vs unused |
| Economics | Estimated USD savings (based on API token pricing) | Quantify the value RTCO provides to users |

All data is **aggregate counts or anonymized command names** (first 3 words, no arguments). Top commands report only tool names (e.g. "git", "cargo"), never full command lines.

**What is NOT collected:** source code, file paths, command arguments, secrets, environment variables, personal data, or repository contents.

**Manage telemetry:**
```bash
rtco telemetry status     # Check current consent state
rtco telemetry enable     # Give consent (interactive prompt)
rtco telemetry disable    # Withdraw consent — stops all collection immediately
rtco telemetry forget     # Withdraw consent + delete all local data + request server-side erasure
```

**Override via environment:**
```bash
export RTK_TELEMETRY_DISABLED=1   # Blocks telemetry regardless of consent
```

## Star History

<a href="https://www.star-history.com/?repos=rtco-ai%2Frtk&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=rtco-ai/rtco&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=rtco-ai/rtco&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=rtco-ai/rtco&type=date&legend=top-left" />
 </picture>
</a>

## StarMapper

<a href="https://starmapper.bruniaux.com/rtco-ai/rtco">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://starmapper.bruniaux.com/api/map-image/rtco-ai/rtco?theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://starmapper.bruniaux.com/api/map-image/rtco-ai/rtco?theme=light" />
    <img alt="StarMapper" src="https://starmapper.bruniaux.com/api/map-image/rtco-ai/rtco" />
  </picture>
</a>

## Core team

- **Patrick Szymkowiak** — Founder
  [GitHub](https://github.com/pszymkowiak) · [LinkedIn](https://www.linkedin.com/in/patrick-szymkowiak/)
- **Florian Bruniaux** — Core contributor
  [GitHub](https://github.com/FlorianBruniaux) · [LinkedIn](https://www.linkedin.com/in/florian-bruniaux-43408b83/)
- **Adrien Eppling** — Core contributor
  [GitHub](https://github.com/aeppling) · [LinkedIn](https://www.linkedin.com/in/adrien-eppling/)

## Contributing

Contributions welcome! Please open an issue or PR on [GitHub](https://github.com/rtco-ai/rtco).

Join the community on [Discord](https://discord.gg/RySmvNF5kF).

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

## Disclaimer

See [DISCLAIMER.md](DISCLAIMER.md).
