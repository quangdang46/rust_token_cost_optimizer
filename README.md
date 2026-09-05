# rtco — Rust Token Cost Optimizer

> [!CAUTION]
> ## ⛔ THIS PROJECT IS ARCHIVED
> **Do not use in production.** Aggressive token compression cuts too much context — agents end up needing extra follow-up queries to compensate, negating the savings. Modern LLMs and agent harnesses handle verbose output better on their own. This repo is **no longer maintained** and may be revisited only if a fundamentally better approach emerges.

<div align="center">
  <img src="rtco_illustration.webp" alt="rtco — cut LLM token costs 60–90% on every CLI command">
</div>

<div align="center">

![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue.svg)
![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)
![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)
![Release](https://img.shields.io/github/v/release/quangdang46/rust_token_cost_optimizer)

</div>

**Cut LLM token costs by 60–90% on every CLI command.**  
A single Rust binary sits between your coding agent and the shell, filtering and compressing noisy command output in real time — more budget for the work that matters.

<div align="center">

```bash
curl -fsSL "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/main/install.sh?$(date +%s)" \
  | bash -s -- --with-hooks
```

</div>

---

## 🤖 Agent Quickstart

Always prefix commands with `rtco` — even unsupported ones passthrough safely:

```bash
rtco git log -20        # 80% savings
rtco cargo test         # 90% savings (failures only)
rtco cargo clippy --all-targets
rtco pnpm install       # 90% savings
rtco gain               # savings dashboard
rtco gain --history     # per-command breakdown
rtco proxy <cmd>        # unfiltered passthrough (still tracked)
```

**RTCO is always safe to use.** Unknown subcommands passthrough with 0% savings but zero risk.

```bash
# ❌ Wrong
git add . && git commit -m "msg" && git push

# ✅ Correct
rtco git add . && rtco git commit -m "msg" && rtco git push
```

**Output conventions**
- stdout = filtered/compressed output
- stderr = diagnostics, raw on error
- exit code = child process exit code (always propagates)

**Agent wiring:** `rtco init` or `--with-hooks` during install auto-configures Claude Code, Cursor, Copilot, Windsurf, Gemini CLI, and more.

---

## TL;DR

### The Problem

Agents run hundreds of shell commands per session. Raw `git log`, `cargo test`, `pnpm install` dump machine noise into context:

| Command class | Typical waste |
|---------------|---------------|
| Tests | Pass spam · ANSI · full stacks |
| Builds | Dependency noise · progress bars |
| Git/GitHub | Metadata walls · ASCII chrome |
| Package managers | Trees · download chatter |

**60–90% of the token bill** is often noise.

### The Solution

**rtco** proxies commands, compresses stdout, tracks savings in SQLite:

```bash
rtco git log -20
rtco cargo test
rtco docker ps
rtco gain          # tokens + $ saved
```

```text
# Before: ~2,847 tokens for git log -20
# After:  ~498 tokens  (≈82% savings)
```

### Why Use rtco?

| Feature | What it does |
|---------|--------------|
| **60–90% savings** | Purpose-built filters per ecosystem |
| **100+ commands** | git · cargo · pnpm · docker · kubectl · gh · pytest · … |
| **&lt;10 ms startup** | Blocking by design — no async runtime tax |
| **CLI proxy** | `rtco <cmd>` for agents; unknown cmds passthrough safely |
| **Analytics** | `rtco gain` tokens + cost history |
| **Auto-wiring** | Hooks for Claude Code · Copilot · Cursor · Windsurf · Gemini CLI · … |
| **Privacy** | Telemetry off by default |

> **Name collision:** another crate also uses “rtco” (Rust Type Kit). Verify:
> `rtco gain` must work. If not, you installed the wrong package.

---

### Quick Example

```bash
# Install binary + provider hooks
curl -fsSL "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/main/install.sh?$(date +%s)" \
  | bash -s -- --with-hooks

# Always prefix (including chains)
rtco git status
rtco git log -20
rtco cargo test
rtco cargo clippy --all-targets
rtco pnpm install
rtco gain
rtco gain --history
```

```bash
# ❌ raw
git add . && git commit -m "msg" && git push

# ✅
rtco git add . && rtco git commit -m "msg" && rtco git push
```

---

## Design Philosophy

1. **Filter at the shell boundary.**  
   Compressing model context after the fact is late. Kill noise when the command exits.

2. **Purpose-built beats generic summarize.**  
   `cargo test` failures-only is more reliable than asking an LLM to “summarize this log.”

3. **Passthrough is a feature.**  
   Unknown commands still run under `rtco` so agents can prefix everything safely.

4. **Startup is a product metric.**  
   No async runtime. Target **&lt;10 ms** startup and **&lt;5 MB** RAM.

5. **Exit codes always propagate.**  
   CI must see the real child status — filtering must never mask failure.

---

## How rtco Compares

| Approach | Savings | Startup | Agent wiring | Structure-aware |
|----------|---------|---------|--------------|-----------------|
| Manual `/compact` | Coarse | N/A | Manual | No |
| Generic summarizer | Lossy | Slow | Custom | Weak |
| Alias soup | Partial | Fast | Fragile | Ad-hoc |
| **rtco** | 60–90% per cmd | **&lt;10 ms** | Hooks for major agents | Per-ecosystem filters |

**When to use rtco:**
- Coding agents that shell out constantly (Claude Code, Cursor, Codex, …)
- Heavy git / cargo / js test loops
- Teams that want measurable token savings (`rtco gain`)

**When rtco might not be ideal:**
- You need full raw logs every time (use `rtco proxy` or no prefix)
- Commands outside the 100+ filter set (passthrough still works, 0% savings)

---

## Installation

### Linux / macOS

```bash
# Binary only
curl -fsSL "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/main/install.sh?$(date +%s)" | bash

# Binary + auto-config hooks for detected AI providers
curl -fsSL "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/main/install.sh?$(date +%s)" \
  | bash -s -- --with-hooks
```

### Windows PowerShell

```powershell
irm "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/main/install.ps1" | iex
# Then for hooks:
.\install.ps1 -WithHooks
```

### Cargo

```bash
cargo install --git https://github.com/quangdang46/rust_token_cost_optimizer
```

### Verify (catch name collision)

```bash
rtco --version   # e.g. rtco 0.2.3+ / 0.28.x line depending on release
rtco gain        # savings dashboard MUST exist
```

If `rtco gain` fails with “command not found”, you have the **wrong** `rtco` package.

More detail: [`INSTALL.md`](INSTALL.md).

---

## Quick Start

```bash
rtco git status
rtco git log -20
rtco cargo test
rtco cargo clippy --all-targets
rtco pnpm install
rtco gain
rtco gain --history
```

### Proxy (no filter, still tracked)

```bash
rtco proxy git log --oneline -20
rtco proxy curl https://api.example.com/data
```

### Agent wiring

```bash
rtco init           # project CLAUDE.md instructions
rtco init --global  # ~/.claude/CLAUDE.md
rtco discover       # find missed RTCO opportunities in sessions
```

---

## Features

| Feature | Detail |
|---------|--------|
| **60–90% savings** | Purpose-built filters per ecosystem |
| **100+ commands** | git · cargo · pnpm · docker · kubectl · gh · pytest · … |
| **Multi-algorithm** | SmartCrusher, CCR, structural anchors |
| **CLI proxy** | `rtco <cmd>` for agents |
| **Analytics** | `rtco gain` tokens + cost |
| **Fast** | &lt;10 ms startup · &lt;5 MB RAM |
| **Auto-wiring** | Claude Code · Copilot · Cursor · Windsurf · Gemini CLI · … |
| **Privacy** | Telemetry off by default |

### Supported ecosystems (sample)

| Ecosystem | Commands |
|-----------|----------|
| Git | `git` `gh` `gt` `diff` |
| Rust | `cargo` |
| JS/TS | `npm` `pnpm` `vitest` `jest` `playwright` `tsc` `next` `prettier` `prisma` |
| Python | `ruff` `pytest` `mypy` `pip` |
| Ops | `docker` `kubectl` `aws` `curl` `wget` `psql` |
| Go / .NET / Ruby | `go` `dotnet` `rspec` … |
| System | `ls` `tree` `read` `grep` `find` `json` `log` `summary` |

### Typical savings by category

| Category | Commands | Typical savings |
|----------|----------|-----------------|
| Tests | vitest, playwright, cargo test | 90–99% |
| Build | next, tsc, lint, prettier | 70–87% |
| Git | status, log, diff, add, commit | 59–80% |
| GitHub | gh pr, gh run, gh issue | 26–87% |
| Package managers | pnpm, npm, npx | 70–90% |
| Files | ls, read, grep, find | 60–75% |
| Infrastructure | docker, kubectl | ~85% |

---

## Commands

### Meta

```bash
rtco gain                # token savings statistics
rtco gain --history      # per-command history
rtco discover            # analyze Claude Code sessions for missed RTCO usage
rtco proxy <cmd>         # run without filtering (still logged)
rtco init                # add RTCO instructions to CLAUDE.md
rtco init --global       # add to ~/.claude/CLAUDE.md
```

### Everyday (always safe to prefix)

```bash
# Git
rtco git status
rtco git log -20
rtco git diff
rtco git show HEAD

# Rust
rtco cargo test
rtco cargo clippy --all-targets
rtco cargo build

# JS/TS
rtco pnpm install
rtco vitest
rtco tsc --noEmit
rtco next build

# Ops
rtco docker ps
rtco kubectl get pods
rtco gh pr view 123
```

Unknown subcommands **passthrough** safely — still use `rtco`.

---

## How It Works

```text
agent / human
    │
    ▼
 rtco <cmd>     →  execute real command
    │              strip ANSI · dedupe · truncate · structure
    ▼
 filtered stdout →  LLM context
    │
    ▼
 SQLite tracking →  rtco gain
```

| Layer | Role |
|-------|------|
| `rtco-cli` | Clap routing + binary |
| `rtco-core` | Shared filter / tracking infrastructure |
| per-ecosystem filters | git · cargo · js · python · cloud · … |
| hooks | provider auto-wiring |
| TOML filter DSL | declarative filter configs |

---

## Configuration

| Location | Purpose |
|----------|---------|
| `~/.config/rtco/config.toml` | Global config |
| `RTCO_DB_PATH` | Override SQLite path for tracking |
| `.rtco/filters/*.toml` | Project-local filter overrides |

```bash
rtco gain
# see docs/usage/TRACKING.md for DB layout and metrics
```

Telemetry is **off by default** — see [`docs/TELEMETRY.md`](docs/TELEMETRY.md).

---

## Troubleshooting

### Wrong package installed (`rtco gain` missing)

```bash
# Uninstall the other "rtco" / clear PATH confusion, then:
curl -fsSL "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/main/install.sh?$(date +%s)" | bash
rtco gain
```

### Agent still runs raw commands

```bash
rtco init --global
# or reinstall hooks:
curl -fsSL "https://raw.githubusercontent.com/quangdang46/rust_token_cost_optimizer/main/install.sh?$(date +%s)" \
  | bash -s -- --with-hooks
rtco discover
```

### Need full unfiltered output once

```bash
rtco proxy cargo test -- --nocapture
```

### Filter looks wrong / too aggressive

Filters fall back to raw output on internal failure. If a filter is over-aggressive, use `proxy` for that command and file a bug with a fixture.

### CI thinks the command succeeded when it failed

That would be a bug — exit codes must propagate. Confirm with:

```bash
rtco false; echo $?   # expect non-zero
```

---

## Limitations

### What rtco Doesn't Do (Yet)

- **Not a model** — compresses shell output only; does not summarize chat turns
- **Not every CLI on earth** — 100+ commands; others passthrough at 0% savings
- **Name collision** — another “rtco” exists; always verify with `rtco gain`

### Known Limitations

| Capability | Current state | Notes |
|------------|---------------|-------|
| Filter coverage | ✅ 100+ cmds | Growing per ecosystem |
| Async runtime | ❌ Forbidden | Startup budget |
| License | Apache-2.0 | Not MIT |
| Telemetry | Off by default | Opt-in only |

---

## FAQ

### Will it break my CI?

Exit codes always propagate from the child process.

### Can I disable filtering once?

`rtco proxy <cmd>` — full output, still logged.

### Where is the DB?

Config / `RTCO_DB_PATH` — see `rtco gain` and [`docs/usage/TRACKING.md`](docs/usage/TRACKING.md).

### Why no async?

Blocking by design for **&lt;10 ms** startup. Async runtimes add measurable overhead here.

### How do I measure savings?

```bash
rtco gain
rtco gain --history
```

### Does it work with Claude Code?

Yes — `rtco init` / `--with-hooks` wires instructions and hooks. Prefer `rtco <cmd>` in agent sessions.

---

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

---

## License

[Apache-2.0](LICENSE)

---

<div align="center">

**Less noise. More tokens for the actual work.**

</div>
