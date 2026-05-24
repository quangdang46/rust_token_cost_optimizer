# Codex CLI Hooks

> Part of [`hooks/`](../README.md) — see also [`src/hooks/`](../../src/hooks/README.md) for installation code

## Specifics

- Prompt-level guidance via awareness document -- no programmatic hook
- `rtco-awareness.md` is injected into `AGENTS.md` with an `@RTCO.md` reference
- Installed to `$CODEX_HOME` when set, otherwise `~/.codex/`, by `rtco init --codex`
