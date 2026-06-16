# Multi-Layer Token Savings — Design Document

## Overview

RTCO achieves 60-90% token savings through a stacked architecture of
independent compression layers.  Each layer targets a different source of
redundancy in CLI output, and they compose additively.

This document describes the layers, their contribution to total savings, and
how they interact.

## Layer Stack

```
Layer 5: Semantic deduplication    ~5-15% additional savings
Layer 4: Content-type compression  ~10-30% additional savings
Layer 3: Structural compression    ~20-40% additional savings
Layer 2: ANSI & noise stripping    ~5-10% additional savings
Layer 1: Token-aware truncation    ~20-40% additional savings
──────────────────────────────────────────────────────────
Total:                             60-90% token savings
```

### Layer 1 — Token-aware truncation

**Strategy**: When output exceeds a token budget, truncate from the middle
(maintaining head and tail) rather than cutting at an arbitrary point.

**Implementation**: `truncate.rs` — `truncate_to_token_budget()` function.

**Savings**: 20-40% on long outputs (CI logs, test runs).

**Example**: A 15,000-line test output truncated to the first 50 and last 20
lines.

### Layer 2 — ANSI & noise stripping

**Strategy**: Strip ANSI escape codes, carriage returns, and control characters
that carry no semantic content.

**Implementation**: `utils::strip_ansi()` — regex-based removal.

**Savings**: 5-10% on colorized output (cargo, clippy, pytest with --color).

**Example**: `\x1b[31mError\x1b[0m` becomes `Error`.

### Layer 3 — Structural compression

**Strategy**: Remove repetitive structural elements: blank lines, separator
lines (`------`), page breaks, timestamps, progress spinners.

**Implementation**: Per-ecosystem regex patterns in filter modules.

**Savings**: 20-40% on build tools, test runners, log tailing.

**Example**: Removing `--- pass ---` separators between each test result.

### Layer 4 — Content-type compression

**Strategy**: Apply type-specific compression: compact JSON, dedent code,
collapse stack traces, group error messages.

**Implementation**: `content_router` dispatch to specialized handlers.

**Savings**: 10-30% on structured outputs (JSON, code, diffs).

**Example**: JSON arrays condensed to single-line, duplicate error messages
grouped with occurrence counts.

### Layer 5 — Semantic deduplication

**Strategy**: Detect and merge near-identical lines (e.g. repeated error lines
with different line numbers, or similar compiler warnings).

**Implementation**: `dedup.rs` — fuzzy deduplication using line fingerprints.

**Savings**: 5-15% on build output with repeated warnings.

**Example**: 20 lines of `warning: unused import` collapsed into
`warning: unused import (x20)`.

## Interaction Between Layers

- Layers are applied **bottom-up** (Layer 1 first, Layer 5 last).
- Each layer operates on the output of the previous layer.
- A layer may disable downstream layers if its output is below a threshold
  (e.g. if L1 truncation already meets the budget, skip L2-L5).

## Tracking Per-Layer Savings

Each layer records its contribution:

```rust
pub struct LayerSavings {
    pub layer: &'static str,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub savings_percent: f64,
}
```

These are accumulated in `FilteredOutput::markers` and exposed via the gain
analytics.

## Configuration

```toml
[savings]
max_tokens = 4000          # Layer 1 budget
strip_ansi = true          # Layer 2
structural = true          # Layer 3
content_aware = true       # Layer 4
dedup = true               # Layer 5
```

Each layer can be independently disabled in the config, allowing users to
trade compression ratio for speed.
