# Features — Rust Token Cost Optimizer

> **rust_token_cost_optimizer** (`rtco`) — a single Rust binary that reduces LLM token consumption for AI coding agents by filtering, compressing, diffing, caching, and routing the outputs of shell commands.
>
> This document lists what works today (carried over from the upstream `rtk-ai/rtk` codebase this repo derives from) and what is planned for the post-fork roadmap. It supersedes `docs/usage/FEATURES.md`, which is the legacy French-language reference.

---

## Table of contents

1. [Identity & positioning](#identity--positioning)
2. [Existing features (inherited)](#existing-features-inherited)
3. [Planned features (roadmap)](#planned-features-roadmap)
4. [Out of scope](#out-of-scope)

---

## Identity & positioning

`rtco` is a fork-with-direction. The upstream binary (`rtk`) is a shell-output filter: regex/heuristic-driven, per-command, hook-based. `rtco` keeps that as the foundation and adds three layers the upstream does not pursue:

| Layer | Upstream `rtk` | `rtco` adds |
|---|---|---|
| **Static filtering** | Yes (60+ TOML filters, 50+ Rust filters) | Maintained, expanded |
| **Accurate measurement** | Estimate via word/char proxy | Real tokenizer counts, per-model |
| **Stateful compression** | None (each command stand-alone) | Diff-aware delta proxy + LRU cache |
| **Agent integration** | Bash hook rewriter | Native MCP server + plugin SDK |

The upstream README pitches "60–90% token savings". `rtco` aims to make that **measured, not estimated**, and to deliver it **across every agent transport** (bash hook, MCP, plugin), not just bash.

---

## Existing features (inherited)

These features ship today, inherited from the upstream codebase. The CLI surface comes from `src/main.rs::Commands` and matches the legacy `docs/usage/FEATURES.md` (in French). All commands fall back to raw passthrough if a filter is unavailable, so the tool is always safe to prefix.

### Global flags

| Flag | Short | Behaviour |
|---|---|---|
| `--verbose` | `-v` | Increase log verbosity (`-v`, `-vv`, `-vvv`) |
| `--ultra-compact` | `-u` | ASCII icons, inline format, smallest output |
| `--skip-env` | | Sets `SKIP_ENV_VALIDATION=1` for child processes (Next.js, tsc, prisma) |

### File commands

| Command | Replaces | Typical savings |
|---|---|---|
| `rtco ls` | `ls`, `tree` | ~80% |
| `rtco tree` | `tree` | ~80% |
| `rtco read <file>` | `cat`/`head`/`tail` (with `--level none/minimal/aggressive`) | 30%–74% |
| `rtco smart <file>` | 2-line heuristic source-file summary | ~95% |
| `rtco find` | `find`, `fd` | ~80% |
| `rtco grep <pat>` | `grep`, `rg` (passes flags through to `rg`) | ~80% |
| `rtco diff <a> <b>` | `diff` | ~60% |
| `rtco wc` | `wc` | varies |
| `rtco json <file>` | structural JSON view (no values) | ~60% |
| `rtco env` | `env` (sensitive vars masked) | varies |
| `rtco log` | dedupe repeated log lines | 60–80% |

`rtco read --level aggressive` strips function bodies for Rust/Python/JS/TS/Go/C/C++/Java/Ruby/Shell, leaving only signatures.

### Git, GitHub, GitLab, Graphite

| Command | Notes | Savings |
|---|---|---|
| `rtco git status` | compact one-line summary | ~80% |
| `rtco git log` | hash + subject only | ~80% |
| `rtco git diff` | grouped + condensed | ~75% |
| `rtco git show` | summary + stat + diff | ~80% |
| `rtco git add/commit/push/pull/fetch/stash/branch/worktree` | reduced to `ok …` form | ~92% |
| `rtco gh pr/issue/run/api` | gh CLI compact | 26–87% |
| `rtco glab …` | GitLab CLI passthrough + filtered list/view | varies |
| `rtco gt log/submit/sync/restack/create/branch` | Graphite (stacked PRs) | varies |

Any unrecognised git/gh/glab/gt subcommand is passed through unchanged.

### Test runners

| Command | Notes | Savings |
|---|---|---|
| `rtco test <cmd>` | generic — failures only | ~90% |
| `rtco err <cmd>` | errors/warnings only | ~80% |
| `rtco cargo test`, `rtco cargo nextest` | Rust | ~90% |
| `rtco jest`, `rtco vitest` | JS | up to 99% |
| `rtco playwright test` | E2E | ~94% |
| `rtco pytest` | Python | ~90% |
| `rtco go test` | Go (NDJSON streaming) | ~90% |
| `rtco rake test`, `rtco rspec`, `rtco rubocop` | Ruby | ~60–90% |
| `rtco gradlew test` | JVM | varies |
| `rtco dotnet test/build/format` | .NET (binlog + trx parsers) | varies |

### Build, lint, format, type-check

| Command | Filter | Savings |
|---|---|---|
| `rtco cargo build/check/clippy/install` | strip `Compiling…` lines, group warnings by lint | ~80% |
| `rtco tsc` | TypeScript errors grouped by file | ~83% |
| `rtco lint` | ESLint / Biome / oxlint, grouped by rule | ~84% |
| `rtco prettier`, `rtco format` | files needing formatting only | ~70% |
| `rtco next build` | route metrics summary | ~87% |
| `rtco ruff check/format`, `rtco mypy` | Python | ~80% |
| `rtco golangci-lint run` | JSON-grouped | ~85% |

### Package managers

`rtco pnpm`, `rtco npm`, `rtco npx` (smart-routes to tsc/lint/prisma filter), `rtco pip` (auto-detects `uv`), `rtco prisma generate/migrate/db-push`, `rtco deps <path>` (auto-detects Cargo/package.json/pyproject/go.mod/Gemfile).

### Containers, cloud, data

`rtco docker ps/images/logs/compose ps/compose logs/compose build` — split-path filters for `ps -a` vs `ps`, capped output.
`rtco kubectl pods/services/logs` (with `-n`/`-A`).
`rtco aws <service>` — forces JSON output, compresses across all AWS services.
`rtco psql`, `rtco curl`, `rtco wget`, `rtco summary <cmd>`, `rtco proxy <cmd>` (pure passthrough with usage tracking).

### TOML filter pack

60 declarative filters under `src/filters/*.toml` for tools that don't need a Rust module:

> ansible-playbook, basedpyright, biome, brew-install, bundle-install, composer-install, df, dotnet-build, du, fail2ban-client, gcc, gcloud, gradle, hadolint, helm, iptables, jira, jj, jq, just, liquibase, make, markdownlint, mise, mix-compile, mix-format, mvn-build, nx, ollama, oxlint, ping, pio-run, poetry-install, pre-commit, ps, quarto-render, rsync, shellcheck, shopify-theme, skopeo, sops, spring-boot, ssh, stat, swift-build, systemctl-status, task, terraform-plan, tofu-fmt/init/plan/validate, trunk-build, turbo, ty, uv-sync, xcodebuild, yadm, yamllint.

User-level overrides live in `~/.rtk/filters.toml` (see [Configuration](#configuration)).

### Hook system

Lightweight shell/plugin hooks rewrite raw commands to `rtco …` before the agent sees them.

| Agent | Mechanism |
|---|---|
| Claude Code, Copilot | `~/.claude/hooks/rtk-rewrite.sh` (PreToolUse) |
| Codex (OpenAI) | shell hook |
| Cursor, Windsurf, Cline, Roo Code, Kilo Code, Antigravity | rules file |
| Hermes | Python plugin (`hooks/hermes/rtk-rewrite/`) |
| OpenCode | TypeScript plugin (`hooks/opencode/rtk.ts`) |
| OpenClaw | TypeScript plugin (`openclaw/index.ts`) |

All rewriting logic lives in Rust (`src/discover/registry.rs`) — every shell hook is a thin delegate calling `rtco rewrite "<command>"`.

| Hook command | Purpose |
|---|---|
| `rtco rewrite <cmd>` | print the rewritten command (exit 1 if no rewrite) |
| `rtco init -g [--agent <name>]` | install hook + RTK.md for the chosen agent |
| `rtco init -g --auto-patch` | non-interactive (CI) |
| `rtco init -g --hook-only` / `--show` / `--uninstall` | management |
| `rtco verify` | SHA-256 integrity check of installed hook |
| `rtco hook-audit --since N` | metrics on hook usage (requires `RTK_HOOK_AUDIT=1`) |
| `rtco trust` | trust-store management |

### Analytics & tracking

SQLite tracking DB at `~/.local/share/rtk/tracking.db` (Linux) / `~/Library/Application Support/rtk/tracking.db` (macOS), 90-day rolling retention.

| Command | Purpose |
|---|---|
| `rtco gain` | total savings + top commands |
| `rtco gain --graph` / `--history` / `--daily/--weekly/--monthly/--all` | breakdowns |
| `rtco gain --quota -t pro\|5x\|20x` | savings projected onto an Anthropic plan quota |
| `rtco gain --failures` | commands where the filter fell back to raw |
| `rtco gain --format json\|csv` | dashboard export |
| `rtco discover` | scan Claude Code history for commands that **could** have been filtered |
| `rtco learn --write-rules` | extract recurring CLI corrections from agent history |
| `rtco cc-economics` | compare Claude Code spend (via `ccusage`) against `rtco` savings |
| `rtco session` | session-level inspection |

### Tee (output recovery)

When a filtered command exits non-zero, the raw output is stashed in `~/.local/share/rtk/tee/` and the path is appended to the filtered output. Configurable via `[tee]` in `config.toml` (`mode = "failures"|"always"|"never"`, `max_files = 20`, 500 B min, 1 MB cap).

### Telemetry (opt-in)

Off by default. With consent, sends one anonymous ping per 23 hours: device hash (SHA-256 of random salt), version, OS, arch, command count, top commands, savings %. Managed via `rtco telemetry status|enable|disable|forget`. Disable via `RTK_TELEMETRY_DISABLED=1`.

### Configuration

`~/.config/rtk/config.toml` (Linux) / `~/Library/Application Support/rtk/config.toml` (macOS):

```toml
[tracking]   enabled = true; history_days = 90
[display]    colors = true; emoji = true; max_width = 120
[filters]    ignore_dirs = [".git", "node_modules", "target", …]
[tee]        enabled = true; mode = "failures"; max_files = 20
[telemetry]  enabled = false
[hooks]      exclude_commands = []
```

Inspect with `rtco config`, scaffold with `rtco config --create`.

---

## Planned features (roadmap)

These are the deltas from upstream. Each item lists the **why**, the **building blocks researched**, and the **status**. None are implemented yet in this repo.

### 1. MCP server (`rtco mcp serve`)

**Why.** The upstream model is bash hooks: every agent needs its own rewrite shim, Windows/PowerShell support is fragile, and Claude Code's built-in tools (`Read`, `Grep`, `Glob`) bypass the bash hook entirely. MCP is the cross-agent contract that fixes all three.

**Building blocks.**
- [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) — official Rust SDK for MCP, latest `1.7.0` (May 2026), tokio-based, supports stdio + `TokioChildProcess` transports.
- MCP `tools/list` + `tools/call` ([spec 2025-03-26](https://modelcontextprotocol.io/specification/2025-03-26/server/tools)) is the only capability we need. Each existing filter becomes a tool with an explicit JSON Schema.

**Initial tool surface (~12 tools, ~80% of in-session calls):**
`rtco_git_status`, `rtco_git_diff`, `rtco_git_log`, `rtco_grep`, `rtco_find`, `rtco_ls`, `rtco_read`, `rtco_cargo_test`, `rtco_cargo_build`, `rtco_cargo_clippy`, `rtco_test` (generic), `rtco_err`.

**Config snippet (target):**
```json
{ "mcpServers": { "rtco": { "command": "rtco", "args": ["mcp", "serve"] } } }
```

**Status.** Roadmap, not started.

### 2. Accurate, model-aware token counting

**Why.** Today the savings table is calibrated against an estimator. Different model families tokenize text very differently — code-heavy outputs can vary ±25% depending on which encoder you trust. To make `rtco gain --quota` land within a few percent of the agent's actual bill, the count has to use each model's real tokenizer.

**Building blocks.**
- [`tiktoken-rs`](https://github.com/zurawiki/tiktoken-rs) `0.11.0` (Apr 2026) covers OpenAI: `o200k_harmony` (gpt-oss), `o200k_base` (GPT-5/o1/o3/o4/4o/4.5/4.1/codex), `cl100k_base` (gpt-4/3.5, embedding-3), `p50k_base`, `p50k_edit`, `r50k_base`/`gpt2`.
- [HuggingFace `tokenizers`](https://github.com/huggingface/tokenizers) for Llama, Mistral, Gemma, etc., loaded from `tokenizer.json` files — same crate the upstream `tiktoken-rs` README recommends for non-OpenAI models.
- Anthropic — there is **no public Claude tokenizer**. Counting goes through the [`count_tokens` API endpoint](https://platform.claude.com/docs/en/docs/build-with-claude/token-counting), which is free but rate-limited (100/2k/4k/8k RPM by tier). Plan: lazy batched counts behind a tier-aware queue, with the existing word-proxy as offline fallback.
- Gemini — same pattern (Google's `countTokens` API; no offline tokenizer).

**Surface (target):**
```bash
rtco gain --tokenizer cl100k_base
rtco gain --model claude-opus-4-7      # routes to Anthropic API
rtco gain --model gpt-4o               # offline (o200k_base)
```

**Status.** Roadmap. Tokenizer abstraction layer in `src/core/` to be added.

### 3. Diff-aware delta proxy

**Why.** In a typical 30-min coding session, an agent runs `cargo test` 4–6 times against output that drifts by only 30–60 lines per run. Today every run is filtered standalone, so the agent re-ingests almost-identical text repeatedly. Sending only the delta past run #1 is closer to free than to "filtered".

**Building blocks.**
- [`similar`](https://github.com/mitsuhiko/similar) crate — dependency-free, multiple algorithms (Myers, Patience, Histogram, LCS), line/word/char/grapheme granularity, unified-diff generation. Same crate `insta` uses, so it's already in the ecosystem.
- Cache key: `sha256(cmd + cwd + git_sha + args)`; LRU + TTL in SQLite alongside `tracking.db`.
- Interaction with existing filter pipeline: filter first → key on filtered output → cheap diff per re-run.

**Surface (target):**
```bash
rtco --diff cargo test           # opt-in per call
rtco config diff_proxy=true      # global default
```

**Status.** Roadmap. Likely the first big delta from upstream — minimal architectural risk, big measurable win.

### 4. Streaming filter for long-running commands

**Why.** Current pipeline is buffer-then-filter. For `cargo build` on a cold target/, the agent waits 5 minutes seeing nothing. Streaming filters can flush filtered chunks as the underlying process emits NDJSON / line-buffered output (Go test, ruff, biome, Vitest already produce this).

**Approach.** Per-command filter trait gets an optional `streaming: true` capability and an incremental `feed_line(&mut self, &str)` method. `cargo build` / `tsc` / `next build` / `pytest` / `go test` are the highest-value targets.

**Status.** Roadmap. Touches `src/core/stream.rs` and `src/core/runner.rs`.

### 5. Project-local + WASM-extensible filters

**Why.** Internal/proprietary tools (a company's CI script, a custom log shipper, in-house CLIs like `ms`, `ffs`, `rfo`) will never get an upstream PR. Today users can drop a TOML into `~/.rtk/filters.toml`, but TOML is too limited for stateful filters. Per-project filters should ship in the repo so a team gets the savings without machine-level config drift, and adventurous users should be able to write filters in any language that compiles to WASM.

**Layers.**
- **`./.rtco/filters.toml`** — repo-local TOML, picked up automatically when CWD is inside the repo.
- **`./.rtco/filters/*.wasm`** — sandboxed plugin loaded via `wasmtime`, called per-line or per-block.
- Plugin manifest (`filter.json`) declares command match, capabilities (read-only, line-stream, block-stream).

**Status.** Roadmap. WASM is later than TOML-local, which is straightforward.

### 6. Native agent integrations (`hooks/`)

The upstream `hooks/` already covers Claude Code, Copilot, Codex, Cursor, Windsurf, Cline, Roo Code, Kilo Code, Antigravity, Hermes, OpenCode, OpenClaw. Planned additions:

| Agent | Status | Notes |
|---|---|---|
| **Kiro** | planned | Default agent on the maintainer's stack; no upstream hook exists. ~50 lines bash + README. |
| **jcode** | planned | Maintainer's own agent host. Tighter integration than a bash hook — calls `rtco` from inside the tool runner, feeds results directly into the model context, surfaces savings in jcode telemetry. |
| **Generic MCP** | planned | Falls out of feature 1 — any MCP-aware agent (Claude Desktop, Cursor, Cline, Codex CLI, GitHub Copilot agent SDK) works without per-agent code. |

### 7. Vietnamese localization

`README_vi.md` to match the existing `_fr/_zh/_ja/_ko/_es` set.

### 8. Quality & observability targets

These are operating constraints, not features, but they shape every roadmap item:

- **Startup:** keep `<10ms` (per `.claude/rules/cli-testing.md`); no async runtime in the hot path. The MCP server (#1) gets its own tokio runtime — startup of the long-lived server is irrelevant, per-tool latency target stays `<10ms`.
- **Memory:** keep `<5MB` resident.
- **Binary size:** keep `<5MB`. WASM (#5) and tokenizers (#2) are feature-gated to keep the default build slim.
- **Test infra:** `insta` snapshots + token-savings asserts (≥60% per filter) per the existing testing rules — every new filter ships with both.
- **Cross-platform:** macOS + Linux first-class, Windows via CI, WSL recommended for hook system on Windows.

---

## Out of scope

Explicitly **not** planned for `rtco`:

- A model-side proxy (intercepting the LLM API itself). Token routing at that layer belongs to a separate tool — `openproxy`, in this maintainer's stack — not to `rtco`. `rtco` only sees the agent's tool/shell output.
- Long-term cross-session memory. That's `mempalace`. `rtco` is single-session, command-scoped.
- A general LLM cost dashboard. `rtco gain` reports `rtco`'s own savings; full-fleet cost analytics belong to a higher-level observability tool.
- Filter logic that requires an LLM call. `rtco`'s value proposition is "deterministic, sub-10ms, offline". Calling out to a model to summarise an error log defeats the purpose.

---

## Compatibility note

`rtco` is derived from the Apache-2.0 licensed [`rtk-ai/rtk`](https://github.com/rtk-ai/rtk) project. Filter behaviour and CLI surface remain compatible at the `<binary> <subcommand>` level; the binary name changes from `rtk` to `rtco`. Existing `~/.rtk/` state files are read on first run and migrated to `~/.rtco/`. The legacy `docs/usage/FEATURES.md` (French) is preserved for reference but supplanted by this document going forward.
