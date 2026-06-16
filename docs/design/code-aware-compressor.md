# Code-Aware Compressor — Design Document

## Overview

The code-aware compressor reduces source code output (from grep, cat, ls of
source directories, or `cargo expand`-like tools) by applying language-specific
transformations that preserve semantics while reducing token count.

## Compression Strategies

### 1. Comment stripping

Remove comments that add no structural information:

| Language | Comment syntax | Strip? | Rationale |
|----------|---------------|--------|-----------|
| Rust | `//`, `//!`, `///` | Doc comments kept, regular stripped | Doc comments carry meaning |
| Python | `#`, `"""` | Stripped | Usually noise for LLM |
| JS/TS | `//`, `/* */` | Stripped | License headers = noise |
| Go | `//` | Stripped | Doc comments kept |
| All | `// SPDX`, `// Copyright` | Stripped | License boilerplate |

### 2. Blank line condensation

Multiple consecutive blank lines are condensed to a single blank line,
preserving paragraph separation without wasting tokens on whitespace.

### 3. Indentation compression

Deeply nested code has its indentation reduced proportionally:

```
// Before (16-space indent in deeply nested code)
                let x = 1;

// After (2-space indent)
let x = 1;
```

The compressor maintains relative indentation differences but shifts the
baseline to 2 spaces.

### 4. Import condensing

Multi-line imports are condensed to a single line:

```
// Before
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, Read, Write};

// After
use std::collections::{HashMap,HashSet,VecDeque};
use std::io::{self,Read,Write};
```

### 5. Type annotation stripping (optional)

When `--strip-types` is passed, obvious type annotations are removed when the
type is clear from context (e.g. `let x: i32 = 42` -> `let x = 42`).

## Language Detection

The compressor uses the file extension (from grep/cat output containing file
paths) or falls back to content heuristics via `content_detector`.

| Extension | Language | Active strategies |
|-----------|----------|-------------------|
| `.rs` | Rust | 1, 2, 3, 4 |
| `.py` | Python | 1, 2, 3 |
| `.js`, `.ts`, `.jsx`, `.tsx` | JS/TS | 1, 2, 3, 4 |
| `.go` | Go | 1, 2, 3 |
| `.java`, `.kt` | JVM | 1, 2, 3, 4 |
| `.c`, `.h`, `.cpp`, `.hpp` | C/C++ | 1, 2, 3 |
| `.rb` | Ruby | 1, 2, 3 |
| `.rs` (doc comments) | Rust | 1 (keep `///`) |

## Implementation

### Module location

`crates/rtco-core/src/compressors/code.rs`

```rust
pub struct CodeCompressor {
    pub strip_comments: bool,
    pub condense_blank_lines: bool,
    pub compress_indentation: bool,
    pub condense_imports: bool,
    pub strip_types: bool,
}

impl CodeCompressor {
    pub fn compress(&self, input: &str, language: &str) -> String { ... }
}
```

### Integration

The compressor plugs into the `ContentRouter` as the `ContentType::Code`
handler, and is also invoked by ecosystem filters that detect code output
(e.g. `cat` of a source file, `grep` with code context).

### Configuration

```toml
[compressor.code]
strip_comments = true
condense_blank_lines = true
compress_indentation = true
condense_imports = true
strip_types = false
```

### Milestones

1. Comment stripping for Rust, Python, JS/TS (regex-based).
2. Blank line condensation + indentation compression.
3. Import condensing (Rust + JS/TS).
4. Language detection integration.
5. Optional type annotation stripping.
6. Preceding `content_detector` integration.
