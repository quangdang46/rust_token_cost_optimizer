<div align="center">

# RTCO — The LLM Token Killer

**Cut LLM token costs by 60-90% on every CLI command.**

[![CI](https://img.shields.io/github/actions/workflow/status/quangdang46/rust_token_cost_optimizer/ci.yml?style=for-the-badge&logo=githubactions&label=CI)](https://github.com/quangdang46/rust_token_cost_optimizer/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/actions/workflow/status/quangdang46/rust_token_cost_optimizer/release.yml?style=for-the-badge&logo=githubactions&label=Release)](https://github.com/quangdang46/rust_token_cost_optimizer/actions/workflows/release.yml)
[![Version](https://img.shields.io/github/v/release/quangdang46/rust_token_cost_optimizer?style=for-the-badge&logo=semver)](https://github.com/quangdang46/rust_token_cost_optimizer/releases)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue?style=for-the-badge&logo=apache)](LICENSE)
[![Stars](https://img.shields.io/github/stars/quangdang46/rust_token_cost_optimizer?style=for-the-badge&logo=github)](https://github.com/quangdang46/rust_token_cost_optimizer/stargazers)

</div>

---

AI coding assistants execute hundreds of shell commands per session. Every `git status`, `cargo test`, or `pnpm install` dumps verbose output into your LLM context window — 60-90% of every token budget wasted on machine noise.

RTCO is a single Rust binary that sits between your AI assistant and the shell, intercepting commands and compressing their output in real-time. Less noise, more tokens for what matters.

## Quick Install

```bash
# Binary only (Linux/macOS)
curl -fsSL https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.sh | bash

# Binary + auto-config MCP+hooks in every detected AI provider (Linux/macOS)
curl -fsSL https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.sh \
  | bash -s -- --with-mcp --with-hooks --all-providers
```

```powershell
# Binary only (Windows)
irm "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.ps1" | iex

# Binary + auto-config MCP+hooks in every detected AI provider (Windows)
irm "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.ps1" | iex
# Then re-run with flags:
.\install.ps1 -WithMcp -WithHooks -AllProviders
```

## How It Works

**1. Run commands through RTCO**

```bash
rtco git log -20
rtco cargo test
rtco docker ps
```

**2. RTCO filters and compresses output in real-time**

Noise is stripped. Duplicates are collapsed. Long output is truncated. What reaches your LLM is the signal, not the noise.

```
# Before: 2,847 tokens for git log -20
$ git log -20 | wc -w
2847

# After: 498 tokens (82% savings)
$ rtco git log -20 | wc -w
498
```

**3. Track your savings**

```bash
$ rtco gain
Total tokens saved: 1,234,567
Total cost saved:  $12.34
Average savings:   78.3%
```

## Key Features

| | Feature | Description |
|---|---|---|
| :zap: | **60-90% Token Savings** | Every command filtered through a purpose-built pipeline |
| :toolbox: | **30+ Supported Tools** | git, cargo, pnpm, docker, kubectl, gh, pytest, and more |
| :shredder: | **Multi-Algorithm Compression** | SmartCrusher, CCR, structural anchors — pick your weapon |
| :robot: | **MCP Server** | `rtco-mcp` for direct agent integration |
| :bar_chart: | **Tracking & Analytics** | `rtco gain` shows real savings, history, and cost reductions |
| :rocket: | **Blazing Fast** | <10ms startup, <5MB memory, zero runtime overhead |
| :link: | **Auto-Wiring** | Works with Claude Code, Copilot, Cursor, Windsurf, Gemini CLI |
| :lock: | **Privacy First** | Telemetry off by default. No data leaves your machine without consent. |

## Supported Tools

| Ecosystem | Commands |
|---|---|
| :octocat: **Git** | `git` `gh` `gt` `diff` |
| :crab: **Rust** | `cargo` |
| :green_book: **JavaScript** | `npm` `pnpm` `vitest` `jest` `playwright` `tsc` `next` `prettier` `prisma` |
| :snake: **Python** | `ruff` `pytest` `mypy` `pip` |
| :whale: **Cloud & Ops** | `docker` `kubectl` `aws` `curl` `wget` `psql` |
| :gear: **Go** | `go` `golangci-lint` |
| :bridge_at_night: **.NET** | `dotnet` `binlog` `trx` |
| :gem: **Ruby** | `rspec` `rubocop` `rake` |
| :file_cabinet: **System** | `ls` `tree` `read` `grep` `find` `wc` `env` `json` `log` `deps` `summary` |

100+ commands supported. Run `rtco --help` for the full list.

---

## Installation Options

### Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.sh | bash
```

### Quick Install (Windows PowerShell)

```powershell
irm "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.ps1" | iex
```

### Install + Auto-Configure MCP+Hooks (All Platforms)

```bash
# Linux/macOS — detect and register MCP+hooks in every AI provider found on disk
curl -fsSL https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.sh \
  | bash -s -- --with-mcp --with-hooks --all-providers
```

```powershell
# Windows — download first, then run with flags
irm "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/master/install.ps1" -OutFile install.ps1
.\install.ps1 -WithMcp -WithHooks -AllProviders
```

### Cargo

```bash
cargo install --git https://github.com/quangdang46/rust_token_cost_optimizer
```

### Pre-built Binaries

Download from [releases](https://github.com/quangdang46/rust_token_cost_optimizer/releases):

- **macOS**: `rtco-x86_64-apple-darwin.tar.gz`, `rtco-aarch64-apple-darwin.tar.gz`
- **Linux**: `rtco-x86_64-unknown-linux-musl.tar.gz`, `rtco-aarch64-unknown-linux-gnu.tar.gz`
- **Windows**: `rtco-x86_64-pc-windows-msvc.zip`

### Verify

```bash
rtco --version   # Should show rtco 0.41.0
rtco gain        # Should show token savings stats
```

## Setup: AI Tool Auto-Wiring

Install the transparent hook and your AI tool will automatically route commands through RTCO:

```bash
rtco init -g                     # Claude Code / Copilot (default)
rtco init -g --gemini            # Gemini CLI
rtco init -g --agent cursor      # Cursor
rtco init -g --agent windsurf    # Windsurf
rtco init -g --opencode          # OpenCode
rtco init --agent cline          # Cline / Roo Code
rtco init -g --codex             # Codex
rtco init --agent hermes         # Hermes
rtco init --agent kilocode       # Kilo Code
```

Once configured, every shell command your AI runs is automatically compressed:

```
# Without RTCO                                 # With RTCO
┌─────────┐     ┌──────┐     ┌───┐            ┌─────────┐     ┌──────┐     ┌──────────┐     ┌───┐
│ Claude  │────▶│ bash │────▶│git│            │ Claude  │────▶│ bash │────▶│  rtco    │────▶│git│
└─────────┘     └──────┘     └───┘            └─────────┘     └──────┘     └──────────┘     └───┘
      ▲                       │                     ▲                       │          │
      │    ~2,000 tokens      │                     │    ~200 tokens        │  filter  │
      │    (full raw git      │                     │    (collapsed         │  +       │
      │     status output)    │                     │     git status)       │  compress│
      └───────────────────────┘                     └───────────────────────┴──────────┘
```

## Tracking Your Savings

```bash
$ rtco gain
╭──────────────────────────────────╮
│      Token Savings Summary       │
├──────────────────────────────────┤
│ Total saved:      1,234,567      │
│ Cost saved:       $12.34         │
│ Commands tracked: 892            │
│ Avg savings:      78.3%          │
╰──────────────────────────────────╯

$ rtco gain --history              # Per-command breakdown
$ rtco gain --days 7               # Last 7 days only
```

## Configuration

RTCO is configured via `~/.config/rtco/config.toml`:

```toml
[hooks]
exclude_commands = ["curl", "playwright"]   # skip rewrite for these

[tee]
enabled = true    # save raw output on failure (default: true)
mode = "failures" # "failures", "always", or "never"
```

When a command fails, RTCO saves the full unfiltered output so the LLM can inspect it without re-executing.

### TOML Filter DSL

RTCO supports a TOML-based filter definition language for customizing how specific commands are processed. Place `*.toml` files in:

- **Global**: `~/.config/rtco/filters/`
- **Project-local**: `.rtco/filters/` (checked into your repo)

See the [TOML Filter DSL docs](docs/filters/) for the full syntax reference.

## Token Savings at a Glance

| Operation | Standard | RTCO | Savings |
|---|---|---|---|
| `git status` | ~2,000 tokens | ~200 tokens | -80% |
| `cargo test` | ~25,000 tokens | ~2,500 tokens | -90% |
| `pytest` | ~8,000 tokens | ~800 tokens | -90% |
| `git push` | ~1,600 tokens | ~120 tokens | -92% |

Estimates based on medium-sized projects. Your mileage may vary.

## Proxy Mode

Need to run a command without filtering? Use proxy mode — RTCO still tracks the call for metrics, but passes output through unchanged:

```bash
rtco proxy git log --oneline -20    # Full unfiltered output
rtco proxy npm install express      # Raw install output
rtco proxy curl https://api.example.com/data  # Any command works
```

Proxy commands appear in `rtco gain --history` with 0% savings.

## Uninstall

```bash
rtco init -g --uninstall   # Remove hook and configuration
cargo uninstall rtco        # Remove binary
```

## Privacy

Telemetry is **disabled by default** and requires explicit opt-in. No data is ever sent without your consent.

```bash
rtco telemetry status   # Check current setting
rtco telemetry enable   # Opt in
rtco telemetry disable  # Withdraw consent
```

## Contribute

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow, design philosophy, and development setup.

- [Architecture Overview](docs/contributing/ARCHITECTURE.md)
- [Technical Reference](docs/contributing/TECHNICAL.md)
- [Filter Implementation Checklist](src/cmds/README.md#adding-a-new-command-filter)

## License

Apache License 2.0 — see [LICENSE](LICENSE) for details.

---

<div align="center">

Made with :zap: by the RTCO contributors.

[![Star History Chart](https://api.star-history.com/svg?repos=quangdang46/rust_token_cost_optimizer&type=Date)](https://star-history.com/#quangdang46/rust_token_cost_optimizer&Date)

</div>
