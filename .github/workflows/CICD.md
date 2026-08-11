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
     │  --locked            │
     │  --all-targets       │
     │  -D warnings         │
     └────────┬─────────────┘
              │
     ┌────────▼─────────┐
     │  cargo test      │
     │  --locked        │
     │  --features      │
     │  prometheus      │
     └────────┬─────────┘
          ┌───┴───┐
     ┌────▼────┐  │
     │ coverage│  │  (cargo tarpaulin)
     └─────────┘  │
          ┌───────▼────────────┐
          │  security scan     │
          │  (cargo-audit +    │
          │   dangerous-patterns│
          │   + new deps)      │
          └───────┬────────────┘
          ┌───────▼────────────┐
          │  semgrep scan      │
          │  (diff on PR,      │
          │   whole tree on push)│
          └───────┬────────────┘
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

All cargo commands use `--locked` so CI builds against the committed
`Cargo.lock` — fresh resolves cannot silently upgrade deps and break the build.

## Release

### Release-please (automated)

On every push to `main`, `release-please.yml` opens/updates a release PR that
bumps the version in `Cargo.toml`, `crates/rtco-core/Cargo.toml`, and
`crates/rtco-cli/Cargo.toml`, and updates `CHANGELOG.md`. Merging the release
PR creates a `v*` tag, which triggers `release.yml`.

`release-please-config.json` uses `release-type: simple` (the workspace root
has `[workspace.package]`, not `[package]`) and bumps all three version
locations via `extra-files`.

### release.yml (builds on v* tag)

Trigger: tag push matching `v*`

- `release` matrix builds native binaries for Linux, macOS, Windows
  (asset name: `rtco-<version>-<os>-<arch>.<ext>`)
- `cross-build` cross-compiles `aarch64-unknown-linux-gnu`,
  `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` via `cross`
- `checksums` job downloads every artifact and aggregates one `checksums.txt`
  (GNU `sha256sum` format) — install.sh/install.ps1 verify against it

### Manual release (when CI runners are unavailable)

If GitHub Actions runners are not allocating (e.g. free tier minutes exhausted),
create the release manually with `gh` CLI:

```bash
# Build locally
cargo build --release

# Create tarball
tar czf "rtco-$VERSION-$(uname -sm | tr ' ' '-').tar.gz" -C target/release rtco
shasum -a 256 "rtco-$VERSION-$(uname -sm | tr ' ' '-').tar.gz" > checksums.txt

# Create release (requires gh CLI with repo scope)
gh release create "$VERSION" --title "rtco $VERSION" --notes "Release notes here" ./*.tar.gz checksums.txt
```

### Prerequisites

- `GITHUB_TOKEN` with `contents:write` scope (default for GitHub Actions)
- **Repo settings → Actions → General → Workflow permissions must allow
  GitHub Actions to create/approve pull requests** — otherwise release-please
  fails with "GitHub Actions is not permitted to create or approve pull
  requests" (set `default_workflow_permissions: write` +
  `can_approve_pull_request_reviews: true`).
- An active GitHub Actions runner allocation (check billing at Settings > Billing & plans)
