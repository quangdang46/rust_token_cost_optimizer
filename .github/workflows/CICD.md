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

### Manual release (when CI runners are unavailable)

If GitHub Actions runners are not allocating (e.g. free tier minutes exhausted),
create the release manually with `gh` CLI:

```bash
# Build locally
cargo build --release

# Create tarball
tar czf "rtco-$VERSION-$(uname -sm | tr ' ' '-').tar.gz" -C target/release rtco rtco-mcp
shasum -a 256 "rtco-$VERSION-$(uname -sm | tr ' ' '-').tar.gz" > rtco-SHA256SUMS.txt

# Create release (requires gh CLI with repo scope)
gh release create "$VERSION" --title "rtco $VERSION" --notes "Release notes here" ./*.tar.gz ./*.txt
```

### Prerequisites

- `GITHUB_TOKEN` with `contents:write` scope (default for GitHub Actions)
- An active GitHub Actions runner allocation (check billing at Settings > Billing & plans)
