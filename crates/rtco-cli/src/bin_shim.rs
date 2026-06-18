//! Re-exports the `rtco-mcp` binary's source so the same JSON-RPC loop
//! can be invoked from the `rtco mcp` subcommand.
//!
//! In Cargo's default layout, every `*.rs` file under `src/bin/` is a
//! separate binary target — they are NOT modules of the parent crate.
//! We want both:
//!   1. `cargo build --bin rtco-mcp` to keep producing the standalone
//!      binary that uses `src/bin/mcp_server.rs` directly as its entry.
//!   2. `rtco mcp` (the `Commands::Mcp` arm in `src/main.rs`) to call
//!      the same JSON-RPC loop as a library function.
//!
//! Cargo does not allow a single source file to be both a bin target's
//! `fn main` and a `pub fn` callable from a sibling module. So we keep
//! the file under `src/bin/mcp_server.rs` for the bin target, and use
//! `#[path]` here to re-include the same source as a normal module
//! inside the rtco-cli crate. Single source of truth.

#[path = "bin/mcp_server.rs"]
pub mod mcp_server;
