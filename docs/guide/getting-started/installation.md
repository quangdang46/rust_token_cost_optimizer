---
title: Installation
description: Install RTCO via curl, Homebrew, Cargo, or from source, and verify the correct version
sidebar:
  order: 1
---

# Installation

## Name collision warning

Two unrelated projects share the name `rtco`. Make sure you install the right one:

- **Rust Token Killer** (`rtco-ai/rtco`) — this project, a token-saving CLI proxy
- **Rust Type Kit** (`reachingforthejack/rtco`) — a different tool for generating Rust types

The easiest way to verify you have the correct one: run `rtco gain`. It should display token savings stats. If it returns "command not found", you either have the wrong package or RTCO is not installed.

## Check before installing

```bash
rtco --version   # should print: rtco x.y.z
rtco gain        # should show token savings stats
```

If both commands work, RTCO is already installed. Skip to [Project initialization](#project-initialization).

## Quick install (Linux and macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/master/install.sh | sh
```

## Homebrew (macOS and Linux)

```bash
brew install rtco-ai/tap/rtco
```

## Cargo

:::caution[Name collision risk]
`cargo install rtco` may install **Rust Type Kit** instead of Rust Token Killer — two unrelated projects share the same crate name. Use the explicit Git URL to guarantee the correct package:
:::

```bash
cargo install --git https://github.com/rtco-ai/rtco rtco
```

## Pre-built binaries (Windows, Linux, macOS)

Download from [GitHub releases](https://github.com/rtco-ai/rtco/releases):

- macOS: `rtco-x86_64-apple-darwin.tar.gz` / `rtco-aarch64-apple-darwin.tar.gz`
- Linux: `rtco-x86_64-unknown-linux-musl.tar.gz` / `rtco-aarch64-unknown-linux-gnu.tar.gz`
- Windows: `rtco-x86_64-pc-windows-msvc.zip`

**Windows users**: Extract the zip and place `rtco.exe` in a directory on your PATH. Run RTCO from Command Prompt, PowerShell, or Windows Terminal — do not double-click the `.exe` (it prints usage and exits immediately). For full hook support, use [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) instead.

## Verify installation

```bash
rtco --version   # rtco x.y.z
rtco gain        # token savings dashboard
```

If `rtco gain` fails but `rtco --version` succeeds, you installed Rust Type Kit by mistake. Uninstall it first:

```bash
cargo uninstall rtco
```

Then reinstall using one of the methods above.

## Project initialization

Run once per project to enable the Claude Code hook:

```bash
rtco init
```

For a global install that patches `settings.json` automatically:

```bash
rtco init --global
```

## Uninstall

```bash
rtco init -g --uninstall    # remove hook, RTCO.md, and settings.json entry
cargo uninstall rtco         # remove binary (if installed via Cargo)
brew uninstall rtco          # remove binary (if installed via Homebrew)
```
