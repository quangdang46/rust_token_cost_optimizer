# Unwrap Audit Report

Date: 2026-06-16
Scope: `crates/rtco-cli/src/` and `crates/rtco-core/src/`

## Summary

| Category | Count | Verdict |
|----------|-------|---------|
| Test code (`#[cfg(test)]` / `#[test]`) | ~180 | Acceptable (prefer `expect()` but non-critical) |
| `lazy_static!` regex init | ~40 | Acceptable per RTCO pattern (bad regex = programming error) |
| `Mutex::lock().unwrap()` | ~20 | Acceptable (poison = fatal in single-threaded design) |
| Doc tests / example comments | ~15 | Acceptable |
| **True production unwraps** | **0** | **All clean** |

## Findings

### crates/rtco-cli/src/main.rs
- 20 `unwrap()` calls, all in `#[cfg(test)]` module (line 2590+). Zero production unwraps.

### crates/rtco-core/src/
- All `unwrap()` calls are one of:
  - `lazy_static!` regex initialization (established RTCO pattern, e.g. `ANSI_RE`, `HEX_RE`)
  - `Mutex::lock().unwrap()` in CCR store/memory (poison = unrecoverable)
  - `#[cfg(test)]` modules and `#[test]` functions
  - Doc comments (`//!`) showing example code

## Critical Production Unwraps Found: 0

No critical production unwraps found. All `unwrap()` calls are in safe contexts (test code, lazy_static initialization, Mutex locks, or doc examples).

## Notes

- The `content_detector.rs` has `Regex::new(...).unwrap()` outside `lazy_static!` at lines 62, 64, 68, 77, 84 but these ARE inside `lazy_static!` blocks (nested inside a function-scoped lazy_static). Verified.
- `crates/rtco-cli/src/main.rs` was clean after confirming all unwraps are in `#[cfg(test)]` module.
- `stream.rs` has extensive test module at `pub mod tests` (line 547) containing all unwraps after that line.
- `tee.rs` has `mod tests` at line 505 containing all unwraps after that line.
- `ccr/store.rs` has `mod tests` at line 147 containing all unwraps after that line.
- `signals/detectors.rs` and `signals/tiered.rs` unwraps are in `#[cfg(test)]` modules.
- `compressors/*` unwraps are all inside `#[cfg(test)]` modules.
