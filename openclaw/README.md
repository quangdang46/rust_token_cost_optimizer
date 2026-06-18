# RTCO Plugin for OpenClaw

Transparently rewrites shell commands executed via OpenClaw's `exec` tool to their RTCO equivalents, achieving 60-90% LLM token savings.

This is the OpenClaw equivalent of the Claude Code hooks in `hooks/rtco-rewrite.sh`.

## How it works

The plugin registers a `before_tool_call` hook that intercepts `exec` tool calls. When the agent runs a command like `git status`, the plugin delegates to `rtco rewrite` which returns the optimized command (e.g. `rtco git status`). The compressed output enters the agent's context window, saving tokens.

All rewrite logic lives in RTCO itself (`rtco rewrite`). This plugin is a thin delegate -- when new filters are added to RTCO, the plugin picks them up automatically with zero changes.

## Installation

### Prerequisites

RTCO must be installed and available in `$PATH`:

```bash
brew install rtco
# or
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/master/install.sh | sh
```

### Install the plugin

```bash
# Copy the plugin to OpenClaw's extensions directory
mkdir -p ~/.openclaw/extensions/rtco-rewrite
cp openclaw/index.ts openclaw/openclaw.plugin.json ~/.openclaw/extensions/rtco-rewrite/

# Restart the gateway
openclaw gateway restart
```

### Or install via OpenClaw CLI

```bash
openclaw plugins install ./openclaw
```

## Configuration

In `openclaw.json`:

```json5
{
  plugins: {
    entries: {
      "rtco-rewrite": {
        enabled: true,
        config: {
          enabled: true,    // Toggle rewriting on/off
          verbose: false     // Log rewrites to console
        }
      }
    }
  }
}
```

## What gets rewritten

Everything that `rtco rewrite` supports (30+ commands). See the [full command list](https://github.com/rtco-ai/rtco#commands).

## What's NOT rewritten

Handled by `rtco rewrite` guards:
- Commands already using `rtco`
- Piped commands (`|`, `&&`, `;`)
- Heredocs (`<<`)
- Commands without an RTCO filter

## Measured savings

| Command | Token savings |
|---------|--------------|
| `git log --stat` | 87% |
| `ls -la` | 78% |
| `git status` | 66% |
| `grep` (single file) | 52% |
| `find -name` | 48% |

## License

Apache 2.0 -- same as RTCO.
