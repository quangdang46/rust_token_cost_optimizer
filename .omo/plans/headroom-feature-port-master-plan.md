# Headroom Feature Port — Master Plan

> **Project**: rtco (rust_token_cost_optimizer)
> **Source**: headroom (context compression toolkit for AI agents)
> **Goal**: Port 10+ headroom compression transforms to rtco's pure-Rust CLI proxy architecture
> **Status**: Planning Phase
> **Total Effort**: ~8–12 weeks (full-time)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture Comparison](#2-architecture-comparison)
3. [Dependency Map & Timeline](#3-dependency-map--timeline)
4. [Phase 1: Token Estimation + Signal System](#4-phase-1-token-estimation--signal-system)
5. [Phase 2: CCR (Compression Context Registry)](#5-phase-2-ccr-compression-context-registry)
6. [Phase 3: Pipeline Orchestrator](#6-phase-3-pipeline-orchestrator)
7. [Phase 4: SmartCrusher (JSON Compression)](#7-phase-4-smartcrusher-json-compression)
8. [Phase 5: DiffCompressor](#8-phase-5-diffcompressor)
9. [Phase 6: LogCompressor](#9-phase-6-logcompressor)
10. [Phase 7: SearchCompressor](#10-phase-7-searchcompressor)
11. [Phase 8: Anchor Selector + CacheAligner](#11-phase-8-anchor-selector--cachealigner)
12. [Integration into rtco](#12-integration-into-rtco)
13. [Testing Strategy](#13-testing-strategy)
14. [Performance Targets](#14-performance-targets)
15. [Files & Modules Reference](#15-files--modules-reference)
16. [Risks & Mitigations](#16-risks--mitigations)

---

## 1. Executive Summary

### What is headroom?
headroom is a Python/TypeScript + Rust extension context compression toolkit. It provides transforms that compress LLM-bound content (tool outputs, logs, diffs, search results, JSON) by 40–80% while preserving semantic meaning. Its Rust core (`headroom-core`) contains 15+ compression modules.

### What is rtco?
rtco is a pure Rust CLI proxy that interposes LLM commands (git, cargo, npm, etc.) and compresses their stdout/stderr before they reach the AI. It currently uses adaptive sizing (SimHash + Kneedle), content detection, and keyword detection — roughly 15% of headroom's capability.

### What we're porting
10 modules from headroom's Rust core to rtco's architecture:

| Module | headroom LOC | Priority | rtco Integration | Token Saving Boost |
|--------|-------------|----------|-------------------|-------------------|
| Token Estimation | ~400 | **P0** | Core dependency | Foundation |
| Line Importance Signals | ~350 | **P0** | Core dependency | Foundation |
| CCR (SQLite) | ~500 | **P0** | Core dependency | Foundation |
| Pipeline Orchestrator | ~800 | **P1** | Architecture | 5–10% |
| SmartCrusher (JSON) | ~2500 | **P1** | Generic filter | 15–25% |
| DiffCompressor | ~1700 | **P1** | `git diff` filter | 10–20% |
| LogCompressor | ~1300 | **P2** | Multiple filters | 10–15% |
| SearchCompressor | ~900 | **P2** | `grep`/`rg` filter | 10–15% |
| Anchor Selector | ~1200 | **P3** | Cross-module | 5–10% |
| CacheAligner | ~400 | **P3** | Edge utility | 2–5% |

### Key Design Decisions
- **Pure Rust only**: No Python/TypeScript deps (rtco constraint)
- **Minimal new deps**: Prefer what's already in Cargo.toml; add crates only when essential
- **Fallback on failure**: Every transform degrades gracefully — raw output passes through
- **Progressive enhancement**: New transforms layer onto existing filters; no filter rewrites
- **SQLite for CCR**: Already in rtco dep tree; Redis support not needed

---

## 2. Architecture Comparison

### headroom Pipeline Model
```
Input → Signatures → [ReformatTransform×N] → [OffloadTransform×N] → Output
                          (serial)                (parallel)
```

- `ReformatTransform`: pack denser without dropping (JsonMinifier, LogTemplate)
- `OffloadTransform`: drop + store original via CCR
- Pipeline runs once per document

### rtco Filter Model
```
Command → Parse(streaming) → [Filter×N] → Render → Output
                 ↓
          AdaptiveSizer (SimHash + Kneedle)
```

- Filters run per-command (git filter for git, npm filter for npm, etc.)
- No pipeline abstraction per se; each filter has its own logic
- AdaptiveSizer runs last, applying truncation based on token budget

### Mapping: headroom → rtco

| headroom Concept | rtco Equivalent | Port Strategy |
|-----------------|-----------------|---------------|
| `Tokenizer::count_text()` | `TokenCounter` trait | New module `src/core/tokenizer/` |
| `LineImportanceDetector` | `ScoreFn` per filter | New trait, per-filter impls |
| `CcrStore` | `CcrStore` trait | New module `src/core/ccr/` |
| `CompressionPipeline` | `FilterPipeline` | New module `src/core/pipeline/` |
| `SmartCrusher` | `src/cmds/json/` | New filter (generic command) |
| `DiffCompressor` | `src/cmds/git/diff.rs` | Extend git filter |
| `LogCompressor` | Multi-module | Shared lib `src/core/log_compressor/` |
| `SearchCompressor` | `src/cmds/search/` | New filter |
| `AnchorSelector` | `AnchorSelector` | Util for multiple filters |
| `AdaptiveSizer` | Already in rtco | Upgrade with signals |

### New Directory Structure
```
src/
├── cmds/
│   ├── json/           ← NEW: SmartCrusher-based filter
│   ├── search/         ← NEW: SearchCompressor-based filter
│   └── ... (existing)
├── core/
│   ├── tokenizer/      ← NEW: multi-backend token estimation
│   ├── ccr/            ← NEW: reversible compression
│   ├── pipeline/       ← NEW: configurable transform pipeline
│   ├── signals/        ← NEW: line importance scoring
│   ├── compressors/    ← NEW: DiffCompressor, LogCompressor, SearchCompressor
│   ├── anchor/         ← NEW: anchor selector
│   └── ... (existing)
```

---

## 3. Dependency Map & Timeline

### Dependency Graph
```
Phase 1: Token Estimation
              │
              ▼
Phase 1b: Line Importance Signals
              │
              ▼
Phase 2: CCR (SQLite) ──────────────┐
              │                      │
              ▼                      ▼
Phase 3: Pipeline Orchestrator ── Phase 4: SmartCrusher
              │                      │
              ├──────────────────────┤
              ▼                      ▼
Phase 5: DiffCompressor ────── Phase 6: LogCompressor
              │
              ▼
Phase 7: SearchCompressor
              │
              ▼
Phase 8: Anchor Selector + CacheAligner
```

**Parallelization opportunities:**
- Phase 1 + 2 are sequential (token estimation needed by everything else; CCR needed by offload transforms)
- Phase 3 (pipeline) can start in parallel with Phase 4 (SmartCrusher) after Phase 1+2
- Phase 5+6+7 can all proceed in parallel after Phase 3+4
- Phase 8 is lowest priority, can start after Phase 3

### Timeline (full-time, single developer)

| Phase | Description | Duration | Parallel |
|-------|-------------|----------|----------|
| P1 | Token Estimation + Signals | 1–2 weeks | Sequential |
| P2 | CCR (SQLite) | 1 week | Sequential |
| P3 | Pipeline Orchestrator | 1–2 weeks | After P1+2 |
| P4 | SmartCrusher | 2–3 weeks | Parallel w/ P3 |
| P5 | DiffCompressor | 1–2 weeks | After P3+4 |
| P6 | LogCompressor | 1–2 weeks | After P3+4 |
| P7 | SearchCompressor | 1 week | After P3+4 |
| P8 | Anchor Selector + CacheAligner | 1 week | After P3 |
| **Total** | | **8–12 weeks** | |

---

## 4. Phase 1: Token Estimation + Signal System

### 4.1 Token Estimation (`src/core/tokenizer/`)

**headroom reference**: `tokenizer/estimator.rs`, `tiktoken_impl.rs`, `hf_impl.rs`, `registry.rs`

**Goal**: Accurately estimate token count of filtered output so adaptive sizer can make informed truncation decisions.

#### Architecture

```rust
/// Core trait
pub trait Tokenizer: Send + Sync {
    fn count_text(&self, text: &str) -> usize;
    fn name(&self) -> &str;
}

/// Enum for strategy pattern
pub enum TokenizerKind {
    Approximate(text.len() / 3.5),          // Always available, zero deps
    TikToken(Box<dyn Tokenizer>),            // tiktoken-rs (optional feature)
    HuggingFace(Box<dyn Tokenizer>),         // tokenizers crate (optional feature)
}

pub struct TokenizerRegistry {
    available: Vec<(String, Box<dyn Tokenizer>)>,
    preferred: String,                       // Configurable
}
```

#### File Breakdown

| File | Purpose | headroom LOC | rtco LOC (est) |
|------|---------|-------------|----------------|
| `src/core/tokenizer/mod.rs` | Trait definition, re-exports | — | 50 |
| `src/core/tokenizer/estimator.rs` | Approximate (char/4 ≈ tokens) | 50 | 30 |
| `src/core/tokenizer/tiktoken_impl.rs` | tiktoken-rs backend (feature-gated) | 120 | 100 |
| `src/core/tokenizer/hf_impl.rs` | tokenizers crate backend (feature-gated) | 130 | 110 |
| `src/core/tokenizer/registry.rs` | Registry with auto-detect + preferred | 100 | 70 |

#### Dependencies
- **Required**: none beyond existing
- **Optional**: `tiktoken-rs` (gated behind `tokenizer-tiktoken` feature)
- **Optional**: `tokenizers` (gated behind `tokenizer-hf` feature)

#### Integration
- `AdaptiveSizer` gets a `&dyn Tokenizer` parameter
- Config: `tokenizer = "approximate" | "tiktoken" | "huggingface"` in rtco config
- Default: approximate (zero deps, <1μs per call)
- TokenizerRegistry auto-detects available backends at init

#### Testing
- Unit tests: known strings with known token counts (from headroom fixtures)
- Integration: verify TikToken backend produces same counts as Python tiktoken
- Performance: <5μs per count_text call for approximate, <50μs for tiktoken

---

### 4.2 Line Importance Signals (`src/core/signals/`)

**headroom reference**: `signals/line_importance.rs`, `signals/tiered.rs`, `signals/keyword_detector.rs`

**Goal**: Score each line of output to guide truncation — preserve important lines, drop noise.

#### Architecture

```rust
pub enum SignalCategory {
    Error, Warning, Summary, Data, Metadata, Separator, Noise,
}

pub struct ImportanceSignal {
    pub category: SignalCategory,
    pub priority: f64,        // 0.0 (drop first) to 1.0 (keep last)
    pub confidence: f64,      // 0.0 to 1.0
}

pub enum SignalContext {
    Text,
    Search,
    Diff,
    Log,
    Json,
    Generic,
}

pub trait LineImportanceDetector: Send + Sync {
    fn score(&self, line: &str, context: &SignalContext) -> ImportanceSignal;
    fn context_hint(&self) -> SignalContext;
}

/// Composition via tiered pipeline
pub struct TieredDetector {
    detectors: Vec<Box<dyn LineImportanceDetector>>,
}

impl LineImportanceDetector for TieredDetector {
    fn score(&self, line: &str, context: &SignalContext) -> ImportanceSignal {
        self.detectors
            .iter()
            .map(|d| d.score(line, context))
            .max_by_key(|s| (s.priority * s.confidence) as u64)
            .unwrap_or(/* low-priority fallback */)
    }
}
```

#### Built-in Detectors

| Detector | Priority Boost | Confidence |
|----------|---------------|------------|
| Error pattern (`error:`, `FAILED`, `Error:`) | 0.9–1.0 | 0.95 |
| Warning pattern (`warning:`, `WARN`) | 0.7–0.8 | 0.85 |
| Summary indicators (`===`, `---`, `Summary:`) | 0.5–0.7 | 0.75 |
| Numeric data lines (stats, counts) | 0.4–0.6 | 0.70 |
| Path/URL lines | 0.3–0.5 | 0.60 |
| Separator lines (`---`, `===`, `****`) | 0.1 | 0.90 |
| Empty/whitespace lines | 0.0 | 1.0 |

#### File Breakdown

| File | Purpose | headroom LOC | rtco LOC (est) |
|------|---------|-------------|----------------|
| `src/core/signals/mod.rs` | Traits + types | 80 | 80 |
| `src/core/signals/detectors.rs` | Built-in detectors (error, warning, etc.) | 120 | 150 |
| `src/core/signals/tiered.rs` | TieredDetector composition | 60 | 50 |
| `src/core/signals/keyword_detector.rs` | Configurable keyword-based detector | 90 | 80 |

#### Integration with AdaptiveSizer

The current `AdaptiveSizer` computes a similarity budget and truncates. With signals:

1. Each line gets a score via `LineImportanceDetector`
2. Lines are sorted by priority descending
3. Cut position chosen by: keep all lines with priority > threshold, fill remaining budget with next-highest
4. AdaptiveSizer still uses SimHash for similarity detection, but signals guide which lines to keep

---

## 5. Phase 2: CCR (Compression Context Registry)

**headroom reference**: `ccr/` — `in_memory.rs`, `sqlite.rs`, `redis.rs`, `lib.rs`

**Goal**: Store original content before compression so it can be restored if needed. Replace truncated lines with a marker (`<<ccr:HASH>>`) and save the original to SQLite.

### Architecture

```rust
pub trait CcrStore: Send + Sync {
    fn put(&self, key: &str, value: &[u8]) -> Result<()>;
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn len(&self) -> usize;
    fn contains(&self, key: &str) -> bool;
}

pub fn compute_key(data: &[u8]) -> String {
    // BLAKE3 hash, first 24 hex characters
    let hash = blake3::hash(data);
    hash.to_hex()[..24].to_string()
}

pub fn marker_for(key: &str) -> String {
    format!("<<ccr:{}>>", key)
}
```

### SQLite Schema

```sql
CREATE TABLE IF NOT EXISTS ccr_entries (
    hash TEXT PRIMARY KEY,
    original BLOB NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    ttl_seconds INTEGER
);
-- Index for lazy purge
CREATE INDEX IF NOT EXISTS idx_ccr_created ON ccr_entries(created_at);
```

### Implementation Notes

- **WAL mode** for concurrent reads (already in rtco's SQLite usage)
- **Lazy purge**: On `get()`, check if entry is expired and delete silently
- **Configurable TTL**: Default 7 days; `ttl_seconds = 0` means no expiry
- **Offload trigger**: Only store lines > configurable length threshold (default: 100 chars)

### File Breakdown

| File | Purpose | headroom LOC | rtco LOC (est) |
|------|---------|-------------|----------------|
| `src/core/ccr/mod.rs` | CcrStore trait, compute_key, marker_for | 80 | 80 |
| `src/core/ccr/store.rs` | SqliteCcrStore impl | 300 | 300 |
| `src/core/ccr/memory.rs` | InMemoryCcrStore (for testing) | 100 | 80 |

### Integration Points

1. **Filter pipeline**: Offload transforms call `ccr_store.put()` before dropping lines
2. **Tracking module**: CCR stats (stored items, bytes saved) go into `src/core/tracking.rs`
3. **`rtco gain`**: Show CCR hit rate, total stored bytes

---

## 6. Phase 3: Pipeline Orchestrator

**headroom reference**: `transforms/pipeline/` — `mod.rs`, `config.rs`, `reformat.rs`, `offload.rs`

**Goal**: Provide a configurable transform pipeline that filters can optionally use. Encapsulate the reformat→offload flow as a reusable abstraction.

### Architecture

```rust
pub trait ReformatTransform: Send + Sync {
    fn name(&self) -> &str;
    fn reformat(&self, input: &str) -> Result<String>;
    /// Estimated token savings (0.0–1.0)
    fn estimated_savings(&self) -> f64;
}

pub trait OffloadTransform: Send + Sync {
    fn name(&self) -> &str;
    fn estimate_bloat(&self, input: &str, signal: &ImportanceSignal) -> bool;
    fn apply(&self, input: &str, store: &dyn CcrStore) -> Result<String>;
}

pub struct PipelineConfig {
    pub max_tokens: usize,                       // Target budget
    pub reformat_transforms: Vec<String>,        // Names to apply
    pub offload_threshold: f64,                  // Bloat ratio to trigger offload
    pub enable_ccr: bool,
}

pub struct CompressionPipeline {
    pub config: PipelineConfig,
    pub reformatters: Vec<Box<dyn ReformatTransform>>,
    pub offloaders: Vec<Box<dyn OffloadTransform>>,
    pub signal_detector: Box<dyn LineImportanceDetector>,
    pub tokenizer: Box<dyn Tokenizer>,
    pub ccr_store: Option<Arc<dyn CcrStore>>,
}

impl CompressionPipeline {
    pub fn run(&self, input: &str) -> Result<String> {
        // 1. Reformat phase (serial) — pack denser
        let mut output = input.to_string();
        for t in &self.reformatters {
            output = t.reformat(&output)?;
        }
        // 2. Offload phase (parallel via rayon) — drop + store
        let lines: Vec<&str> = output.lines().collect();
        //    ... score each line, estimate bloat, offload if threshold exceeded
        // 3. Return compressed output
    }
}
```

### Config (TOML)

```toml
[pipeline]
max_tokens = 4096
reformat_transforms = ["json_minify"]
offload_threshold = 0.3
enable_ccr = true
```

### File Breakdown

| File | headroom LOC | rtco LOC (est) |
|------|-------------|----------------|
| `src/core/pipeline/mod.rs` | 150 | 150 |
| `src/core/pipeline/config.rs` | 100 | 100 |
| `src/core/pipeline/reformat.rs` | ReformatTransform trait + impls | 200 |
| `src/core/pipeline/offload.rs` | OffloadTransform trait + impls | 200 |
| `src/core/pipeline/runner.rs` | Execution logic | 200 |

### Integration

- Pipeline is **optional** — filters can continue to work independently
- `PipelineConfig` loaded from rtco config file
- Each filter module can opt-in: `FilterResult::Pipeline(CompressionPipeline::new(config))`
- Pipeline wraps existing per-filter compression logic

---

## 7. Phase 4: SmartCrusher (JSON Compression)

**headroom reference**: `transforms/smart_crusher/` (21 files)

**Goal**: Compress JSON output from any command (curl JSON APIs, jq output, npm audit --json, etc.) by classifying JSON arrays and applying per-field compression strategies.

### Architecture (Simplified for rtco)

headroom's SmartCrusher is 21 files. For rtco, we need a focused subset:

```rust
/// Compression strategy for a JSON field
pub enum CompressionStrategy {
    None,              // Keep as-is
    Skip,              // Drop entirely
    TopN(usize),       // Keep first N elements
    Sample(f64),       // Keep ~fraction of elements
    ClusterSample(usize), // K-means cluster → keep centroids
}

pub struct FieldSpec {
    pub path: Vec<String>,       // JSON path to field
    pub strategy: CompressionStrategy,
    pub max_items: usize,        // For TopN / ClusterSample
}

pub struct SmartCrusherConfig {
    pub field_strategies: Vec<FieldSpec>,
    pub max_depth: usize,
    pub min_array_size: usize,   // Only crush arrays longer than this
    pub preserve_top_level_keys: bool,
}
```

### Pipeline Within SmartCrusher

1. **Parse** JSON with `serde_json::Value` (already in rtco deps)
2. **Classify** each array field:
   - `DictArray`: array of objects → TopN or ClusterSample
   - `StringArray`: array of strings → TopN or Sample
   - `NumberArray`: array of numbers → Sample (statistical)
   - `NestedArray`: nested arrays → TopN
   - `MixedArray`: mixed types → TopN (conservative)
   - `Empty`: skip
3. **Plan**: per-field compression strategy
4. **Execute**: compress accordingly
5. **Render**: back to JSON string

### File Breakdown

| File | Purpose | headroom LOC | rtco LOC (est) |
|------|---------|-------------|----------------|
| `src/core/compressors/smart_crusher/mod.rs` | SmartCrusher struct + pub interface | — | 80 |
| `src/core/compressors/smart_crusher/classifier.rs` | ArrayType classification | 300 | 200 |
| `src/core/compressors/smart_crusher/types.rs` | Strategies, FieldStats, CompressionPlan | 400 | 250 |
| `src/core/compressors/smart_crusher/crusher.rs` | crush_array, SmartCrusher impl | 400 | 300 |
| `src/core/compressors/smart_crusher/planner.rs` | Strategy selection logic | 250 | 200 |
| `src/core/compressors/smart_crusher/stats.rs` | Array statistics (mean, variance, etc.) | 200 | 150 |
| `src/cmds/json/mod.rs` | JSON filter using SmartCrusher | — | 100 |

**Total new code**: ~1280 lines (vs 2500 in headroom — simplified by not porting observer, hashing, outliers, error_keywords, constraints, orchestration)

### Integration

New filter module: `src/cmds/json/`
- Matches commands that produce JSON: `curl`, `jq`, `npm audit --json`, `cargo metadata`, etc.
- Routes to `SmartCrusher::compress()` for JSON output
- Detects JSON content type via `content_detector.rs` (already ported)
- Falls back to identity if JSON parse fails

### Dependencies

- `serde_json` (already in Cargo.toml) — with `preserve_order` feature

---

## 8. Phase 5: DiffCompressor

**headroom reference**: `transforms/diff_compressor.rs` (1685 lines)

**Goal**: Compress unified diff output by scoring hunks and lines, keeping important changes, trimming context.

### Architecture

```rust
pub struct DiffCompressorConfig {
    pub max_context_lines: usize,    // Default: 3 (vs git's 3 default)
    pub max_hunks_per_file: usize,   // Default: 20
    pub max_files: usize,            // Default: 50
    pub enable_ccr: bool,
    pub importance_threshold: f64,   // Min importance to keep a line
}

pub enum DiffLineScore {
    Essential(f64),   // Changed line (1.0)
    HighContext(f64),   // Context near change (0.7)
    LowContext(f64),    // Context far from change (0.3)
    Separator(f64),     // File separator (0.2)
    Noise(f64),         // Trimmable (0.0)
}
```

### Scoring Heuristics

| Pattern | Boost | Reason |
|---------|-------|--------|
| Changed lines (`-`/`+`) | 1.0 | Core semantic content |
| Context lines within 2 of change | 0.8 | Important for understanding |
| Context lines 3–5 from change | 0.5 | Moderately useful |
| Context lines >5 from change | 0.2 | Low value |
| File header (`diff --git`) | 1.0 | Essential |
| Hunk header (`@@ ... @@`) | 0.9 | Essential |
| Binary file markers | 0.5 | Low value for LLM |

### Pipeline

1. **Parse** unified diff using `unidiff` crate (or write minimal parser)
2. **Score** each hunk per-file
3. **Sort** hunks by average priority
4. **Select** top N hunks within token budget
5. **Trim** context lines per hunk to `max_context_lines`
6. **Offload** via CCR if enabled

### File Breakdown

| File | Purpose | headroom LOC | rtco LOC (est) |
|------|---------|-------------|----------------|
| `src/core/compressors/diff_compressor.rs` | DiffCompressor impl | 1685 | 900 |
| `src/cmds/git/diff_compressor.rs` | Integration into git filter | — | 150 |

**Savings**: ~60% of headroom complexity because:
- No multi-auth-mode policy (headroom handles PAYG vs Subscription — rtco doesn't)
- No Redis CCR backend
- Simplified configuration (no TOML config file needed initially)

### Integration

- Extended `src/cmds/git/mod.rs` to detect diff output and route to DiffCompressor
- Config: `git.diff_max_context_lines = 3` in rtco config

---

## 9. Phase 6: LogCompressor

**headroom reference**: `transforms/log_compressor.rs` (1295 lines)

**Goal**: Detect log output format, classify lines by log level, score and truncate noise while keeping errors.

### Architecture

```rust
pub enum LogFormat {
    Pytest, Npm, Cargo, Jest, Make, Generic,
}

pub enum LogLevel {
    Error, Fail, Warn, Info, Debug, Trace, Unknown,
}

pub struct LogCompressorConfig {
    pub max_lines_per_level: HashMap<LogLevel, usize>,
    // e.g., Error -> 100, Warn -> 50, Info -> 20, Debug -> 5
    pub preserve_error_context: usize,  // Lines before/after error to keep
    pub enable_template_detection: bool,  // Collapse repeated patterns
}

pub struct LogCompressor {
    pub config: LogCompressorConfig,
    pub format_detector: FormatDetector,  // Auto-detect format
}
```

### Pipeline

1. **Detect format**: Match first N lines against format signatures
2. **Classify**: Per-line log level detection
3. **Score**: Errors > Warnings > Info > Debug (by config)
4. **Select**: Keep all errors + context, fill remaining budget with lower levels
5. **Template collapse**: If enable_template_detection, replace repeated similar lines with `[N similar lines suppressed]`

### Template Detection

```rust
/// Collapse repeated lines of the same pattern
/// Example: 50 "Test foo ... FAILED" → "Test foo ... FAILED [×50]"
pub fn collapse_repeated(lines: &[LogLine], threshold: usize) -> Vec<LogLine>;
```

### File Breakdown

| File | Purpose | headroom LOC | rtco LOC (est) |
|------|---------|-------------|----------------|
| `src/core/compressors/log_compressor.rs` | LogCompressor impl | 1295 | 700 |
| `src/core/compressors/log_format.rs` | LogFormat, format detection, LogLevel | — | 200 |

**Savings**: ~55% of headroom complexity because:
- No multi-auth policies
- Simplified template detection (aho-corasick not needed — simple prefix matching suffices)
- No magika integration (rtco already has content_detector)

### Integration

- Shared lib used by `src/cmds/npm/`, `src/cmds/cargo/`, `src/cmds/pytest/`
- Each filter auto-detects if output looks like structured logs
- Config per-command: `npm.log_compression = true`

---

## 10. Phase 7: SearchCompressor

**headroom reference**: `transforms/search_compressor.rs` (877 lines)

**Goal**: Compress grep/rg/find output by scoring each match, keeping important files/matches.

### Architecture

```rust
pub struct SearchMatch {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub score: f64,
}

pub struct SearchFileGroup {
    pub file_path: String,
    pub matches: Vec<SearchMatch>,
    pub file_score: f64,  // Max + avg of match scores
}

pub struct SearchCompressorConfig {
    pub max_files: usize,           // Default: 20
    pub max_matches_per_file: usize, // Default: 10
    pub min_match_score: f64,       // Drop matches below this
    pub group_by_file: bool,        // Default: true
}
```

### Scoring

| Match Pattern | Score Boost |
|--------------|-------------|
| Match contains `error`/`fail`/`exception` | +0.3 |
| Match in source file (.rs, .py, .ts) | +0.2 |
| Match in test file | +0.1 |
| Match in config file | 0.0 |
| Match is a definition (fn, class, def) | +0.3 |
| Match is a comment/string | -0.2 |
| Very long lines (>200 chars) | -0.1 |

### Pipeline

1. **Parse** grep/rg output: `file:line:content`
2. **Group** by file
3. **Score** each match
4. **Score** each file (max match score + avg * 0.5)
5. **Sort** files by score descending
6. **Select** top files within budget
7. **Select** top matches per file
8. **Render** back to grep-like output

### File Breakdown

| File | Purpose | headroom LOC | rtco LOC (est) |
|------|---------|-------------|----------------|
| `src/core/compressors/search_compressor.rs` | SearchCompressor impl | 877 | 500 |

### Integration

- New filter: `src/cmds/search/mod.rs` — matches grep, rg, ag, find commands
- Config: `search.max_files = 20`

---

## 11. Phase 8: Anchor Selector + CacheAligner

### 11.1 Anchor Selector

**headroom reference**: `transforms/anchor_selector.rs` (1189 lines)

**Goal**: Identify and preserve "anchor" lines in output — lines that establish context, define structure, or serve as reference points.

#### Simplified Architecture (for rtco)

```rust
pub enum AnchorType {
    Header,           // Section headers, H1-H6 markers
    Command,          // Shell commands, `$ cmd`
    Path,             // File paths, URLs
    Key,              // Key=value lines
    Definition,       // def, fn, class, struct definitions
    Summary,          // Summary/statistics lines
}

pub struct AnchorSelector {
    pub preserve_anchors: bool,        // Default: true
    pub max_anchors: usize,            // Default: 20
    pub anchor_boost: f64,             // Priority boost for anchors (0.5)
}
```

#### Strategy

headroom's anchor selector is 1189 lines with sophisticated ML-like heuristics. For rtco:

1. Define 6 anchor types with regex patterns
2. Scan all lines for anchor matches
3. Preserve anchors even during aggressive truncation
4. Anchor priority overrides normal signal scoring

### 11.2 CacheAligner

**headroom reference**: `transforms/cache_control.rs` (part of 400 lines)

**Goal**: Align output to token boundaries that maximize LLM prefix caching (Anthropic prompt caching, etc.).

#### Simplified Architecture

```rust
pub struct CacheAligner {
    pub target_alignment: usize,  // Cache boundary (e.g., 1024 tokens)
    pub pad_token: String,        // Filler (e.g., "\n")
}
```

**Note**: CacheAligner is primarily useful for Anthropic's prompt caching API. For rtco CLI proxy, this is lower priority. The concept of "frozen zones" (lines that should never be truncated) is more immediately useful.

### File Breakdown

| File | headroom LOC | rtco LOC (est) |
|------|-------------|----------------|
| `src/core/anchor/mod.rs` | 1189 | 300 |
| `src/core/anchor/selectors.rs` | — | 200 |
| `src/core/cache_aligner.rs` | 400 | 150 |

---

## 12. Integration into rtco

### 12.1 Config System

New sections in rtco config (TOML):

```toml
[tokenizer]
backend = "approximate"  # approximate | tiktoken | huggingface

[pipeline]
max_tokens = 4096
reformat_transforms = ["json_minify"]
offload_threshold = 0.3

[ccr]
enabled = true
default_ttl_days = 7

[signals]
enable_keywords = true
custom_keywords = ["CRITICAL", "deprecated", "TODO"]

[smart_crusher]
max_depth = 10
min_array_size = 5

[diff]
max_context_lines = 3
max_hunks_per_file = 20

[log]
max_lines_per_level = { Error = 100, Warn = 50, Info = 20, Debug = 5 }

[search]
max_files = 20
max_matches_per_file = 10
```

### 12.2 Feature Flags (Cargo.toml)

```toml
[features]
default = ["tokenizer-approximate", "ccr-sqlite"]
tokenizer-approximate = []
tokenizer-tiktoken = ["dep:tiktoken-rs"]
tokenizer-hf = ["dep:tokenizers", "dep:hf-hub"]
ccr-sqlite = ["dep:rusqlite"]
smart-crusher = []
pipeline-orchestrator = []

[dependencies]
# New optional deps
tiktoken-rs = { version = "0.5", optional = true }
tokenizers = { version = "0.19", optional = true }
rusqlite = { version = "0.31", features = ["bundled"], optional = true }
unidiff = { version = "0.3", optional = true }
```

### 12.3 Tracking & Statistics

All new modules report into `src/core/tracking.rs`:

| Metric | Source |
|--------|--------|
| Token count before/after | Tokenizer |
| Lines kept/dropped by priority | Signals |
| CCR store hit rate | CCR |
| CCR bytes stored | CCR |
| Lines offloaded | Pipeline |
| JSON arrays crushed | SmartCrusher |
| Diff context lines trimmed | DiffCompressor |
| Log lines by level preserved/dropped | LogCompressor |

### 12.4 Graceful Degradation

Every new module follows rtco's fallback pattern:

```rust
fn compress_output(output: &str, config: &Config) -> Result<String> {
    // Try compression
    let compressed = try_compress(output, config)
        .context("SmartCrusher failed")?;
    Ok(compressed)
}

fn try_compress(output: &str, config: &Config) -> Result<String> {
    // If any step fails, return original
    let json = serde_json::from_str(output)
        .context("Not valid JSON")?;
    // ... compress
}
```

---

## 13. Testing Strategy

### 13.1 Unit Tests (per module)

| Module | Test Type | Fixtures |
|--------|-----------|----------|
| Tokenizer | Known token counts | Pre-computed text samples |
| CcrStore | put/get/len roundtrip, expiry | SQLite in-memory db |
| Signals | Priority scoring accuracy | Known lines (error, warning, etc.) |
| SmartCrusher | JSON compression ratios | Real API responses (npm audit --json, cargo metadata) |
| DiffCompressor | Hunk selection, context trimming | Unified diff output (git log -p) |
| LogCompressor | Format detection, level classification | pytest, npm, cargo output logs |
| SearchCompressor | Grouping, scoring | grep/rg output samples |
| Anchor Selector | Anchor detection | Mixed content samples |
| Pipeline | End-to-end compression | Integration fixtures |

### 13.2 Snapshot Tests (insta)

Use rtco's existing `insta` snapshot pattern:

```rust
#[test]
fn test_smart_crusher_npm_audit() {
    let input = include_str!("../../fixtures/smart_crusher/npm_audit.json");
    let output = SmartCrusher::compress(input).unwrap();
    insta::assert_snapshot!("npm_audit_crushed", output);
}
```

### 13.3 Token Savings Assertions

```rust
#[test]
fn test_diff_compressor_savings() {
    let input = include_str!("../../fixtures/diff/large_diff.diff");
    let compressed = DiffCompressor::compress(input).unwrap();
    let savings = 1.0 - (compressed.len() as f64 / input.len() as f64);
    assert!(savings > 0.5, "DiffCompressor should save >50%");
}
```

### 13.4 Integration Tests

- Each new filter must pass the `bash scripts/test-all.sh` smoke test suite
- New tests added to smoke test suite for JSON, diff, log, and search commands

---

## 14. Performance Targets

| Module | Startup Overhead | Per-Operation | Memory |
|--------|-----------------|---------------|--------|
| Tokenizer (approximate) | <1μs | <1μs | 0 |
| Tokenizer (tiktoken) | <10ms init | <50μs | ~2MB |
| CcrStore (SQLite) | <5ms init | <1ms put/get | Configurable |
| Signals | <100μs init | <1μs per line | <100KB |
| SmartCrusher | <1ms init | <50ms (10MB JSON) | <10MB |
| DiffCompressor | <1ms init | <20ms (1000 lines) | <5MB |
| LogCompressor | <1ms init | <10ms (1000 lines) | <2MB |
| SearchCompressor | <1ms init | <10ms (1000 matches) | <2MB |
| Pipeline orchestrator | <1ms init | <100ms worst case | <20MB |

**Overall rtco constraints**: <10ms startup, <5MB baseline memory

The new modules are designed to be lazy-loaded — initialized only when the relevant command is invoked.

---

## 15. Files & Modules Reference

### New Files

```
src/
├── core/
│   ├── tokenizer/
│   │   ├── mod.rs          # Trait, TokenizerKind enum
│   │   ├── estimator.rs    # Approximate token counter
│   │   ├── tiktoken_impl.rs # TikToken backend (feature-gated)
│   │   ├── hf_impl.rs      # HF backend (feature-gated)
│   │   └── registry.rs     # Registry + auto-detect
│   ├── ccr/
│   │   ├── mod.rs          # CcrStore trait, compute_key, marker_for
│   │   ├── store.rs        # SqliteCcrStore
│   │   └── memory.rs       # InMemoryCcrStore (testing)
│   ├── signals/
│   │   ├── mod.rs          # LineImportanceDetector trait, types
│   │   ├── detectors.rs    # Built-in detectors
│   │   ├── tiered.rs       # TieredDetector composition
│   │   └── keyword_detector.rs
│   ├── pipeline/
│   │   ├── mod.rs          # CompressionPipeline
│   │   ├── config.rs       # PipelineConfig
│   │   ├── reformat.rs     # ReformatTransform trait + impls
│   │   ├── offload.rs      # OffloadTransform trait + impls
│   │   └── runner.rs       # Pipeline execution
│   ├── compressors/
│   │   ├── mod.rs
│   │   ├── smart_crusher.rs → smart_crusher/ (directory)
│   │   ├── diff_compressor.rs
│   │   ├── log_compressor.rs
│   │   └── search_compressor.rs
│   └── anchor/
│       ├── mod.rs          # AnchorSelector
│       └── selectors.rs    # Anchor type detectors
├── cmds/
│   ├── json/
│   │   └── mod.rs          # JSON filter (uses SmartCrusher)
│   └── search/
│       └── mod.rs          # Search filter (uses SearchCompressor)
```

### Modified Files

```
src/
├── cmds/
│   ├── git/mod.rs          # + DiffCompressor integration
│   ├── npm/mod.rs          # + LogCompressor integration
│   ├── cargo/mod.rs        # + LogCompressor integration
│   └── ... (other filters)
├── core/
│   ├── mod.rs              # + pub mod tokenizer, ccr, signals, pipeline, compressors, anchor
│   ├── tracking.rs         # + new metrics
│   ├── adaptive_sizer.rs   # + tokenizer param, signal-guided truncation
│   └── config.rs           # + new config sections
├── cmds/mod.rs             # + json, search (if routing via cmd_router)
├── main.rs                 # + init new modules
Cargo.toml                  # + features, optional deps
```

---

## 16. Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| SQLite CCR increases binary size | Medium | Medium | Bundle partial; make feature-gated |
| tiktoken-rs adds 10+ deps | Medium | High | Feature-gate; default is approximate |
| SmartCrusher too complex to port fully | High | Low | Port only classifier + basic strategies; skip observer/hashing/outliers |
| Pipeline orchestrator slows startup | Medium | Low | Lazy initialization; init on first use |
| JSON filter too broad (catches non-JSON) | Low | Medium | Content detection via existing detector |
| Duplicate compression (filter + pipeline) | Medium | Medium | Clear separation: filter = detection, pipeline = compression |
| SearchCompressor regex parsing fragile | Medium | Low | Support multiple grep output formats; fallback on parse failure |
| LogCompressor template detection too aggressive | Low | Medium | Configurable threshold; off by default |
| Maintenance burden of 2000+ new LOC | Medium | Medium | Modular design; each module independently testable |
| Feature creep (porting everything) | High | Medium | Strict scope: stop at 10 modules; reevaluate after Phase 4 |

### Immediate Next Steps (Week 1)

1. Create `src/core/tokenizer/` module with approximate counter (no new deps)
2. Create `LineImportanceDetector` trait + basic error/warning detectors
3. Wire tokenizer into `AdaptiveSizer` as optional parameter
4. Upgrade `AdaptiveSizer` to use signals for line selection
5. All above requires zero new dependencies

---

*Generated: 2026-06-07*
*Author: Sisyphus (orchestrator)*
