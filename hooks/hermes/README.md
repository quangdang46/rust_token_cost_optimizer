# RTCO Plugin for Hermes

Rewrites Hermes `terminal` tool commands to RTCO equivalents before execution, so Hermes receives compact command output without changing your workflow.

## Installation

```bash
rtco init --agent hermes
```

The installer writes the plugin to `~/.hermes/plugins/rtco-rewrite/` and enables it through `plugins.enabled` in the Hermes config. The repository copy lives in `hooks/hermes/`; don't use that repo path as the runtime install path.

## Development

Run the Hermes plugin tests from the repository root:

```bash
python3 -m unittest discover -s hooks/hermes
```

## How it works

Hermes loads plugins from Python, so the plugin entrypoint is Python. The Python code is only a thin Hermes adapter. It reads the Hermes terminal tool payload, calls `rtco rewrite` for the actual command decision, then mutates the terminal tool `command` before Hermes executes it.

All rewrite rules stay in Rust inside `rtco rewrite`. When RTCO adds or changes command rewrite behavior, the Hermes plugin picks up that behavior by delegating to the RTCO binary.

## Fail-open behavior

The plugin does not block command execution. If anything goes wrong, Hermes runs the original command unchanged.

If rtco is not available in PATH when Hermes loads the plugin, the plugin prints a warning and skips hook registration.

- `rtco` is missing from `PATH`
- `rtco rewrite` exits with an error
- Hermes sends a non-terminal tool call
- The tool payload has no string `command`
- The plugin raises an unexpected exception

## Limitations

- Only Hermes `terminal` tool calls are rewritten.
- Commands skipped by `rtco rewrite` stay unchanged, including commands already prefixed with `rtco`, compound shell commands, heredocs, and commands without an RTCO filter.
- Shell hooks are not used for Hermes command rewriting. The integration depends on Hermes loading Python plugins and passing a mutable terminal tool payload.
