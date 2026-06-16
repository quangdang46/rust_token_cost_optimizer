# Contributing to rtco

## Prerequisites

- **Rust**: 1.80+ (install via [rustup](https://rustup.rs/))
- **Cargo**: Included with Rust
- **Git**: For source control

## Build

```bash
# Clone and build
git clone https://github.com/rtco-ai/rtco.git
cd rtco
cargo build --workspace

# Release build
cargo build --release
```

## Test

```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test <test_name>

# Module tests
cargo test <module>::

# Review snapshots (insta)
cargo insta review
cargo insta accept

# Run ignored (integration) tests
cargo test --ignored
```

## Quality Gate

**All three must pass before committing:**

```bash
cargo fmt --all
cargo clippy --all-targets
cargo test --workspace
```

## Project Structure

```
rtco/
├── crates/
│   ├── rtco-cli/          # CLI binary and command modules
│   └── rtco-core/         # Core library (config, tracking, utils)
├── docs/                  # Documentation
├── tests/                 # Test fixtures and integration tests
│   └── fixtures/          # Real command output for tests
```

## Adding a New Command Filter

1. **Create module**: Add `<cmd>_cmd.rs` in the appropriate `src/cmds/<ecosystem>/` directory
2. **Register**: Add variant to `Commands` enum in `crates/rtco-cli/src/main.rs`
3. **Add routing**: Wire the `run()` function in `main.rs`
4. **Implement filter**: Write `run()` returning `Result<()>`
5. **Add fixture**: Capture real command output to `tests/fixtures/`
6. **Write tests**:
   - Snapshot test with `assert_snapshot!()`
   - Token savings test (verify >=60% savings)
   - Edge case tests (empty, malformed, unicode)
7. **Run quality gate**: `cargo fmt && cargo clippy && cargo test`
8. **Review snapshots**: `cargo insta review`

## Commit Conventions

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<optional body>
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci`

Scope examples: `git`, `cargo`, `js`, `python`, `go`, `ruby`, `core`, `hooks`, `docs`, `ci`

Examples:
```
feat(git): add ultra-compact mode for git status
fix(core): prevent panic on UTF-8 truncation boundary
docs(architecture): update module count to 64
```

## Code Style

- Follow Rust 2024 idioms
- Use `anyhow::Result` with `.context("...")?` everywhere
- No `unwrap()` in production code (use `.context()?` or `expect()` in tests)
- All regex must use `lazy_static!` (never compile inside a function)
- No async/tokio — single-threaded by design
- Use iterator chains over manual loops
- Prefer `&str` over `&String` in function signatures

## Pull Request Process

1. Ensure all tests pass and quality gate is clean
2. Update CHANGELOG.md if adding features or fixing bugs
3. Add or update test fixtures with real command output
4. Verify token savings >=60% for filter changes
5. Request review from maintainers
6. Squash merge into `main`

## Getting Help

- Open an issue on [GitHub](https://github.com/rtco-ai/rtco/issues)
- For design questions, refer to [ARCHITECTURE.md](docs/contributing/ARCHITECTURE.md)
