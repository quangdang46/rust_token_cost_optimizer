# Refactor main.rs Plan

## Current State

`crates/rtco-cli/src/main.rs` is 3331 lines and growing. It contains:

- **~790 lines**: Enum definitions for all commands (Commands, GitCommands, PnpmCommands, DockerCommands, KubectlCommands, PrismaCommands, DotnetCommands, GoCommands, GtCommands, HookCommands, etc.)
- **~1500 lines**: The main dispatch `match` block in `run_cli()` routing each `Commands::*` variant to its module
- **~500 lines**: Passthrough command validation, meta-command handling, global flag extraction
- **~500 lines**: Tests

## Problems

1. **Single file, single responsibility violation**: enum definitions, dispatch logic, passthrough rules, and validation all live in one file
2. **Merge conflicts**: every new command touches `main.rs` in the enum, the dispatch match, and the passthrough list
3. **Test isolation**: command dispatch tests are coupled to the full enum, making targeted testing hard
4. **Cognitive load**: developers must scroll through 3300 lines to understand command routing

## Proposed Structure

```
crates/rtco-cli/src/
├── main.rs              # Entry point only (~150 lines)
├── cli.rs               # Clap-derived CLI definition (Commands enum + sub-enums)
├── dispatch.rs          # Command dispatch (routing Commands::* to module run())
├── passthrough.rs       # Passthrough logic + command validation
└── tracking.rs          # Common tracking integrations
```

## File Responsibilities

### cli.rs (extracted from main.rs lines 30-1130)

- All `#[derive(Parser)]` and `#[derive(Subcommand)]` enums and structs
- The `Cli` struct with global flags (verbose, ultra_compact)
- `Commands` enum with all subcommand variants
- All secondary enums: GitCommands, PnpmCommands, DockerCommands, etc.
- `AgentTarget` enum
- No logic, pure type definitions

### dispatch.rs (extracted from main.rs lines 1404-2400)

- `fn dispatch(cli: Cli) -> Result<i32>`
- The large `match cli.command` block
- Calls into module `run()` functions
- Tracks execution via `TimedExecution`
- Returns exit codes

### passthrough.rs (extracted from main.rs lines 1140-1400 + scattered)

- `RTCO_META_COMMANDS` constant
- `fn run_fallback()`
- `fn is_operational_command()`
- `fn has_stdin_redirects()`
- `fn should_silence_fallback()`

### main.rs (reduced, ~150 lines)

```rust
mod cli;
mod dispatch;
mod passthrough;

fn run_cli() -> Result<i32> {
    migrate_data_dir_once();
    rtco_core::telemetry::maybe_ping();
    let cli = match Cli::try_parse() { ... };
    dispatch(cli)
}

fn main() {
    std::process::exit(run_cli().unwrap_or_else(|e| { ... }));
}
```

## Migration Strategy

### Phase 1: Extract cli.rs

1. Create `crates/rtco-cli/src/cli.rs`
2. Move all `#[derive(Parser, Subcommand)]` enums and imports
3. Export everything `pub use cli::*` from a compatibility shim
4. Verify `cargo build && cargo test`

### Phase 2: Extract passthrough.rs

1. Create `crates/rtco-cli/src/passthrough.rs`
2. Move `RTCO_META_COMMANDS`, `run_fallback()`, `is_operational_command()`
3. Update imports in main.rs

### Phase 3: Extract dispatch.rs

1. Create `crates/rtco-cli/src/dispatch.rs`
2. Move the main dispatch match block
3. Move `run_cli()` dispatch logic
4. Keep `migrate_data_dir_once()`, `main()` in main.rs

### Phase 4: Clean up

1. Remove compatibility shims
2. Update module declarations
3. Run `cargo fmt && cargo clippy && cargo test`

## Risks

| Risk | Mitigation |
|------|------------|
| Broken imports during extraction | Use `pub use` re-exports in each phase, verify after each |
| Circular dependencies | Dispatch will import cli types; modules import from dispatch |
| Merge conflicts for in-flight PRs | Complete in one focused session, no partial state |

## Expected Benefits

- main.rs shrinks from 3331 lines to ~150 lines
- Each new file has a single responsibility
- Commands enum changes no longer cause merge conflicts in the dispatch logic
- Passthrough rules are testable in isolation
- Developers find command definitions in one place (cli.rs), routing in another (dispatch.rs)
