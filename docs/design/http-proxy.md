# HTTP Proxy — Design Document

## Overview

The HTTP proxy mode (`rtco serve`) runs RTCO as a long-lived HTTP server that
accepts CLI output payloads and returns compressed versions suitable for LLM
context windows.  This enables tight integration with tool-use agents,
continuous integration pipelines, and IDE plugins without requiring a local
binary invocation per request.

## Architecture

```
┌──────────────┐      POST /compress      ┌──────────────────┐
│  Agent / CI  │  ──────────────────────>  │   rtco serve     │
│  / IDE       │                          │   (HTTP server)   │
│              │  <──────────────────────  │                  │
│              │   { compressed, stats }   └──────────────────┘
└──────────────┘                                  │
                                                   │  detect content type
                                                   ▼
                                          ┌──────────────────┐
                                          │  Content Router   │
                                          │  (Bead 34)        │
                                          └──────────────────┘
                                                  │
                                        ┌─────────┼──────────┐
                                        ▼         ▼          ▼
                                  JSON       Code/Logs    Plain
                                  Handler    Handler      Handler
```

### Server lifecycle

1. **Startup** — parse CLI flags, bind to `0.0.0.0:<port>`, load filter configs.
2. **Request loop** — each `POST /compress` is handled synchronously.
3. **Graceful shutdown** — `SIGTERM` / `SIGINT` drains in-flight requests.

## Endpoints

### `POST /compress`

Request:
```json
{
  "content": "raw CLI output to compress",
  "content_type": "auto | json | code | logs | plain | git_diff | html",
  "options": {
    "max_tokens": 4000,
    "strip_ansi": true,
    "deduplicate": true
  }
}
```

Response (200):
```json
{
  "compressed": "the compressed output",
  "original_tokens": 15000,
  "compressed_tokens": 1200,
  "savings_percent": 92.0,
  "content_type": "logs",
  "handler": "build-log-compressor",
  "elapsed_ms": 1.2
}
```

### `GET /health`

```json
{ "status": "ok", "version": "0.41.0", "uptime_secs": 86400 }
```

### `POST /analyze`

Same input as `/compress` but returns analysis without compression:
```json
{
  "detected_type": "logs",
  "line_count": 500,
  "estimated_tokens": 15000,
  "redundant_lines": 120,
  "recommended_handler": "build-log-compressor"
}
```

## Provider Handlers

Each handler is a registered `ContentHandler` (see Bead 34) that the router
selects based on content type:

| Provider | Handler | Strategy |
|----------|---------|----------|
| JSON | `JsonHandler` | Strip insignificant whitespace, compact arrays |
| Code | `CodeHandler` | Remove comments (language-aware), dedent, condense blank lines |
| Logs | `LogsHandler` | Remove timestamps, dedup repeated errors, group stack traces |
| PlainText | `PassthroughHandler` | Identity (no compression) |
| GitDiff | `GitDiffHandler` | Strip diff metadata, keep only changed hunks |
| HTML | `HtmlHandler` | Strip tags, extract text content |

## Semantic Caching

To avoid re-compressing the same output repeatedly, the proxy maintains an
in-memory LRU cache keyed by SHA-256 of the input content:

```rust
struct SemanticCache {
    cache: LruCache<[u8; 32], CacheEntry>,
    max_entries: usize,
}

struct CacheEntry {
    compressed: String,
    original_tokens: usize,
    compressed_tokens: usize,
    created_at: Instant,
}
```

**Cache invalidation**: entries expire after TTL (configurable, default 60 s).
When the cache is full, the least-recently-used entry is evicted.

**Cache key design**: full SHA-256 of input bytes.  A semantic content-type
aware approach (fingerprinting structural features) is deferred to a later
milestone.

## Rate Limiting

Per-client token-bucket rate limiting protects the server from abuse:

- **Burst**: 100 requests
- **Refill rate**: 10 requests/second
- **Per-IP tracking**: in-memory hash map with periodic GC
- **Headers**: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

When exceeded, respond with `429 Too Many Requests`:
```json
{ "error": "rate_limit_exceeded", "retry_after_secs": 5 }
```

Rate limits are only active when `--rate-limit` is passed; disabled by default
for local development.

## Configuration

CLI flags for `rtco serve`:

| Flag | Default | Description |
|------|---------|-------------|
| `--port` / `-p` | `8721` | TCP port |
| `--host` | `0.0.0.0` | Bind address |
| `--cache-size` | `1024` | Max LRU cache entries (0 = off) |
| `--cache-ttl` | `60` | Cache TTL in seconds |
| `--rate-limit` | `0` | Requests/second (0 = unlimited) |
| `--timeout` | `5000` | Request timeout in ms |
| `--max-content-length` | `10485760` | Max body size in bytes (10 MB) |

## Milestones

### M1 — Core proxy (MVP)
- [ ] Add `serve` subcommand to `main.rs` Commands enum
- [ ] Single `POST /compress` endpoint with synchronous handler dispatch
- [ ] Integration with `content_router::ContentRouter`
- [ ] `GET /health` endpoint

### M2 — Provider handlers & analysis
- [ ] Implement all 6 stub handlers with real compression logic
- [ ] `POST /analyze` endpoint
- [ ] Content-type override in request body

### M3 — Performance
- [ ] Semantic LRU caching
- [ ] Configurable cache TTL and max entries
- [ ] Benchmark target: P50 < 5 ms, P99 < 20 ms (for cached responses)

### M4 — Production hardening
- [ ] Rate limiting per-IP
- [ ] Request timeout middleware
- [ ] Max content length enforcement
- [ ] Graceful shutdown via signal handlers
- [ ] Structured logging (JSON to stderr)

### M5 — Observability
- [ ] Prometheus `/metrics` endpoint (see Bead 38)
- [ ] Request-level tracing
- [ ] Per-handler latency histograms

## Dependencies

- **HTTP server**: `tiny_http` (lightweight, single-threaded, matches RTCO's
  no-async constraint) or `ureq` in server role if `tiny_http` is unavailable.
- **Caching**: `lru` crate for the semantic cache.
- **Rate limiting**: hand-rolled token bucket (no extra dependency).

All dependencies must be added to `crates/rtco-cli/Cargo.toml` and gated behind
a `proxy` feature flag to keep the base binary lean.

## Open Questions

1. **TLS termination** — should the proxy handle HTTPS directly, or rely on a
   reverse proxy (nginx, Caddy)?  Decision: rely on reverse proxy for TLS.
2. **Unix socket support** — useful for local agent integration without port
   allocation.  Deferred to M4.
3. **Streaming compression** — for very large outputs (>10 MB), a streaming
   endpoint could return compressed chunks as they are processed.  Deferred.
