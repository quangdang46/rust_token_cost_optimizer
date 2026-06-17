# CI/CD Flows

## CI

Triggers: push to `main` or pull_request targeting `main`

```
     ┌──────────────────┐
     │  push / PR to    │
     │  main            │
     └────────┬─────────┘
              │
     ┌────────▼─────────┐
     │  cargo fmt       │
     │  -- --check      │
     └────────┬─────────┘
              │
     ┌────────▼─────────────┐
     │  cargo clippy        │
     │  --all-targets       │
     │  -D warnings         │
     └────────┬─────────────┘
              │
     ┌────────▼─────────┐
     │  cargo test      │
     │  --all-features  │
     └────────┬─────────┘
          ┌───┴───┐
     ┌────▼────┐  │
     │ coverage│  │
     └─────────┘  │
          ┌───────▼────────┐
          │  cargo build   │
          │  (integration  │
          │   tests)       │
          └───────┬────────┘
          ┌───────▼────────┐
          │  cargo test    │
          │  -- --ignored  │
          └───────┬────────┘
          ┌───────▼────────┐
          │  cargo build   │
          │  --release     │
          └───────┬────────┘
          ┌───────▼────────────┐
          │  hyperfine bench   │
          │  (optional)        │
          └────────────────────┘
```

## Release

Trigger: tag push matching `v*`

Builds release binaries for Linux, macOS, and Windows, then creates a
GitHub Release with the artifacts and auto-generated release notes.

### How to release

```bash
# 1. Ensure main is up to date and all CI checks pass
git checkout main && git pull

# 2. Tag the release (semver, e.g. v0.41.0)
VERSION="v0.41.0"
git tag -a "$VERSION" -m "Release $VERSION"
git push origin "$VERSION"

# 3. CI automatically builds binaries and creates the GitHub Release
```

The release workflow uses `softprops/action-gh-release` to upload artifacts
and `cargo build --release` to produce stripped, LTO-optimized binaries.
Release notes are auto-generated from commits since the last tag.

### Prerequisites

- `GITHUB_TOKEN` with `contents:write` scope (default for GitHub Actions)
- A tag pushed to the remote matching `v*`
