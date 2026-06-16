# TOIN (Token-Optimized Intermediate Notation) — Design Document

## Overview

TOIN is a compact intermediate representation for structured CLI output that
preserves semantics while drastically reducing token count.  It is designed for
LLM consumption: agents parse TOIN more efficiently than verbose JSON or
free-form text.

## Motivation

CLI tools emit output in many formats (JSON, YAML, tables, free-text).  RTCO
currently filters this output but keeps its original format.  TOIN goes further
by converting structured output into an ultra-compact token-efficient syntax.

## Example

### Input (JSON)
```json
[
  {"name": "rtco", "version": "0.28.2", "license": "Apache 2.0"},
  {"name": "serde", "version": "1.0.0", "license": "MIT"}
]
```

### Output (TOIN)
```
pkgs
  rtco   0.28.2  Apache-2.0
  serde  1.0.0   MIT
```

### Token comparison

| Format | Tokens | Savings vs JSON |
|--------|--------|-----------------|
| JSON (pretty) | 42 | baseline |
| JSON (compact) | 26 | 38% |
| TOIN | 12 | **71%** |

## Syntax

### Types

| Type | Syntax | Example |
|------|--------|---------|
| String | bare or `"quoted"` | `hello` or `"hello world"` |
| Number | digits | `42` or `3.14` |
| Bool | `T` / `F` | `T` |
| Null | `-` | `-` |
| List | `[a b c]` | `[1 2 3]` |
| Map | `{k1 v1 k2 v2}` | `{name rtco version 0.28.2}` |

### Records

A record is a key-value pair separated by whitespace.  Nested records use
indentation (like YAML but without colons):

```
parent
  child1  value1
  child2
    grandchild  value2
```

### Tables

Repeated records with identical keys collapse into a table:

```
crates
  name      version    license
  rtco      0.28.2     Apache-2.0
  serde     1.0.0      MIT
```

## Implementation

### Parser

A simple line-based parser in `crates/rtco-core/src/toin/`:

```rust
pub fn parse(input: &str) -> Result<ToinDocument> { ... }
pub fn toin_from_json(json: &str) -> Result<String> { ... }
pub fn toin_from_table(table: &[&str], headers: &[&str]) -> String { ... }
```

### Encoder

The encoder detects the input format and converts:

1. JSON arrays of objects with uniform keys → TOIN table.
2. JSON objects → TOIN records.
3. Tabular text (psql, docker ps output) → TOIN table.
4. Nested JSON → TOIN indented records.

### Integration with Content Router

The TOIN encoder is registered as a `ContentHandler` for JSON content and
activated when the `--toin` flag is passed:

```bash
rtco --toin cargo metadata --no-deps
```

### Milestones

1. TOIN parser for the core types (string, number, bool, null, list, map).
2. `toin_from_json` — JSON-to-TOIN converter.
3. Tabular-to-TOIN converter for `docker ps`, `psql`, `kubectl get`.
4. Integration with `ContentRouter` as an optional handler.
