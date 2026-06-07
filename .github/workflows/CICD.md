# CI/CD Flows

## CI (ci.yml)

Trigger: push or pull_request to `main`

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
     └──────────────────┘
```

## CD (cd.yml)

Trigger: push to `main` (or workflow_dispatch)

Runs the same quality checks as CI on push to main.

## Manual release

Not configured for this fork. Use `cargo build --release` locally to build binaries.
