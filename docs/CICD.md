# CI/CD Pipeline

## Overview

rtco uses GitHub Actions for continuous integration and delivery. The pipeline is defined in `.github/workflows/` with separate workflows for CI (testing) and CD (release).

## Workflows

### CI (`ci.yml`)

Triggered on every push and pull request to `main`. Runs:

| Step | Description |
|------|-------------|
| **Build** | `cargo build --workspace` on macos-latest, ubuntu-latest, windows-latest |
| **Test** | `cargo test --workspace` across all three platforms |
| **Lint** | `cargo clippy --all-targets -- -D warnings` |
| **Format** | `cargo fmt --all --check` |
| **Doc tests** | `cargo test --doc` |

### CD (`cd.yml`)

Triggered by pushing a version tag (`v*.*.*`). Builds release binaries and publishes:

| Step | Description |
|------|-------------|
| **Build release** | `cargo build --release --workspace` for macos/linux/windows |
| **Package** | Create `.tar.gz` (macos/linux) and `.zip` (windows) archives |
| **Upload artifacts** | Attach release artifacts to the GitHub release |
| **Publish** | `cargo publish` to crates.io |
| **Homebrew** | Trigger Homebrew formula update |

## Local CI Simulation

```bash
# Run the same checks CI performs
cargo fmt --all && cargo clippy --all-targets && cargo test --workspace
```

## Release Process

1. **Branch**: Ensure `main` is up to date with `git pull`
2. **Version bump**: Update version in `Cargo.toml` (workspace + crate)
3. **Changelog**: Update `CHANGELOG.md` following Keep a Changelog format
4. **Tag**: `git tag v<major>.<minor>.<patch> && git push origin --tags`
5. **Release**: CD workflow builds binaries, creates GitHub release, publishes to crates.io

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `CARGO_REGISTRY_TOKEN` | crates.io publish token (GitHub secret) |
| `HOMEBREW_TOKEN` | Token for Homebrew formula update (GitHub secret) |

## Snapshot Testing

The CI pipeline includes snapshot testing via `insta`:

```bash
# Review snapshots locally before pushing
cargo insta review
cargo insta accept
```

Snapshots are stored in `src/cmds/<ecosystem>/snapshots/` and are committed to the repository.

## Performance Benchmarks

Run manually to verify <10ms startup time:

```bash
hyperfine 'cargo run --release -- git status' 'git status' --warmup 3
```
